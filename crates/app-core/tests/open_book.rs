use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

use app_core::{BookReader, OpenBook};
use epub_core::{Book, BookMetadata};

struct FakeReader<'a>(&'a AtomicUsize);

impl BookReader for FakeReader<'_> {
    fn read(&self, _path: &Path) -> Result<Book, epub_core::BookError> {
        self.0.fetch_add(1, Ordering::Relaxed);
        Ok(Book {
            metadata: BookMetadata::default(),
            chapters: Vec::new(),
            cover: None,
        })
    }
}

#[test]
fn opening_rejects_other_formats_before_reaching_the_reader() {
    let read_count = AtomicUsize::new(0);
    let error = OpenBook::new(FakeReader(&read_count))
        .execute(Path::new("book.pdf"))
        .unwrap_err();

    assert!(matches!(error, app_core::OpenBookError::UnsupportedFormat));
    assert_eq!(read_count.load(Ordering::Relaxed), 0);
}

#[test]
fn opening_accepts_epub_extension_without_case_sensitivity() {
    let read_count = AtomicUsize::new(0);
    OpenBook::new(FakeReader(&read_count))
        .execute(Path::new("book.EPUB"))
        .unwrap();

    assert_eq!(read_count.load(Ordering::Relaxed), 1);
}
