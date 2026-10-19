use std::collections::{HashMap, HashSet};
use std::io::{Read, Seek};

use percent_encoding::percent_decode_str;
use roxmltree::{Document, Node};
use scraper::{Html, Selector};
use zip::{CompressionMethod, ZipArchive};

use crate::content::parse_chapter;
use crate::{Book, BookError, BookMetadata};

const MAX_ARCHIVE_ENTRIES: usize = 100_000;
const MAX_ARCHIVE_BYTES: u64 = 2 * 1024 * 1024 * 1024;
const MAX_COMPRESSION_RATIO: u64 = 100;
const MAX_XML_BYTES: u64 = 32 * 1024 * 1024;
const MAX_COVER_BYTES: u64 = 20 * 1024 * 1024;
const MAX_SPINE_REFERENCES: usize = 10_000;
const MAX_READING_BYTES: u64 = 256 * 1024 * 1024;
const DC_NAMESPACE: &str = "http://purl.org/dc/elements/1.1/";
const FONT_OBFUSCATION: [&str; 2] = [
    "http://www.idpf.org/2008/embedding",
    "http://ns.adobe.com/pdf/enc#RC",
];

pub fn parse_epub<R: Read + Seek>(source: R) -> Result<Book, BookError> {
    let mut archive = ZipArchive::new(source).map_err(|_| BookError::InvalidArchive)?;
    validate_entries(&mut archive)?;
    validate_mimetype(&mut archive)?;
    ensure_not_protected(&mut archive)?;
    let package_path = package_path(&mut archive)?;
    let package_source = read_xml(&mut archive, &package_path)?;
    let package = Document::parse(&package_source).map_err(|_| BookError::InvalidPackage)?;
    let metadata = read_metadata(&package)?;
    let mut chapters = Vec::new();
    read_chapters(
        &mut archive,
        &package,
        &package_path,
        |_| {},
        |_, chapter| chapters.push(chapter),
    )?;
    let cover = read_cover(&mut archive, &package, &package_path)?;
    Ok(Book {
        metadata,
        chapters,
        cover,
    })
}

pub fn parse_epub_progressive<R, S, C, V>(
    source: R,
    mut on_start: S,
    on_chapter: C,
    mut on_cover: V,
) -> Result<(), BookError>
where
    R: Read + Seek,
    S: FnMut(BookMetadata, usize),
    C: FnMut(usize, crate::Chapter),
    V: FnMut(Option<Vec<u8>>),
{
    let mut archive = ZipArchive::new(source).map_err(|_| BookError::InvalidArchive)?;
    validate_entries(&mut archive)?;
    validate_mimetype(&mut archive)?;
    ensure_not_protected(&mut archive)?;
    let package_path = package_path(&mut archive)?;
    let package_source = read_xml(&mut archive, &package_path)?;
    let package = Document::parse(&package_source).map_err(|_| BookError::InvalidPackage)?;
    let metadata = read_metadata(&package)?;
    read_chapters(
        &mut archive,
        &package,
        &package_path,
        |total| on_start(metadata.clone(), total),
        on_chapter,
    )?;
    on_cover(read_cover(&mut archive, &package, &package_path)?);
    Ok(())
}

fn validate_entries<R: Read + Seek>(archive: &mut ZipArchive<R>) -> Result<(), BookError> {
    if archive.is_empty() || archive.len() > MAX_ARCHIVE_ENTRIES {
        return Err(BookError::ArchiveLimit);
    }
    let mut total_size = 0u64;
    let mut names = HashSet::new();
    for index in 0..archive.len() {
        let entry = archive
            .by_index(index)
            .map_err(|_| BookError::InvalidArchive)?;
        let name = entry.name().map_err(|_| BookError::InvalidArchive)?;
        let normalized = normalize_path("", &name)?;
        if !names.insert(normalized) {
            return Err(BookError::InvalidArchive);
        }
        total_size = total_size
            .checked_add(entry.size())
            .ok_or(BookError::ArchiveLimit)?;
        if total_size > MAX_ARCHIVE_BYTES
            || compression_is_excessive(entry.size(), entry.compressed_size())
        {
            return Err(BookError::ArchiveLimit);
        }
    }
    Ok(())
}

fn compression_is_excessive(expanded: u64, compressed: u64) -> bool {
    expanded > 0
        && (compressed == 0
            || compressed
                .checked_mul(MAX_COMPRESSION_RATIO)
                .is_some_and(|maximum| expanded > maximum))
}

