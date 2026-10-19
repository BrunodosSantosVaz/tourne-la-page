use std::io::Cursor;

use epub_core::{BookError, parse_epub, parse_epub_progressive};

#[test]
fn reads_epub_metadata_spine_and_clean_text() {
    let book = parse_epub(Cursor::new(minimal_epub())).expect("fixture should be a valid EPUB");

    assert_eq!(book.metadata.title, "The Local Reader");
    assert_eq!(book.metadata.authors, ["Ada Example"]);
    assert_eq!(book.metadata.subjects, ["Fiction"]);
    assert_eq!(book.metadata.series.as_deref(), Some("Local Examples"));
    assert_eq!(book.chapters.len(), 2);
    assert_eq!(book.chapters[0].title, "First chapter");
    assert!(book.chapters[0].text.contains("Hello, reader!"));
    assert!(book.chapters[0].text.contains("A second paragraph."));
    assert!(!book.chapters[0].text.contains("hidden-script-marker"));
    assert!(!book.chapters[0].text.contains("hidden-style-marker"));
    assert!(book.chapters[1].text.contains("A new chapter."));
}

#[test]
fn progressive_parser_emits_metadata_then_chapters_in_reading_order() {
    let mut started = None;
    let mut chapters = Vec::new();
    let mut cover = None;
    parse_epub_progressive(
        Cursor::new(minimal_epub()),
        |metadata, total| started = Some((metadata, total)),
        |index, chapter| chapters.push((index, chapter)),
        |image| cover = image,
    )
    .expect("fixture should stream successfully");

    let (metadata, total) = started.expect("metadata is emitted before chapter content");
    assert_eq!(metadata.title, "The Local Reader");
    assert_eq!(total, 2);
    assert_eq!(
        chapters.iter().map(|(index, _)| *index).collect::<Vec<_>>(),
        [0, 1]
    );
    assert_eq!(chapters[0].1.title, "First chapter");
    assert!(cover.is_none());
}

#[test]
fn uses_navigation_titles_instead_of_xml_filenames() {
    let source =
        r#"<html><head><title>chapter-01.xhtml</title></head><body><p>Text</p></body></html>"#;
    let book = parse_epub(Cursor::new(epub_with_chapter(source))).unwrap();

    assert_eq!(book.chapters[0].title, "Opening the book");
}

#[test]
fn reads_cover_image_from_the_epub_manifest() {
    let bytes = epub_with_cover(b"local-cover-data");
    let book = parse_epub(Cursor::new(bytes)).expect("cover fixture should be valid");

    assert_eq!(book.cover.as_deref(), Some(b"local-cover-data".as_slice()));
}

#[test]
fn opens_the_distributable_local_demo_book() {
    let path = format!(
        "{}/../../examples/amostra-tournepage.epub",
        env!("CARGO_MANIFEST_DIR")
    );
    let book = parse_epub(std::fs::File::open(path).unwrap()).expect("demo EPUB should be valid");

    assert_eq!(book.metadata.title, "Uma pausa para ler");
    assert_eq!(book.chapters.len(), 2);
    assert!(book.chapters[0].text.contains("Este livro de demonstracao"));
}

#[test]
fn rejects_non_zip_input_with_a_domain_error() {
    let error = parse_epub(Cursor::new(b"not a zip archive")).unwrap_err();

    assert!(matches!(error, BookError::InvalidArchive));
}

#[test]
fn does_not_open_an_epub_marked_with_drm_encryption() {
    let bytes = epub_with_extra_file("META-INF/encryption.xml", b"<encryption/> ");
    let error = parse_epub(Cursor::new(bytes)).unwrap_err();

    assert!(matches!(error, BookError::Protected));
}

#[test]
fn rejects_paths_that_escape_the_archive_root() {
    let bytes = epub_with_extra_file("../outside.txt", b"private");
    let error = parse_epub(Cursor::new(bytes)).unwrap_err();

    assert!(matches!(error, BookError::UnsafePath));
}

#[test]
fn rejects_excessive_compression_before_reading_chapter_data() {
    let expanded = format!(
        "<html><body><p>{}</p></body></html>",
        "repeat ".repeat(80_000)
    );
    let bytes = epub_with_chapter(&expanded);
    let error = parse_epub(Cursor::new(bytes)).unwrap_err();

    assert!(matches!(error, BookError::ArchiveLimit));
}

#[test]
fn rejects_excessive_spine_references_before_emitting_chapters() {
    let bytes = repeated_spine(10_001, CHAPTER_ONE);
    assert!(matches!(
        parse_epub(Cursor::new(&bytes)),
        Err(BookError::ArchiveLimit)
    ));
    let mut emitted = 0;
    let result = parse_epub_progressive(Cursor::new(bytes), |_, _| {}, |_, _| emitted += 1, |_| {});
    assert!(matches!(result, Err(BookError::ArchiveLimit)));
    assert_eq!(emitted, 0);
}

#[test]
fn repeated_chapters_cannot_amplify_the_total_text_budget() {
    let source = format!(
        "<html><body><p>{}</p></body></html>",
        "a".repeat(2 * 1024 * 1024)
    );
    let bytes = repeated_spine(129, &source);
    assert!(matches!(
        parse_epub(Cursor::new(&bytes)),
        Err(BookError::ArchiveLimit)
    ));
    let mut emitted = 0;
    let result = parse_epub_progressive(Cursor::new(bytes), |_, _| {}, |_, _| emitted += 1, |_| {});
    assert!(matches!(result, Err(BookError::ArchiveLimit)));
    assert_eq!(emitted, 0);
}

