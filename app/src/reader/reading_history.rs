//! Per-book reading history. Wheel events only update memory; SQLite writes are batched.
#[cfg(test)]
mod tests;
use super::*;
const PREFIX: &str = "reading-progress:";

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
#[serde(default)]
pub(super) struct Progress {
    pub percent: f32,
    pub last_read: u64,
    pub chapter: usize,
    #[serde(skip)]
    legacy: bool,
}

pub(super) fn load(data_dir: &Path, last_opened: Option<&Path>) -> HashMap<PathBuf, Progress> {
    let mut history = library::preferences_with_prefix(data_dir, PREFIX)
        .unwrap_or_default()
        .into_iter()
        .filter_map(|(key, json)| {
            let progress: Progress = serde_json::from_str(&json).ok()?;
            (progress.percent.is_finite() && (0.0..=100.0).contains(&progress.percent))
                .then(|| (PathBuf::from(key.strip_prefix(PREFIX).unwrap()), progress))
        })
        .collect::<HashMap<_, _>>();
    for (key, value) in library::preferences_with_prefix(data_dir, "chapter:").unwrap_or_default() {
        if let Ok(chapter) = value.parse() {
            history
                .entry(PathBuf::from(&key["chapter:".len()..]))
                .or_insert(Progress {
                    chapter,
                    legacy: true,
                    ..Default::default()
                });
        }
    }
    if let Some(path) = last_opened {
        let previous = history.entry(path.to_owned()).or_default();
        previous.last_read = previous.last_read.max(1);
    }
    history
}

pub(super) fn restore_legacy(state: &mut Reader) {
    for item in &state.library {
        let Some(saved) = state
            .reading_history
            .get_mut(&item.path)
            .filter(|p| p.legacy)
        else {
            continue;
        };
        let total: usize = item.book.chapters.iter().map(|c| c.text.len()).sum();
        let before: usize = item
            .book
            .chapters
            .iter()
            .take(
                saved
                    .chapter
                    .min(item.book.chapters.len().saturating_sub(1)),
            )
            .map(|c| c.text.len())
            .sum();
        // Older versions saved only the chapter; its start is the best recoverable estimate.
        saved.percent = if total == 0 {
            0.0
        } else {
            before as f32 / total as f32 * 100.0
        };
        saved.legacy = false;
    }
}

pub(super) fn percent(state: &Reader, path: &Path) -> f32 {
    state.reading_history.get(path).map_or(0.0, |p| p.percent)
}

pub(super) fn display_percent(value: f32) -> u8 {
    value.clamp(0.0, 100.0).floor() as u8
}

pub(super) fn in_progress(state: &Reader, path: &Path) -> bool {
    state
        .reading_history
        .get(path)
        .is_some_and(|p| p.percent < 100.0)
}

pub(super) fn started(state: &mut Reader, path: &Path) -> Task<Message> {
    let now = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
        .min(u128::from(u64::MAX)) as u64;
    let next = state
        .reading_history
        .values()
        .map(|p| p.last_read)
        .max()
        .unwrap_or(0)
        .saturating_add(1);
    state
        .reading_history
        .entry(path.to_owned())
        .or_default()
        .last_read = now.max(next);
    state.reading_dirty = false;
    persist(state, path)
}

pub(super) fn remember(state: &mut Reader, chapter: usize, position: f32, maximum: f32) {
    if state.page != Page::Reader
        || state.book_loading
        || !position.is_finite()
        || !maximum.is_finite()
        || maximum < 0.0
        || state.book.as_ref().is_none_or(|b| b.chapters.is_empty())
    {
        return;
    }
    let Some(path) = &state.book_path else {
        return;
    };
    let value = if maximum == 0.0 {
        100.0
    } else {
        (position / maximum * 100.0).clamp(0.0, 100.0)
    };
    let saved = state.reading_history.entry(path.clone()).or_default();
    // Keep the furthest point reached: revisiting an earlier passage never erases reading.
    if value > saved.percent || chapter != saved.chapter {
        saved.percent = saved.percent.max(value);
        saved.chapter = chapter;
        state.reading_dirty = true;
    }
}

fn persist(state: &Reader, path: &Path) -> Task<Message> {
    let Some(progress) = state.reading_history.get(path) else {
        return Task::none();
    };
    queue_preference(
        state,
        format!("{PREFIX}{}", path.display()),
        PreferenceValue::ReadingProgress(progress.clone()),
    )
}

pub(super) fn flush(state: &mut Reader) -> Task<Message> {
    if !state.reading_dirty {
        return Task::none();
    }
    state.reading_dirty = false;
    state
        .book_path
        .as_ref()
        .map_or_else(Task::none, |path| persist(state, path))
}

pub(super) fn close(state: &mut Reader, id: iced::window::Id) -> Task<Message> {
    state.closing = true;
    state.reading_dirty = false;
    state.settings.auto_scroll = false;
    // Queue snapshots synchronously, then drain every key before closing the window.
    let _ = persist_annotations(state);
    if let Some(path) = state
        .book_path
        .as_ref()
        .filter(|p| state.reading_history.contains_key(*p))
    {
        let _ = persist(state, path);
    }
    preferences::flush(state).then(move |message| match message {
        Message::PreferencesSaved(result) => Task::done(Message::CloseFlushed(id, result)),
        failure => Task::done(failure),
    })
}

pub(super) fn view<'a>(state: &'a Reader, path: &Path) -> Element<'a, Message> {
    use iced::widget::progress_bar;
    let value = percent(state, path);
    let accent = theme_definition_for(state).map_or(ACCENT, |theme| hex_color(&theme.accent));
    container(
        row![
            progress_bar(0.0..=100.0, value)
                .length(Length::Fill)
                .girth(4)
                .style(move |_| progress_bar::Style {
                    background: mix(surface_for(state), foreground_for(state), 0.16).into(),
                    bar: ensure_contrast(accent, surface_for(state), 3.0).into(),
                    border: iced::Border::default().rounded(2),
                }),
            text(format!("{}%", display_percent(value)))
                .size(12)
                .color(muted_for(state)),
        ]
        .spacing(8)
        .align_y(Alignment::Center),
    )
    .id(iced::widget::Id::from(format!(
        "book-progress-{}",
        path.display()
    )))
    .height(18)
    .width(Length::Fill)
    .into()
}
