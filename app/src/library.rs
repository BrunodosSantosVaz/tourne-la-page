use std::fs::{self, File};
use std::io::Cursor;
use std::path::{Path, PathBuf};
use std::sync::Arc;

use epub_core::{Book, parse_epub};
use iced::widget::image::Handle;
use image::{ImageReader, Limits};
use storage::{LibraryStore, StoreError};

const MAX_LIBRARY_BOOKS: usize = 20_000;
const MAX_DIRECTORY_DEPTH: usize = 64;
const MAX_COVER_DIMENSION: u32 = 4_096;
const MAX_COVER_ALLOCATION: u64 = 64 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct LibraryItem {
    pub path: PathBuf,
    pub book: Arc<Book>,
    pub cover: Option<Handle>,
}

pub fn saved_folders(data_dir: &Path) -> Result<Vec<PathBuf>, StoreError> {
    LibraryStore::open(data_dir)?.folders()
}

pub fn add_folder(data_dir: &Path, path: &Path) -> Result<(), StoreError> {
    LibraryStore::open(data_dir)?.add_folder(path)
}

pub fn preference(data_dir: &Path, key: &str) -> Result<Option<String>, StoreError> {
    LibraryStore::open(data_dir)?.preference(key)
}

pub fn save_preference(data_dir: &Path, key: &str, value: &str) -> Result<(), StoreError> {
    LibraryStore::open(data_dir)?.set_preference(key, value)
}

pub fn preferences_with_prefix(
    data_dir: &Path,
    prefix: &str,
) -> Result<Vec<(String, String)>, StoreError> {
    LibraryStore::open(data_dir)?.preferences_with_prefix(prefix)
}

pub fn scan_library(folders: &[PathBuf]) -> (Vec<LibraryItem>, usize) {
    let mut books = Vec::new();
    let mut skipped = 0;
    for folder in folders {
        scan_directory(folder, 0, &mut books, &mut skipped);
    }
    books.sort_by_cached_key(|item| item.book.metadata.title.to_lowercase());
    (books, skipped)
}

fn scan_directory(path: &Path, depth: usize, books: &mut Vec<LibraryItem>, skipped: &mut usize) {
    if depth > MAX_DIRECTORY_DEPTH || books.len() >= MAX_LIBRARY_BOOKS {
        return;
    }
    let Ok(entries) = fs::read_dir(path) else {
        *skipped += 1;
        return;
    };
    for entry in entries.flatten() {
        if books.len() >= MAX_LIBRARY_BOOKS {
            return;
        }
        let Ok(kind) = entry.file_type() else {
            *skipped += 1;
            continue;
        };
        if kind.is_dir() {
            scan_directory(&entry.path(), depth + 1, books, skipped);
        } else if kind.is_file() && is_epub(&entry.path()) {
            match read_item(entry.path()) {
                Some(item) => books.push(item),
                None => *skipped += 1,
            }
        }
    }
}

fn read_item(path: PathBuf) -> Option<LibraryItem> {
    let book = Arc::new(parse_epub(File::open(&path).ok()?).ok()?);
    let cover = book.cover.as_deref().and_then(decode_cover);
    Some(LibraryItem { path, book, cover })
}

fn is_epub(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("epub"))
}

pub fn decode_cover(bytes: &[u8]) -> Option<Handle> {
    let mut reader = ImageReader::new(Cursor::new(bytes))
        .with_guessed_format()
        .ok()?;
    let mut limits = Limits::default();
    limits.max_image_width = Some(MAX_COVER_DIMENSION);
    limits.max_image_height = Some(MAX_COVER_DIMENSION);
    limits.max_alloc = Some(MAX_COVER_ALLOCATION);
    reader.limits(limits);
    let thumbnail = reader.decode().ok()?.thumbnail(180, 270).to_rgba8();
    Some(Handle::from_rgba(
        thumbnail.width(),
        thumbnail.height(),
        thumbnail.into_raw(),
    ))
}

#[cfg(test)]
mod tests {
    use super::scan_library;

    #[test]
    fn finds_the_demo_book_in_a_folder_tree() {
        let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .parent()
            .unwrap()
            .join("examples");
        let (books, skipped) = scan_library(&[root]);

        assert_eq!(books.len(), 1);
        assert_eq!(books[0].book.metadata.title, "Uma pausa para ler");
        assert_eq!(skipped, 0);
    }
}