fn validate_mimetype<R: Read + Seek>(archive: &mut ZipArchive<R>) -> Result<(), BookError> {
    let first = archive.by_index(0).map_err(|_| BookError::InvalidArchive)?;
    if first.name().map_err(|_| BookError::InvalidArchive)? != "mimetype"
        || first.compression() != CompressionMethod::Stored
    {
        return Err(BookError::InvalidPackage);
    }
    let mut value = Vec::new();
    first
        .take(64)
        .read_to_end(&mut value)
        .map_err(|_| BookError::InvalidArchive)?;
    if value.as_slice() != b"application/epub+zip" {
        return Err(BookError::InvalidPackage);
    }
    Ok(())
}

fn ensure_not_protected<R: Read + Seek>(archive: &mut ZipArchive<R>) -> Result<(), BookError> {
    if has_entry(archive, "META-INF/license.lcpl")? || has_entry(archive, "META-INF/rights.xml")? {
        return Err(BookError::Protected);
    }
    if has_entry(archive, "META-INF/encryption.xml")? {
        let source = read_xml(archive, "META-INF/encryption.xml")?;
        let document = Document::parse(&source).map_err(|_| BookError::Protected)?;
        let algorithms = document
            .descendants()
            .filter(|node| node.has_tag_name("EncryptionMethod"))
            .filter_map(|node| node.attribute("Algorithm"));
        let mut found = false;
        for algorithm in algorithms {
            found = true;
            if !FONT_OBFUSCATION.contains(&algorithm) {
                return Err(BookError::Protected);
            }
        }
        if !found {
            return Err(BookError::Protected);
        }
    }
    Ok(())
}

fn has_entry<R: Read + Seek>(archive: &ZipArchive<R>, expected: &str) -> Result<bool, BookError> {
    for name in archive.file_names() {
        if name.map_err(|_| BookError::InvalidArchive)?.as_ref() == expected {
            return Ok(true);
        }
    }
    Ok(false)
}

fn package_path<R: Read + Seek>(archive: &mut ZipArchive<R>) -> Result<String, BookError> {
    let container = read_xml(archive, "META-INF/container.xml")?;
    let document = Document::parse(&container).map_err(|_| BookError::MissingPackage)?;
    let path = document
        .descendants()
        .find(|node| node.has_tag_name("rootfile"))
        .and_then(|node| node.attribute("full-path"))
        .ok_or(BookError::MissingPackage)?;
    normalize_path("", path)
}

fn read_xml<R: Read + Seek>(archive: &mut ZipArchive<R>, path: &str) -> Result<String, BookError> {
    let entry = archive
        .by_name(path)
        .map_err(|_| BookError::MissingPackage)?;
    if entry.size() > MAX_XML_BYTES {
        return Err(BookError::ArchiveLimit);
    }
    let mut bytes = Vec::new();
    entry
        .take(MAX_XML_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| BookError::InvalidArchive)?;
    if bytes.len() as u64 > MAX_XML_BYTES {
        return Err(BookError::ArchiveLimit);
    }
    String::from_utf8(bytes).map_err(|_| BookError::InvalidPackage)
}

fn read_metadata(package: &Document<'_>) -> Result<BookMetadata, BookError> {
    let metadata = package
        .descendants()
        .find(|node| node.has_tag_name("metadata"))
        .ok_or(BookError::InvalidPackage)?;
    let title = metadata
        .descendants()
        .find(|node| node.has_tag_name((DC_NAMESPACE, "title")))
        .and_then(node_text)
        .unwrap_or_else(|| "Untitled".to_owned());
    let authors = metadata
        .descendants()
        .filter(|node| node.has_tag_name((DC_NAMESPACE, "creator")))
        .filter_map(node_text)
        .filter(|value| !value.is_empty())
        .collect();
    let language = metadata
        .descendants()
        .find(|node| node.has_tag_name((DC_NAMESPACE, "language")))
        .and_then(node_text)
        .filter(|value| !value.is_empty());
    let description = metadata
        .descendants()
        .find(|node| node.has_tag_name((DC_NAMESPACE, "description")))
        .and_then(node_text);
    let publisher = metadata
        .descendants()
        .find(|node| node.has_tag_name((DC_NAMESPACE, "publisher")))
        .and_then(node_text);
    let date = metadata
        .descendants()
        .find(|node| node.has_tag_name((DC_NAMESPACE, "date")))
        .and_then(node_text);
    let subjects = metadata
        .descendants()
        .filter(|node| node.has_tag_name((DC_NAMESPACE, "subject")))
        .filter_map(node_text)
        .collect();
    let series = package_series(package);
    Ok(BookMetadata {
        title,
        authors,
        language,
        description,
        publisher,
        date,
        subjects,
        series,
    })
}

