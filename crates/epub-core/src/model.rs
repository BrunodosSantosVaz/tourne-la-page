#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct BookMetadata {
    pub title: String,
    pub authors: Vec<String>,
    pub language: Option<String>,
    pub description: Option<String>,
    pub publisher: Option<String>,
    pub date: Option<String>,
    pub subjects: Vec<String>,
    pub series: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chapter {
    pub title: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct Book {
    pub metadata: BookMetadata,
    pub chapters: Vec<Chapter>,
    pub cover: Option<Vec<u8>>,
}

#[derive(Debug, thiserror::Error, Clone, PartialEq, Eq)]
pub enum BookError {
    #[error("the file is not a valid EPUB archive")]
    InvalidArchive,
    #[error("the archive contains an unsafe path")]
    UnsafePath,
    #[error("the archive exceeds the safe reading limits")]
    ArchiveLimit,
    #[error("the book is protected by DRM")]
    Protected,
    #[error("the EPUB package document is missing")]
    MissingPackage,
    #[error("the EPUB package document is invalid")]
    InvalidPackage,
    #[error("the EPUB file could not be read")]
    SourceUnavailable,
}