fn repeated_spine(count: usize, chapter: &str) -> Vec<u8> {
    use std::io::Read;
    let mut original = zip::ZipArchive::new(Cursor::new(epub_with_chapter(chapter))).unwrap();
    let mut output = zip::ZipWriter::new(Cursor::new(Vec::new()));
    for index in 0..original.len() {
        let mut entry = original.by_index(index).unwrap();
        let name = entry.name().unwrap().into_owned();
        let mut content = Vec::new();
        entry.read_to_end(&mut content).unwrap();
        if name == "OEBPS/package.opf" {
            let package = String::from_utf8(content).unwrap();
            let start = package.find("<spine>").unwrap() + "<spine>".len();
            let end = package.find("</spine>").unwrap();
            content = format!(
                "{}{}{}",
                &package[..start],
                "<itemref idref=\"c1\"/>".repeat(count),
                &package[end..]
            )
            .into_bytes();
        }
        output
            .start_file(
                &name,
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored),
            )
            .unwrap();
        output.write_all(&content).unwrap();
    }
    output.finish().unwrap().into_inner()
}

fn epub_with_extra_file(path: &str, content: &[u8]) -> Vec<u8> {
    let mut archive = fixture_archive(CHAPTER_ONE, None);
    archive
        .start_file(path, zip::write::SimpleFileOptions::default())
        .unwrap();
    archive.write_all(content).unwrap();
    archive.finish().unwrap().into_inner()
}

fn minimal_epub() -> Vec<u8> {
    fixture_archive(CHAPTER_ONE, None)
        .finish()
        .unwrap()
        .into_inner()
}

fn epub_with_chapter(chapter: &str) -> Vec<u8> {
    fixture_archive(chapter, None)
        .finish()
        .unwrap()
        .into_inner()
}

fn epub_with_cover(cover: &[u8]) -> Vec<u8> {
    fixture_archive(CHAPTER_ONE, Some(cover))
        .finish()
        .unwrap()
        .into_inner()
}

fn fixture_archive(chapter_one: &str, cover: Option<&[u8]>) -> zip::ZipWriter<Cursor<Vec<u8>>> {
    use std::io::Write;

    let mut archive = zip::ZipWriter::new(Cursor::new(Vec::new()));
    archive
        .start_file(
            "mimetype",
            zip::write::SimpleFileOptions::default()
                .compression_method(zip::CompressionMethod::Stored),
        )
        .unwrap();
    archive.write_all(b"application/epub+zip").unwrap();
    archive
        .start_file(
            "META-INF/container.xml",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
    archive.write_all(CONTAINER_XML.as_bytes()).unwrap();
    archive
        .start_file(
            "OEBPS/package.opf",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
    let package = if cover.is_some() {
        PACKAGE_OPF.replace(
            "</metadata><manifest>",
            "<meta name=\"cover\" content=\"cover-id\"/></metadata><manifest><item id=\"cover-id\" href=\"cover.png\" media-type=\"image/png\"/>",
        )
    } else {
        PACKAGE_OPF.to_owned()
    };
    archive.write_all(package.as_bytes()).unwrap();
    archive
        .start_file("OEBPS/nav.xhtml", zip::write::SimpleFileOptions::default())
        .unwrap();
    archive
        .write_all(b"<html><body><nav><ol><li><a href='chapter-01.xhtml'>Opening the book</a></li><li><a href='chapter-02.xhtml'>A quiet ending</a></li></ol></nav></body></html>")
        .unwrap();
    let chapter_options = zip::write::SimpleFileOptions::default()
        .compression_method(zip::CompressionMethod::Deflated);
    archive
        .start_file("OEBPS/chapter-01.xhtml", chapter_options)
        .unwrap();
    archive.write_all(chapter_one.as_bytes()).unwrap();
    archive
        .start_file(
            "OEBPS/chapter-02.xhtml",
            zip::write::SimpleFileOptions::default(),
        )
        .unwrap();
    archive.write_all(CHAPTER_TWO.as_bytes()).unwrap();
    if let Some(cover) = cover {
        archive
            .start_file("OEBPS/cover.png", chapter_options)
            .unwrap();
        archive.write_all(cover).unwrap();
    }
    archive
}

use std::io::Write;

const CONTAINER_XML: &str = r#"<?xml version="1.0"?><container xmlns="urn:oasis:names:tc:opendocument:xmlns:container" version="1.0"><rootfiles><rootfile full-path="OEBPS/package.opf" media-type="application/oebps-package+xml"/></rootfiles></container>"#;
const PACKAGE_OPF: &str = r#"<?xml version="1.0"?><package xmlns="http://www.idpf.org/2007/opf" version="3.0" unique-identifier="book-id"><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:identifier id="book-id">urn:uuid:test-book</dc:identifier><dc:title>The Local Reader</dc:title><dc:creator>Ada Example</dc:creator><dc:language>en</dc:language><dc:subject>Fiction</dc:subject><meta name="calibre:series" content="Local Examples"/></metadata><manifest><item id="nav" href="nav.xhtml" media-type="application/xhtml+xml" properties="nav"/><item id="c1" href="chapter-01.xhtml" media-type="application/xhtml+xml"/><item id="c2" href="chapter-02.xhtml" media-type="application/xhtml+xml"/></manifest><spine><itemref idref="c1"/><itemref idref="c2"/></spine></package>"#;
const CHAPTER_ONE: &str = r#"<html><head><title>First chapter</title><style>hidden-style-marker</style><script>hidden-script-marker</script></head><body><h1>First chapter</h1><p>Hello, <em>reader</em>!</p><p>A second paragraph.</p></body></html>"#;
const CHAPTER_TWO: &str = r#"<html><head><title>Second chapter</title></head><body><h1>Second chapter</h1><p>A new chapter.</p></body></html>"#;