fn package_series(package: &Document<'_>) -> Option<String> {
    package
        .descendants()
        .filter(|node| node.has_tag_name("meta"))
        .find(|node| node.attribute("name") == Some("calibre:series"))
        .and_then(|node| node.attribute("content"))
        .map(str::to_owned)
}

fn read_cover<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    package: &Document<'_>,
    package_path: &str,
) -> Result<Option<Vec<u8>>, BookError> {
    let manifest = package
        .descendants()
        .find(|node| node.has_tag_name("manifest"))
        .ok_or(BookError::InvalidPackage)?;
    let cover_id = package
        .descendants()
        .filter(|node| node.has_tag_name("meta"))
        .find(|node| node.attribute("name") == Some("cover"))
        .and_then(|node| node.attribute("content"));
    let item = manifest.children().find(|node| {
        node.has_tag_name("item")
            && (node
                .attribute("properties")
                .is_some_and(|value| value.split_whitespace().any(|part| part == "cover-image"))
                || node.attribute("id") == cover_id)
    });
    let Some(href) = item.and_then(|node| node.attribute("href")) else {
        return Ok(None);
    };
    let path = resolve_href(package_path, href)?;
    let entry = archive
        .by_name(&path)
        .map_err(|_| BookError::InvalidPackage)?;
    if entry.size() > MAX_COVER_BYTES {
        return Err(BookError::ArchiveLimit);
    }
    let mut bytes = Vec::with_capacity(entry.size() as usize);
    entry
        .take(MAX_COVER_BYTES + 1)
        .read_to_end(&mut bytes)
        .map_err(|_| BookError::InvalidArchive)?;
    if bytes.len() as u64 > MAX_COVER_BYTES {
        return Err(BookError::ArchiveLimit);
    }
    Ok(Some(bytes))
}

fn node_text(node: Node<'_, '_>) -> Option<String> {
    let value = node
        .text()?
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    (!value.is_empty()).then_some(value)
}

fn read_chapters<R, S, C>(
    archive: &mut ZipArchive<R>,
    package: &Document<'_>,
    package_path: &str,
    mut on_ready: S,
    mut on_chapter: C,
) -> Result<(), BookError>
where
    R: Read + Seek,
    S: FnMut(usize),
    C: FnMut(usize, crate::Chapter),
{
    let manifest = package
        .descendants()
        .find(|node| node.has_tag_name("manifest"))
        .ok_or(BookError::InvalidPackage)?;
    let items = manifest
        .children()
        .filter(|node| node.has_tag_name("item"))
        .filter_map(|node| Some((node.attribute("id")?.to_owned(), Item::from_node(node)?)))
        .collect::<HashMap<_, _>>();
    let title_map = read_navigation_titles(archive, &items, package_path)?;
    let spine = package
        .descendants()
        .find(|node| node.has_tag_name("spine"))
        .ok_or(BookError::InvalidPackage)?;
    let mut spine_items = Vec::new();
    let mut reading_bytes = 0_u64;
    for reference in spine.children().filter(|node| node.has_tag_name("itemref")) {
        let id = reference
            .attribute("idref")
            .ok_or(BookError::InvalidPackage)?;
        let item = items.get(id).ok_or(BookError::InvalidPackage)?;
        if item.media_type != "application/xhtml+xml" {
            continue;
        }
        let path = resolve_href(package_path, &item.href)?;
        reading_bytes = reading_bytes
            .checked_add(
                archive
                    .by_name(&path)
                    .map_err(|_| BookError::InvalidPackage)?
                    .size(),
            )
            .ok_or(BookError::ArchiveLimit)?;
        if spine_items.len() >= MAX_SPINE_REFERENCES || reading_bytes > MAX_READING_BYTES {
            return Err(BookError::ArchiveLimit);
        }
        let fallback = title_map
            .get(&path)
            .cloned()
            .unwrap_or_else(|| fallback_chapter_title(&item.href));
        spine_items.push((path, fallback));
    }
    if spine_items.is_empty() {
        return Err(BookError::InvalidPackage);
    }
    on_ready(spine_items.len());
    for (index, (path, fallback)) in spine_items.into_iter().enumerate() {
        let source = read_xml(archive, &path)?;
        on_chapter(index, parse_chapter(&source, &fallback));
    }
    Ok(())
}

