use epub_core::{BookError, parse_epub};
use std::io::{Cursor, Write};

const PACKAGE: &str = r#"<package><metadata xmlns:dc="http://purl.org/dc/elements/1.1/"><dc:title>  </dc:title><dc:creator> </dc:creator><dc:language> </dc:language><dc:description>Description</dc:description><dc:publisher>Publisher</dc:publisher><dc:date>2026</dc:date></metadata><manifest><item id="c" href="chapter-01.xhtml" media-type="application/xhtml+xml"/></manifest><spine><itemref idref="c"/></spine></package>"#;

fn fixture(package: &str, extra: &[(&str, &[u8])]) -> Vec<u8> {
    let mut writer = zip::ZipWriter::new(Cursor::new(Vec::new()));
    let entries = [
        ("mimetype", b"application/epub+zip".as_slice()),
        (
            "META-INF/container.xml",
            b"<container><rootfile full-path='OEBPS/content.opf'/></container>".as_slice(),
        ),
        ("OEBPS/content.opf", package.as_bytes()),
        (
            "OEBPS/chapter-01.xhtml",
            b"<html><body><p>Reading text</p></body></html>".as_slice(),
        ),
    ];
    for (name, bytes) in entries.into_iter().chain(extra.iter().copied()) {
        writer
            .start_file(
                name,
                zip::write::SimpleFileOptions::default()
                    .compression_method(zip::CompressionMethod::Stored),
            )
            .unwrap();
        writer.write_all(bytes).unwrap();
    }
    writer.finish().unwrap().into_inner()
}

#[test]
fn empty_optional_metadata_does_not_replace_fallbacks_with_blank_values() {
    let book = parse_epub(Cursor::new(fixture(PACKAGE, &[]))).unwrap();
    assert_eq!(book.metadata.title, "Untitled");
    assert!(book.metadata.authors.is_empty());
    assert!(book.metadata.language.is_none());
    assert_eq!(book.metadata.description.as_deref(), Some("Description"));
    assert_eq!(book.metadata.publisher.as_deref(), Some("Publisher"));
    assert_eq!(book.metadata.date.as_deref(), Some("2026"));
    assert_eq!(book.chapters[0].title, "Capítulo 01");
}

#[test]
fn legacy_ncx_resolves_encoded_links_and_fragments_to_human_chapter_names() {
    let package = PACKAGE.replace(
        "</manifest>",
        "<item id='toc' href='toc.ncx' media-type='application/x-dtbncx+xml'/></manifest>",
    );
    let ncx = b"<ncx><navMap><navPoint><navLabel><text> A real chapter </text></navLabel><content src='chapter%2D01.xhtml#start'/></navPoint></navMap></ncx>";
    let book = parse_epub(Cursor::new(fixture(&package, &[("OEBPS/toc.ncx", ncx)]))).unwrap();
    assert_eq!(book.chapters[0].title, "A real chapter");
}

#[test]
fn font_obfuscation_is_allowed_but_content_encryption_is_rejected() {
    for (algorithm, allowed) in [
        ("http://www.idpf.org/2008/embedding", true),
        ("http://ns.adobe.com/pdf/enc#RC", true),
        ("http://www.w3.org/2001/04/xmlenc#aes256-cbc", false),
    ] {
        let xml = format!("<encryption><EncryptionMethod Algorithm='{algorithm}'/></encryption>");
        let result = parse_epub(Cursor::new(fixture(
            PACKAGE,
            &[("META-INF/encryption.xml", xml.as_bytes())],
        )));
        if allowed {
            assert!(result.is_ok());
        } else {
            assert!(matches!(result, Err(BookError::Protected)));
        }
    }
    for path in ["META-INF/license.lcpl", "META-INF/rights.xml"] {
        assert!(matches!(
            parse_epub(Cursor::new(fixture(PACKAGE, &[(path, b"protected")]))),
            Err(BookError::Protected)
        ));
    }
}

#[test]
fn encoded_and_external_chapter_references_never_escape_the_archive() {
    for href in [
        "../../outside.xhtml",
        "%2e%2e/%2e%2e/outside.xhtml",
        "/absolute.xhtml",
        "https://example.com/book.xhtml",
        "%FF.xhtml",
        "folder%5Coutside.xhtml",
    ] {
        let package = PACKAGE.replace("chapter-01.xhtml", href);
        assert!(
            matches!(
                parse_epub(Cursor::new(fixture(&package, &[]))),
                Err(BookError::UnsafePath)
            ),
            "{href}"
        );
    }
}

#[test]
fn invalid_manifest_spine_and_xml_are_domain_errors() {
    for package in [
        "<broken",
        "<package/>",
        &PACKAGE.replace("idref=\"c\"", "idref=\"missing\""),
        &PACKAGE.replace("<spine><itemref idref=\"c\"/></spine>", "<spine/>"),
        &PACKAGE.replace("application/xhtml+xml", "image/png"),
    ] {
        assert!(matches!(
            parse_epub(Cursor::new(fixture(package, &[]))),
            Err(BookError::InvalidPackage)
        ));
    }
}

#[test]
fn normalized_duplicate_entries_are_rejected_before_reading() {
    assert!(matches!(
        parse_epub(Cursor::new(fixture(
            PACKAGE,
            &[("OEBPS/./chapter-01.xhtml", b"shadowed")]
        ))),
        Err(BookError::InvalidArchive)
    ));
}
