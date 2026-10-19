use std::path::PathBuf;

use storage::LibraryStore;

#[test]
fn persists_and_deduplicates_library_folders() {
    let data_dir = std::env::temp_dir().join(format!(
        "tournepage-storage-test-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&data_dir).unwrap();
    let database = data_dir.join("tournepage.sqlite3");
    let store = LibraryStore::open(&data_dir).unwrap();
    let folder = PathBuf::from("/books/example");
    store.add_folder(&folder).unwrap();
    store.add_folder(&folder).unwrap();
    store.set_preference("reader", "{\"size\":20}").unwrap();

    assert_eq!(store.folders().unwrap(), [folder]);
    assert_eq!(
        store.preference("reader").unwrap().as_deref(),
        Some("{\"size\":20}")
    );
    drop(store);
    std::fs::remove_file(&database).unwrap();
    std::fs::remove_file(database.with_extension("sqlite3-wal")).ok();
    std::fs::remove_file(database.with_extension("sqlite3-shm")).ok();
    std::fs::remove_dir(data_dir).unwrap();
}

#[test]
fn reads_only_the_requested_preference_prefix_and_preserves_other_data() {
    let dir = std::env::temp_dir().join(format!(
        "tourne-la-page-reviews-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let store = LibraryStore::open(&dir).unwrap();
    store
        .set_preference("book-review:/livros/a.epub", "nota-a")
        .unwrap();
    store
        .set_preference("book-review:/livros/b.epub", "nota-b")
        .unwrap();
    store
        .set_preference("book-reviews-other", "preservado")
        .unwrap();
    assert_eq!(
        store.preferences_with_prefix("book-review:").unwrap().len(),
        2
    );
    assert_eq!(
        store.preference("book-reviews-other").unwrap().as_deref(),
        Some("preservado")
    );
    drop(store);
    std::fs::remove_dir_all(&dir).unwrap();
}

#[test]
fn reopening_schema_one_preserves_existing_folders_and_preferences() {
    let dir = std::env::temp_dir().join(format!(
        "tourne-la-page-reopen-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    let folder = PathBuf::from("/books/already-imported");
    {
        let previous = LibraryStore::open(&dir).unwrap();
        previous.add_folder(&folder).unwrap();
        previous
            .set_preference("annotations:/book.epub", "saved note")
            .unwrap();
    }
    let reopened = LibraryStore::open(&dir).unwrap();
    assert_eq!(reopened.folders().unwrap(), [folder]);
    assert_eq!(
        reopened
            .preference("annotations:/book.epub")
            .unwrap()
            .as_deref(),
        Some("saved note")
    );
    drop(reopened);
    std::fs::remove_dir_all(dir).unwrap();
}