struct Item {
    href: String,
    media_type: String,
    properties: String,
}

impl Item {
    fn from_node(node: Node<'_, '_>) -> Option<Self> {
        Some(Self {
            href: node.attribute("href")?.to_owned(),
            media_type: node.attribute("media-type")?.to_owned(),
            properties: node.attribute("properties").unwrap_or_default().to_owned(),
        })
    }
}

fn read_navigation_titles<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    items: &HashMap<String, Item>,
    package_path: &str,
) -> Result<HashMap<String, String>, BookError> {
    let nav = items.values().find(|item| {
        item.properties
            .split_whitespace()
            .any(|value| value == "nav")
    });
    if let Some(nav) = nav {
        let path = resolve_href(package_path, &nav.href)?;
        return read_html_navigation(archive, &path);
    }
    let ncx = items
        .values()
        .find(|item| item.media_type == "application/x-dtbncx+xml");
    if let Some(ncx) = ncx {
        let path = resolve_href(package_path, &ncx.href)?;
        return read_ncx_navigation(archive, &path);
    }
    Ok(HashMap::new())
}

fn read_html_navigation<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    nav_path: &str,
) -> Result<HashMap<String, String>, BookError> {
    let source = read_xml(archive, nav_path)?;
    let document = Html::parse_document(&source);
    let selector = Selector::parse("nav a[href]").map_err(|_| BookError::InvalidPackage)?;
    let mut titles = HashMap::new();
    for anchor in document.select(&selector) {
        let Some(href) = anchor.value().attr("href") else {
            continue;
        };
        let title = anchor
            .text()
            .collect::<String>()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ");
        if !title.is_empty() {
            titles.insert(resolve_href(nav_path, href)?, title);
        }
    }
    Ok(titles)
}

fn read_ncx_navigation<R: Read + Seek>(
    archive: &mut ZipArchive<R>,
    ncx_path: &str,
) -> Result<HashMap<String, String>, BookError> {
    let source = read_xml(archive, ncx_path)?;
    let document = Document::parse(&source).map_err(|_| BookError::InvalidPackage)?;
    let mut titles = HashMap::new();
    for point in document
        .descendants()
        .filter(|node| node.has_tag_name("navPoint"))
    {
        let title = point
            .descendants()
            .find(|node| node.has_tag_name("navLabel"))
            .and_then(|node| node.descendants().find(|node| node.has_tag_name("text")))
            .and_then(node_text);
        let href = point
            .descendants()
            .find(|node| node.has_tag_name("content"))
            .and_then(|node| node.attribute("src"));
        if let (Some(title), Some(href)) = (title, href) {
            titles.insert(resolve_href(ncx_path, href)?, title);
        }
    }
    Ok(titles)
}

fn fallback_chapter_title(href: &str) -> String {
    let stem = href
        .rsplit('/')
        .next()
        .unwrap_or(href)
        .rsplit_once('.')
        .map_or(href, |(stem, _)| stem);
    let words = stem
        .replace(['_', '-'], " ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    if words.to_lowercase().starts_with("chapter") {
        let number = words.trim_start_matches(|character: char| !character.is_ascii_digit());
        if !number.is_empty() {
            return format!("Capítulo {number}");
        }
    }
    words
}

fn resolve_href(package_path: &str, href: &str) -> Result<String, BookError> {
    let without_fragment = href.split('#').next().unwrap_or_default();
    let without_query = without_fragment.split('?').next().unwrap_or_default();
    let decoded = percent_decode_str(without_query)
        .decode_utf8()
        .map_err(|_| BookError::UnsafePath)?;
    let base = package_path
        .rsplit_once('/')
        .map_or("", |(directory, _)| directory);
    normalize_path(base, decoded.as_ref())
}

fn normalize_path(base: &str, path: &str) -> Result<String, BookError> {
    if path.starts_with('/') || path.contains(['\\', '\0', ':']) {
        return Err(BookError::UnsafePath);
    }
    let mut parts = base
        .split('/')
        .filter(|part| !part.is_empty())
        .map(str::to_owned)
        .collect::<Vec<_>>();
    for part in path.split('/') {
        match part {
            "" | "." => {}
            ".." if parts.pop().is_some() => {}
            ".." => return Err(BookError::UnsafePath),
            value => parts.push(value.to_owned()),
        }
    }
    Ok(parts.join("/"))
}
