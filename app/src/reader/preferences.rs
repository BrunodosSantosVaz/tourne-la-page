//! Versioned snapshots survive dropped UI tasks and are drained explicitly on shutdown.
use super::*;
pub(super) type Pending = Arc<Mutex<HashMap<String, (u64, Arc<PreferenceValue>)>>>;

pub(super) fn queue(state: &Reader, key: String, value: PreferenceValue) -> Task<Message> {
    let version = state
        .preference_write_version
        .fetch_add(1, Ordering::SeqCst)
        + 1;
    match state.preference_versions.lock() {
        Ok(mut pending) => {
            pending.insert(key.clone(), (version, Arc::new(value)));
        }
        Err(_) => {
            return Task::done(Message::PreferencesSaved(Err(
                "Falha na fila de gravação.".into()
            )));
        }
    }
    task(state, Some((key, version)))
}

pub(super) fn flush(state: &Reader) -> Task<Message> {
    task(state, None)
}

pub(super) fn annotations(state: &Reader, path: &Path) -> Vec<BookAnnotation> {
    let pending = state.preference_versions.lock().ok().and_then(|queue| {
        queue
            .get(&annotation_key(path))
            .map(|(_, value)| Arc::clone(value))
    });
    if let Some(value) = pending
        && let PreferenceValue::Annotations(notes) = value.as_ref()
    {
        return notes.clone();
    }
    load_annotations(&state.data_dir, path)
}

fn task(state: &Reader, only: Option<(String, u64)>) -> Task<Message> {
    let data_dir = state.data_dir.clone();
    let writer = Arc::clone(&state.preference_writer);
    let pending = Arc::clone(&state.preference_versions);
    Task::perform(
        async move {
            tokio::task::spawn_blocking(move || {
                let _guard = writer
                    .lock()
                    .map_err(|_| "Falha na fila de gravação.".to_owned())?;
                write(&data_dir, &pending, only)
            })
            .await
            .map_err(|error| error.to_string())
            .and_then(|result| result)
        },
        Message::PreferencesSaved,
    )
}

fn write(data_dir: &Path, pending: &Pending, only: Option<(String, u64)>) -> Result<(), String> {
    let batch = pending
        .lock()
        .map_err(|_| "Falha na fila de gravação.".to_owned())?
        .iter()
        .filter(|(key, (version, _))| {
            only.as_ref()
                .is_none_or(|(requested, expected)| requested == *key && expected == version)
        })
        .map(|(key, (version, value))| (key.clone(), *version, Arc::clone(value)))
        .collect::<Vec<_>>();
    if batch.is_empty() {
        return Ok(());
    }
    let store = storage::LibraryStore::open(data_dir).map_err(|error| error.to_string())?;
    for (key, version, value) in batch {
        let text = match value.as_ref() {
            PreferenceValue::ReadingProgress(value) => serde_json::to_string(value),
            PreferenceValue::BookReview(value) => serde_json::to_string(value),
            PreferenceValue::Settings(value) => serde_json::to_string(value),
            PreferenceValue::Annotations(value) => serde_json::to_string(value),
            PreferenceValue::Text(value) => Ok(value.clone()),
        }
        .map_err(|error| error.to_string())?;
        store
            .set_preference(&key, &text)
            .map_err(|error| error.to_string())?;
        let mut queue = pending
            .lock()
            .map_err(|_| "Falha na fila de gravação.".to_owned())?;
        if queue
            .get(&key)
            .is_some_and(|(current, _)| *current == version)
        {
            queue.remove(&key);
        }
    }
    Ok(())
}
