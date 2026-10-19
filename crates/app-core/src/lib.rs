use std::path::Path;

use epub_core::{Book, BookError};

pub trait BookReader {
    fn read(&self, path: &Path) -> Result<Book, BookError>;
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum OpenBookError {
    #[error("only EPUB books can be opened")]
    UnsupportedFormat,
    #[error(transparent)]
    Book(BookError),
}

pub struct OpenBook<R> {
    reader: R,
}

impl<R: BookReader> OpenBook<R> {
    pub fn new(reader: R) -> Self {
        Self { reader }
    }

    pub fn execute(&self, path: &Path) -> Result<Book, OpenBookError> {
        let is_epub = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("epub"));
        if !is_epub {
            return Err(OpenBookError::UnsupportedFormat);
        }
        self.reader.read(path).map_err(OpenBookError::Book)
    }
}
