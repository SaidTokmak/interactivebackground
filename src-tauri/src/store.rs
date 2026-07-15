use std::{
    collections::HashSet,
    path::Path,
    sync::Mutex,
    time::{SystemTime, UNIX_EPOCH},
};

use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::{
    model::{Task, TaskStatus},
    settings::{
        AppSettings, BackgroundFit, BackgroundPreset, BackgroundSettings, BackgroundSource,
        DesktopWidget, LanguagePreference, PomodoroAction, PomodoroMode, PomodoroState,
        ThemePreference, WallpaperTemplate, WidgetKind, WidgetLayout,
    },
};

const PRIMARY_BACKGROUND_KEY: &str = "__primary__";

/// SQLite bağlantısını Tauri'nin global state sistemi içinde tutar.
///
/// `Connection` aynı anda birden fazla komut tarafından kullanılmamalıdır.
/// `Mutex`, bağlantıya erişimi sıraya koyar ve `AppStore` tipini thread-safe yapar.
pub struct AppStore {
    connection: Mutex<Connection>,
}

impl AppStore {
    pub fn open(path: impl AsRef<Path>) -> Result<Self, String> {
        let connection = Connection::open(path).map_err(database_error)?;
        Self::from_connection(connection)
    }

    #[cfg(test)]
    fn in_memory() -> Result<Self, String> {
        let connection = Connection::open_in_memory().map_err(database_error)?;
        Self::from_connection(connection)
    }

    fn from_connection(mut connection: Connection) -> Result<Self, String> {
        connection
            .execute_batch(
                "PRAGMA foreign_keys = ON;
                 PRAGMA journal_mode = WAL;

                 CREATE TABLE IF NOT EXISTS tasks (
                     id            INTEGER PRIMARY KEY AUTOINCREMENT,
                     title         TEXT NOT NULL,
                     status        TEXT NOT NULL CHECK (status IN ('todo', 'in_progress', 'done')),
                     scheduled_for TEXT,
                     created_at    TEXT NOT NULL DEFAULT CURRENT_TIMESTAMP
                 );

                 CREATE TABLE IF NOT EXISTS app_settings (
                     id        INTEGER PRIMARY KEY CHECK (id = 1),
                     template  TEXT NOT NULL CHECK (template IN ('focus', 'kanban')),
                     opacity   INTEGER NOT NULL CHECK (opacity BETWEEN 40 AND 100),
                     edit_mode INTEGER NOT NULL CHECK (edit_mode IN (0, 1)),
                     monitor_id TEXT,
                     auto_calm_minutes INTEGER DEFAULT 5,
                     theme_mode TEXT NOT NULL DEFAULT 'system'
                         CHECK (theme_mode IN ('system', 'light', 'dark')),
                     language TEXT NOT NULL DEFAULT 'system'
                         CHECK (language IN ('system', 'tr', 'en'))
                 );

                 CREATE TABLE IF NOT EXISTS monitor_backgrounds (
                     monitor_key TEXT PRIMARY KEY,
                     source TEXT NOT NULL DEFAULT 'preset'
                         CHECK (source IN ('preset', 'custom')),
                     preset TEXT NOT NULL DEFAULT 'folded_horizon'
                         CHECK (preset IN ('folded_horizon', 'midnight', 'graphite', 'ember')),
                     custom_path TEXT,
                     fit TEXT NOT NULL DEFAULT 'cover'
                         CHECK (fit IN ('cover', 'contain', 'stretch')),
                     overlay INTEGER NOT NULL DEFAULT 16 CHECK (overlay BETWEEN 0 AND 70),
                     blur INTEGER NOT NULL DEFAULT 0 CHECK (blur BETWEEN 0 AND 24)
                 );

                 CREATE TABLE IF NOT EXISTS widget_layouts (
                     monitor_key TEXT NOT NULL,
                     template TEXT NOT NULL CHECK (template IN ('focus', 'kanban')),
                     x REAL NOT NULL,
                     y REAL NOT NULL,
                     width REAL NOT NULL,
                     height REAL NOT NULL,
                     locked INTEGER NOT NULL DEFAULT 0 CHECK (locked IN (0, 1)),
                     snap_to_grid INTEGER NOT NULL DEFAULT 1 CHECK (snap_to_grid IN (0, 1)),
                     PRIMARY KEY (monitor_key, template)
                 );

                 CREATE TABLE IF NOT EXISTS desktop_widgets (
                     id INTEGER PRIMARY KEY AUTOINCREMENT,
                     monitor_key TEXT NOT NULL,
                     kind TEXT NOT NULL CHECK (kind IN ('focus', 'kanban', 'pomodoro', 'clock', 'date')),
                     x REAL NOT NULL,
                     y REAL NOT NULL,
                     width REAL NOT NULL,
                     height REAL NOT NULL,
                     locked INTEGER NOT NULL DEFAULT 0 CHECK (locked IN (0, 1)),
                     snap_to_grid INTEGER NOT NULL DEFAULT 1 CHECK (snap_to_grid IN (0, 1)),
                     visible INTEGER NOT NULL DEFAULT 1 CHECK (visible IN (0, 1)),
                     sort_order INTEGER NOT NULL DEFAULT 0
                 );

                 CREATE INDEX IF NOT EXISTS desktop_widgets_monitor_order
                 ON desktop_widgets (monitor_key, sort_order, id);

                 CREATE TABLE IF NOT EXISTS pomodoro_states (
                     widget_id INTEGER PRIMARY KEY,
                     mode TEXT NOT NULL DEFAULT 'work' CHECK (mode IN ('work', 'break')),
                     work_minutes INTEGER NOT NULL DEFAULT 25 CHECK (work_minutes BETWEEN 1 AND 180),
                     break_minutes INTEGER NOT NULL DEFAULT 5 CHECK (break_minutes BETWEEN 1 AND 60),
                     remaining_seconds INTEGER NOT NULL DEFAULT 1500,
                     running INTEGER NOT NULL DEFAULT 0 CHECK (running IN (0, 1)),
                     ends_at INTEGER,
                     FOREIGN KEY (widget_id) REFERENCES desktop_widgets(id) ON DELETE CASCADE
                 );

                 CREATE TABLE IF NOT EXISTS app_meta (
                     key TEXT PRIMARY KEY,
                     value TEXT NOT NULL
                 );

                 INSERT OR IGNORE INTO app_settings (id, template, opacity, edit_mode)
                 VALUES (1, 'focus', 82, 0);",
            )
            .map_err(database_error)?;

        // `CREATE TABLE IF NOT EXISTS` mevcut tabloya yeni sütun eklemez. Eski
        // Eski veritabanlarını ileri taşımak için sütunu ayrıca kontrol ederiz.
        ensure_monitor_column(&connection)?;
        ensure_auto_calm_column(&connection)?;
        ensure_theme_column(&connection)?;
        ensure_language_column(&connection)?;
        migrate_widget_layouts(&mut connection)?;

        let store = Self {
            connection: Mutex::new(connection),
        };
        store.seed_if_empty()?;
        Ok(store)
    }

    fn seed_if_empty(&self) -> Result<(), String> {
        let mut connection = self.lock_connection()?;
        let task_count: i64 = connection
            .query_row("SELECT COUNT(*) FROM tasks", [], |row| row.get(0))
            .map_err(database_error)?;

        if task_count > 0 {
            return Ok(());
        }

        // Dört örnek görev tek bir transaction içinde eklenir. Eklemenin herhangi
        // biri başarısız olursa transaction drop edilir ve tamamı geri alınır.
        let transaction = connection.transaction().map_err(database_error)?;
        let seed_tasks = [
            ("Rust ownership notlarını bitir", "done", "09:30"),
            ("Wallpaper pencere prototipi", "done", "11:00"),
            ("SQLite görev modelini kur", "in_progress", "14:00"),
            ("30 dk yürüyüş", "todo", "18:30"),
        ];

        for (title, status, scheduled_for) in seed_tasks {
            transaction
                .execute(
                    "INSERT INTO tasks (title, status, scheduled_for) VALUES (?1, ?2, ?3)",
                    params![title, status, scheduled_for],
                )
                .map_err(database_error)?;
        }

        transaction.commit().map_err(database_error)
    }

    pub fn list(&self) -> Result<Vec<Task>, String> {
        let connection = self.lock_connection()?;
        let mut statement = connection
            .prepare(
                "SELECT id, title, status, scheduled_for
                 FROM tasks
                 ORDER BY id ASC",
            )
            .map_err(database_error)?;

        let task_rows = statement
            .query_map([], task_from_row)
            .map_err(database_error)?;

        task_rows.map(|row| row.map_err(database_error)).collect()
    }

    pub fn create(&self, title: String, scheduled_for: Option<String>) -> Result<Task, String> {
        // Önce domain doğrulamasını çalıştırıyoruz. Henüz id oluşmadığı için geçici
        // olarak 0 verilir; gerçek id INSERT işleminden sonra SQLite'tan alınır.
        let mut task = Task::new(0, title, normalize_time(scheduled_for))?;
        let connection = self.lock_connection()?;
        connection
            .execute(
                "INSERT INTO tasks (title, status, scheduled_for) VALUES (?1, ?2, ?3)",
                params![
                    task.title,
                    task.status.as_database_value(),
                    task.scheduled_for
                ],
            )
            .map_err(database_error)?;
        task.id = connection.last_insert_rowid();
        Ok(task)
    }

    pub fn toggle(&self, id: i64) -> Result<Task, String> {
        let connection = self.lock_connection()?;
        let mut task = find_task(&connection, id)?;
        task.status = if task.status == TaskStatus::Done {
            TaskStatus::Todo
        } else {
            TaskStatus::Done
        };

        connection
            .execute(
                "UPDATE tasks SET status = ?1 WHERE id = ?2",
                params![task.status.as_database_value(), id],
            )
            .map_err(database_error)?;
        Ok(task)
    }

    pub fn move_to(&self, id: i64, status: TaskStatus) -> Result<Task, String> {
        let connection = self.lock_connection()?;
        let changed_rows = connection
            .execute(
                "UPDATE tasks SET status = ?1 WHERE id = ?2",
                params![status.as_database_value(), id],
            )
            .map_err(database_error)?;

        if changed_rows == 0 {
            return Err(task_not_found(id));
        }

        find_task(&connection, id)
    }

    pub fn delete(&self, id: i64) -> Result<(), String> {
        let connection = self.lock_connection()?;
        let changed_rows = connection
            .execute("DELETE FROM tasks WHERE id = ?1", [id])
            .map_err(database_error)?;

        if changed_rows == 0 {
            return Err(task_not_found(id));
        }

        Ok(())
    }

    pub fn get_settings(&self) -> Result<AppSettings, String> {
        let connection = self.lock_connection()?;
        connection
            .query_row(
                "SELECT template, opacity, edit_mode, monitor_id, auto_calm_minutes, theme_mode,
                        language
                 FROM app_settings WHERE id = 1",
                [],
                settings_from_row,
            )
            .map_err(database_error)
    }

    pub fn update_settings(&self, settings: AppSettings) -> Result<AppSettings, String> {
        let settings = settings.validate()?;
        let connection = self.lock_connection()?;
        connection
            .execute(
                "UPDATE app_settings
                 SET template = ?1, opacity = ?2, edit_mode = ?3, monitor_id = ?4,
                     auto_calm_minutes = ?5, theme_mode = ?6, language = ?7
                 WHERE id = 1",
                params![
                    settings.template.as_database_value(),
                    i64::from(settings.opacity),
                    settings.edit_mode,
                    settings.monitor_id,
                    settings.auto_calm_minutes,
                    settings.theme.as_database_value(),
                    settings.language.as_database_value(),
                ],
            )
            .map_err(database_error)?;
        Ok(settings)
    }

    pub fn get_background_settings(
        &self,
        monitor_id: Option<String>,
    ) -> Result<BackgroundSettings, String> {
        let connection = self.lock_connection()?;
        let monitor_key = background_monitor_key(monitor_id.as_deref());
        connection
            .query_row(
                "SELECT source, preset, custom_path, fit, overlay, blur
                 FROM monitor_backgrounds WHERE monitor_key = ?1",
                [monitor_key],
                |row| background_settings_from_row(row, monitor_id.clone()),
            )
            .optional()
            .map_err(database_error)
            .map(|settings| {
                settings.unwrap_or_else(|| BackgroundSettings::defaults_for(monitor_id))
            })
    }

    pub fn update_background_settings(
        &self,
        settings: BackgroundSettings,
    ) -> Result<BackgroundSettings, String> {
        let settings = settings.validate()?;
        let connection = self.lock_connection()?;
        let monitor_key = background_monitor_key(settings.monitor_id.as_deref());
        connection
            .execute(
                "INSERT INTO monitor_backgrounds
                    (monitor_key, source, preset, custom_path, fit, overlay, blur)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)
                 ON CONFLICT(monitor_key) DO UPDATE SET
                    source = excluded.source,
                    preset = excluded.preset,
                    custom_path = excluded.custom_path,
                    fit = excluded.fit,
                    overlay = excluded.overlay,
                    blur = excluded.blur",
                params![
                    monitor_key,
                    settings.source.as_database_value(),
                    settings.preset.as_database_value(),
                    settings.custom_path,
                    settings.fit.as_database_value(),
                    settings.overlay,
                    settings.blur,
                ],
            )
            .map_err(database_error)?;
        Ok(settings)
    }

    pub fn get_widget_layout(
        &self,
        monitor_id: Option<String>,
        template: WallpaperTemplate,
    ) -> Result<WidgetLayout, String> {
        let connection = self.lock_connection()?;
        let monitor_key = background_monitor_key(monitor_id.as_deref());
        connection
            .query_row(
                "SELECT x, y, width, height, locked, snap_to_grid
                 FROM widget_layouts WHERE monitor_key = ?1 AND template = ?2",
                params![monitor_key, template.as_database_value()],
                |row| widget_layout_from_row(row, monitor_id.clone(), template),
            )
            .optional()
            .map_err(database_error)
            .map(|layout| {
                layout.unwrap_or_else(|| WidgetLayout::defaults_for(monitor_id, template))
            })
    }

    pub fn update_widget_layout(&self, layout: WidgetLayout) -> Result<WidgetLayout, String> {
        let layout = layout.validate()?;
        let connection = self.lock_connection()?;
        let monitor_key = background_monitor_key(layout.monitor_id.as_deref());
        connection
            .execute(
                "INSERT INTO widget_layouts
                    (monitor_key, template, x, y, width, height, locked, snap_to_grid)
                 VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)
                 ON CONFLICT(monitor_key, template) DO UPDATE SET
                    x = excluded.x,
                    y = excluded.y,
                    width = excluded.width,
                    height = excluded.height,
                    locked = excluded.locked,
                    snap_to_grid = excluded.snap_to_grid",
                params![
                    monitor_key,
                    layout.template.as_database_value(),
                    layout.x,
                    layout.y,
                    layout.width,
                    layout.height,
                    layout.locked,
                    layout.snap_to_grid,
                ],
            )
            .map_err(database_error)?;
        Ok(layout)
    }

    pub fn reset_widget_layout(
        &self,
        monitor_id: Option<String>,
        template: WallpaperTemplate,
    ) -> Result<WidgetLayout, String> {
        let connection = self.lock_connection()?;
        connection
            .execute(
                "DELETE FROM widget_layouts WHERE monitor_key = ?1 AND template = ?2",
                params![
                    background_monitor_key(monitor_id.as_deref()),
                    template.as_database_value()
                ],
            )
            .map_err(database_error)?;
        Ok(WidgetLayout::defaults_for(monitor_id, template))
    }

    pub fn list_desktop_widgets(
        &self,
        monitor_id: Option<String>,
    ) -> Result<Vec<DesktopWidget>, String> {
        let connection = self.lock_connection()?;
        let monitor_key = background_monitor_key(monitor_id.as_deref());
        let mut statement = connection
            .prepare(
                "SELECT id, kind, x, y, width, height, locked, snap_to_grid, visible, sort_order
                 FROM desktop_widgets WHERE monitor_key = ?1
                 ORDER BY sort_order ASC, id ASC",
            )
            .map_err(database_error)?;
        let rows = statement
            .query_map([monitor_key], |row| {
                desktop_widget_from_row(row, monitor_id.clone())
            })
            .map_err(database_error)?;
        rows.map(|row| row.map_err(database_error)).collect()
    }

    pub fn add_desktop_widget(
        &self,
        monitor_id: Option<String>,
        kind: WidgetKind,
    ) -> Result<DesktopWidget, String> {
        let connection = self.lock_connection()?;
        let monitor_key = background_monitor_key(monitor_id.as_deref()).to_string();
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM desktop_widgets WHERE monitor_key = ?1",
                [&monitor_key],
                |row| row.get(0),
            )
            .map_err(database_error)?;
        if count >= 12 {
            return Err("Bir monitörde en fazla 12 widget kullanılabilir.".into());
        }
        let sort_order: i64 = connection
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM desktop_widgets WHERE monitor_key = ?1",
                [&monitor_key],
                |row| row.get(0),
            )
            .map_err(database_error)?;
        let mut widget = DesktopWidget::defaults_for(monitor_id, kind, sort_order).validate()?;
        insert_desktop_widget(&connection, &monitor_key, &widget)?;
        widget.id = connection.last_insert_rowid();
        ensure_pomodoro_state(&connection, &widget)?;
        Ok(widget)
    }

    pub fn update_desktop_widget(&self, widget: DesktopWidget) -> Result<DesktopWidget, String> {
        let widget = widget.validate()?;
        if widget.id <= 0 {
            return Err("Widget kimliği geçersiz.".into());
        }
        let connection = self.lock_connection()?;
        let changed = connection
            .execute(
                "UPDATE desktop_widgets SET
                    monitor_key = ?1, kind = ?2, x = ?3, y = ?4, width = ?5,
                    height = ?6, locked = ?7, snap_to_grid = ?8, visible = ?9,
                    sort_order = ?10
                 WHERE id = ?11",
                params![
                    background_monitor_key(widget.monitor_id.as_deref()),
                    widget.kind.as_database_value(),
                    widget.x,
                    widget.y,
                    widget.width,
                    widget.height,
                    widget.locked,
                    widget.snap_to_grid,
                    widget.visible,
                    widget.sort_order,
                    widget.id,
                ],
            )
            .map_err(database_error)?;
        if changed == 0 {
            return Err("Widget bulunamadı.".into());
        }
        ensure_pomodoro_state(&connection, &widget)?;
        Ok(widget)
    }

    pub fn duplicate_desktop_widget(&self, id: i64) -> Result<DesktopWidget, String> {
        let connection = self.lock_connection()?;
        let original = find_desktop_widget(&connection, id)?;
        let monitor_key = background_monitor_key(original.monitor_id.as_deref()).to_string();
        let count: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM desktop_widgets WHERE monitor_key = ?1",
                [&monitor_key],
                |row| row.get(0),
            )
            .map_err(database_error)?;
        if count >= 12 {
            return Err("Bir monitörde en fazla 12 widget kullanılabilir.".into());
        }
        let sort_order: i64 = connection
            .query_row(
                "SELECT COALESCE(MAX(sort_order), -1) + 1 FROM desktop_widgets WHERE monitor_key = ?1",
                [&monitor_key],
                |row| row.get(0),
            )
            .map_err(database_error)?;
        let mut duplicate = original;
        duplicate.id = 0;
        duplicate.sort_order = sort_order;
        duplicate.locked = false;
        duplicate.x = (duplicate.x + 0.025).min(0.985 - duplicate.width);
        duplicate.y = (duplicate.y + 0.025).min(0.985 - duplicate.height);
        duplicate = duplicate.validate()?;
        insert_desktop_widget(&connection, &monitor_key, &duplicate)?;
        duplicate.id = connection.last_insert_rowid();
        ensure_pomodoro_state(&connection, &duplicate)?;
        if duplicate.kind == WidgetKind::Pomodoro {
            connection
                .execute(
                    "UPDATE pomodoro_states SET
                        mode = (SELECT mode FROM pomodoro_states WHERE widget_id = ?1),
                        work_minutes = (SELECT work_minutes FROM pomodoro_states WHERE widget_id = ?1),
                        break_minutes = (SELECT break_minutes FROM pomodoro_states WHERE widget_id = ?1),
                        remaining_seconds = (SELECT remaining_seconds FROM pomodoro_states WHERE widget_id = ?1),
                        running = 0, ends_at = NULL
                     WHERE widget_id = ?2",
                    params![id, duplicate.id],
                )
                .map_err(database_error)?;
        }
        Ok(duplicate)
    }

    pub fn delete_desktop_widget(&self, id: i64) -> Result<(), String> {
        let connection = self.lock_connection()?;
        let changed = connection
            .execute("DELETE FROM desktop_widgets WHERE id = ?1", [id])
            .map_err(database_error)?;
        if changed == 0 {
            return Err("Widget bulunamadı.".into());
        }
        Ok(())
    }

    pub fn reorder_desktop_widgets(
        &self,
        monitor_id: Option<String>,
        ordered_ids: Vec<i64>,
    ) -> Result<Vec<DesktopWidget>, String> {
        let mut connection = self.lock_connection()?;
        let monitor_key = background_monitor_key(monitor_id.as_deref()).to_string();
        let expected: i64 = connection
            .query_row(
                "SELECT COUNT(*) FROM desktop_widgets WHERE monitor_key = ?1",
                [&monitor_key],
                |row| row.get(0),
            )
            .map_err(database_error)?;
        if expected != ordered_ids.len() as i64 {
            return Err("Widget sırası eksik veya geçersiz.".into());
        }
        if ordered_ids.iter().copied().collect::<HashSet<_>>().len() != ordered_ids.len() {
            return Err("Widget sırası eksik veya geçersiz.".into());
        }
        let transaction = connection.transaction().map_err(database_error)?;
        for (index, id) in ordered_ids.iter().enumerate() {
            let changed = transaction
                .execute(
                    "UPDATE desktop_widgets SET sort_order = ?1 WHERE id = ?2 AND monitor_key = ?3",
                    params![index as i64, id, &monitor_key],
                )
                .map_err(database_error)?;
            if changed == 0 {
                return Err("Widget sırası eksik veya geçersiz.".into());
            }
        }
        transaction.commit().map_err(database_error)?;
        drop(connection);
        self.list_desktop_widgets(monitor_id)
    }

    pub fn get_pomodoro_state(&self, widget_id: i64) -> Result<PomodoroState, String> {
        let connection = self.lock_connection()?;
        let widget = find_desktop_widget(&connection, widget_id)?;
        if widget.kind != WidgetKind::Pomodoro {
            return Err("Seçilen widget Pomodoro değil.".into());
        }
        ensure_pomodoro_state(&connection, &widget)?;
        normalize_pomodoro(&connection, widget_id)
    }

    pub fn update_pomodoro(
        &self,
        widget_id: i64,
        action: PomodoroAction,
    ) -> Result<PomodoroState, String> {
        let connection = self.lock_connection()?;
        let widget = find_desktop_widget(&connection, widget_id)?;
        if widget.kind != WidgetKind::Pomodoro {
            return Err("Seçilen widget Pomodoro değil.".into());
        }
        ensure_pomodoro_state(&connection, &widget)?;
        let mut state = normalize_pomodoro(&connection, widget_id)?;
        let now = unix_timestamp()?;
        match action {
            PomodoroAction::Start => {
                state.running = true;
                state.ends_at = Some(now + state.remaining_seconds.max(1));
            }
            PomodoroAction::Pause => {
                state.running = false;
                state.ends_at = None;
            }
            PomodoroAction::Reset => {
                state.mode = PomodoroMode::Work;
                state.remaining_seconds = i64::from(state.work_minutes) * 60;
                state.running = false;
                state.ends_at = None;
            }
            PomodoroAction::Skip => {
                state.mode = if state.mode == PomodoroMode::Work {
                    PomodoroMode::Break
                } else {
                    PomodoroMode::Work
                };
                state.remaining_seconds = mode_duration(&state);
                state.running = false;
                state.ends_at = None;
            }
            PomodoroAction::Complete => {}
        }
        write_pomodoro_state(&connection, &state)?;
        Ok(state)
    }

    pub fn configure_pomodoro(
        &self,
        widget_id: i64,
        work_minutes: u16,
        break_minutes: u16,
    ) -> Result<PomodoroState, String> {
        if !(1..=180).contains(&work_minutes) || !(1..=60).contains(&break_minutes) {
            return Err("Pomodoro süreleri izin verilen aralıkta değil.".into());
        }
        let connection = self.lock_connection()?;
        let widget = find_desktop_widget(&connection, widget_id)?;
        if widget.kind != WidgetKind::Pomodoro {
            return Err("Seçilen widget Pomodoro değil.".into());
        }
        ensure_pomodoro_state(&connection, &widget)?;
        let state = PomodoroState {
            widget_id,
            mode: PomodoroMode::Work,
            work_minutes,
            break_minutes,
            remaining_seconds: i64::from(work_minutes) * 60,
            running: false,
            ends_at: None,
        };
        write_pomodoro_state(&connection, &state)?;
        Ok(state)
    }

    fn lock_connection(&self) -> Result<std::sync::MutexGuard<'_, Connection>, String> {
        self.connection
            .lock()
            .map_err(|_| "Veritabanı bağlantısına erişilemedi.".into())
    }
}

fn find_task(connection: &Connection, id: i64) -> Result<Task, String> {
    connection
        .query_row(
            "SELECT id, title, status, scheduled_for FROM tasks WHERE id = ?1",
            [id],
            task_from_row,
        )
        .optional()
        .map_err(database_error)?
        .ok_or_else(|| task_not_found(id))
}

fn task_from_row(row: &Row<'_>) -> rusqlite::Result<Task> {
    let status: String = row.get(2)?;
    let status = TaskStatus::from_database_value(&status).map_err(|message| {
        rusqlite::Error::FromSqlConversionFailure(
            2,
            rusqlite::types::Type::Text,
            std::io::Error::new(std::io::ErrorKind::InvalidData, message).into(),
        )
    })?;

    Ok(Task {
        id: row.get(0)?,
        title: row.get(1)?,
        status,
        scheduled_for: row.get(3)?,
    })
}

fn settings_from_row(row: &Row<'_>) -> rusqlite::Result<AppSettings> {
    let template: String = row.get(0)?;
    let template = WallpaperTemplate::from_database_value(&template).map_err(|message| {
        rusqlite::Error::FromSqlConversionFailure(
            0,
            rusqlite::types::Type::Text,
            std::io::Error::new(std::io::ErrorKind::InvalidData, message).into(),
        )
    })?;
    let opacity: i64 = row.get(1)?;
    let opacity = u8::try_from(opacity).map_err(|error| {
        rusqlite::Error::FromSqlConversionFailure(
            1,
            rusqlite::types::Type::Integer,
            Box::new(error),
        )
    })?;

    let theme: String = row.get(5)?;
    let theme = ThemePreference::from_database_value(&theme).map_err(|message| {
        rusqlite::Error::FromSqlConversionFailure(
            5,
            rusqlite::types::Type::Text,
            std::io::Error::new(std::io::ErrorKind::InvalidData, message).into(),
        )
    })?;
    let language: String = row.get(6)?;
    let language = LanguagePreference::from_database_value(&language).map_err(|message| {
        rusqlite::Error::FromSqlConversionFailure(
            6,
            rusqlite::types::Type::Text,
            std::io::Error::new(std::io::ErrorKind::InvalidData, message).into(),
        )
    })?;

    Ok(AppSettings {
        template,
        opacity,
        edit_mode: row.get(2)?,
        monitor_id: row.get(3)?,
        auto_calm_minutes: row.get(4)?,
        theme,
        language,
    })
}

fn background_settings_from_row(
    row: &Row<'_>,
    monitor_id: Option<String>,
) -> rusqlite::Result<BackgroundSettings> {
    let source_value: String = row.get(0)?;
    let preset_value: String = row.get(1)?;
    let fit_value: String = row.get(3)?;
    let conversion_error = |index, message: String| {
        rusqlite::Error::FromSqlConversionFailure(
            index,
            rusqlite::types::Type::Text,
            std::io::Error::new(std::io::ErrorKind::InvalidData, message).into(),
        )
    };

    Ok(BackgroundSettings {
        monitor_id,
        source: BackgroundSource::from_database_value(&source_value)
            .map_err(|message| conversion_error(0, message))?,
        preset: BackgroundPreset::from_database_value(&preset_value)
            .map_err(|message| conversion_error(1, message))?,
        custom_path: row.get(2)?,
        fit: BackgroundFit::from_database_value(&fit_value)
            .map_err(|message| conversion_error(3, message))?,
        overlay: row.get(4)?,
        blur: row.get(5)?,
    })
}

fn widget_layout_from_row(
    row: &Row<'_>,
    monitor_id: Option<String>,
    template: WallpaperTemplate,
) -> rusqlite::Result<WidgetLayout> {
    Ok(WidgetLayout {
        monitor_id,
        template,
        x: row.get(0)?,
        y: row.get(1)?,
        width: row.get(2)?,
        height: row.get(3)?,
        locked: row.get(4)?,
        snap_to_grid: row.get(5)?,
    })
}

fn desktop_widget_from_row(
    row: &Row<'_>,
    monitor_id: Option<String>,
) -> rusqlite::Result<DesktopWidget> {
    let kind_value: String = row.get(1)?;
    let kind = WidgetKind::from_database_value(&kind_value).map_err(|message| {
        rusqlite::Error::FromSqlConversionFailure(
            1,
            rusqlite::types::Type::Text,
            std::io::Error::new(std::io::ErrorKind::InvalidData, message).into(),
        )
    })?;
    Ok(DesktopWidget {
        id: row.get(0)?,
        monitor_id,
        kind,
        x: row.get(2)?,
        y: row.get(3)?,
        width: row.get(4)?,
        height: row.get(5)?,
        locked: row.get(6)?,
        snap_to_grid: row.get(7)?,
        visible: row.get(8)?,
        sort_order: row.get(9)?,
    })
}

fn find_desktop_widget(connection: &Connection, id: i64) -> Result<DesktopWidget, String> {
    let (monitor_key, mut widget): (String, DesktopWidget) = connection
        .query_row(
            "SELECT monitor_key, id, kind, x, y, width, height, locked, snap_to_grid, visible, sort_order
             FROM desktop_widgets WHERE id = ?1",
            [id],
            |row| {
                let monitor_key: String = row.get(0)?;
                let monitor_id = if monitor_key == PRIMARY_BACKGROUND_KEY {
                    None
                } else {
                    Some(monitor_key.clone())
                };
                let kind_value: String = row.get(2)?;
                let kind = WidgetKind::from_database_value(&kind_value).map_err(|message| {
                    rusqlite::Error::FromSqlConversionFailure(
                        2,
                        rusqlite::types::Type::Text,
                        std::io::Error::new(std::io::ErrorKind::InvalidData, message).into(),
                    )
                })?;
                Ok((
                    monitor_key,
                    DesktopWidget {
                        id: row.get(1)?,
                        monitor_id,
                        kind,
                        x: row.get(3)?,
                        y: row.get(4)?,
                        width: row.get(5)?,
                        height: row.get(6)?,
                        locked: row.get(7)?,
                        snap_to_grid: row.get(8)?,
                        visible: row.get(9)?,
                        sort_order: row.get(10)?,
                    },
                ))
            },
        )
        .optional()
        .map_err(database_error)?
        .ok_or_else(|| "Widget bulunamadı.".to_string())?;
    widget.monitor_id = if monitor_key == PRIMARY_BACKGROUND_KEY {
        None
    } else {
        Some(monitor_key)
    };
    Ok(widget)
}

fn insert_desktop_widget(
    connection: &Connection,
    monitor_key: &str,
    widget: &DesktopWidget,
) -> Result<(), String> {
    connection
        .execute(
            "INSERT INTO desktop_widgets
                (monitor_key, kind, x, y, width, height, locked, snap_to_grid, visible, sort_order)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                monitor_key,
                widget.kind.as_database_value(),
                widget.x,
                widget.y,
                widget.width,
                widget.height,
                widget.locked,
                widget.snap_to_grid,
                widget.visible,
                widget.sort_order,
            ],
        )
        .map_err(database_error)?;
    Ok(())
}

fn ensure_pomodoro_state(connection: &Connection, widget: &DesktopWidget) -> Result<(), String> {
    if widget.kind != WidgetKind::Pomodoro {
        return Ok(());
    }
    connection
        .execute(
            "INSERT OR IGNORE INTO pomodoro_states (widget_id) VALUES (?1)",
            [widget.id],
        )
        .map_err(database_error)?;
    Ok(())
}

fn normalize_pomodoro(connection: &Connection, widget_id: i64) -> Result<PomodoroState, String> {
    let mut state = connection
        .query_row(
            "SELECT mode, work_minutes, break_minutes, remaining_seconds, running, ends_at
             FROM pomodoro_states WHERE widget_id = ?1",
            [widget_id],
            |row| {
                let mode_value: String = row.get(0)?;
                let mode = PomodoroMode::from_database_value(&mode_value).map_err(|message| {
                    rusqlite::Error::FromSqlConversionFailure(
                        0,
                        rusqlite::types::Type::Text,
                        std::io::Error::new(std::io::ErrorKind::InvalidData, message).into(),
                    )
                })?;
                Ok(PomodoroState {
                    widget_id,
                    mode,
                    work_minutes: row.get(1)?,
                    break_minutes: row.get(2)?,
                    remaining_seconds: row.get(3)?,
                    running: row.get(4)?,
                    ends_at: row.get(5)?,
                })
            },
        )
        .map_err(database_error)?;
    if state.running {
        let now = unix_timestamp()?;
        let remaining = state.ends_at.unwrap_or(now) - now;
        if remaining > 0 {
            state.remaining_seconds = remaining;
        } else {
            state.mode = if state.mode == PomodoroMode::Work {
                PomodoroMode::Break
            } else {
                PomodoroMode::Work
            };
            state.remaining_seconds = mode_duration(&state);
            state.running = false;
            state.ends_at = None;
        }
        write_pomodoro_state(connection, &state)?;
    }
    Ok(state)
}

fn write_pomodoro_state(connection: &Connection, state: &PomodoroState) -> Result<(), String> {
    connection
        .execute(
            "UPDATE pomodoro_states SET mode = ?1, work_minutes = ?2, break_minutes = ?3,
                remaining_seconds = ?4, running = ?5, ends_at = ?6
             WHERE widget_id = ?7",
            params![
                state.mode.as_database_value(),
                state.work_minutes,
                state.break_minutes,
                state.remaining_seconds,
                state.running,
                state.ends_at,
                state.widget_id,
            ],
        )
        .map_err(database_error)?;
    Ok(())
}

fn mode_duration(state: &PomodoroState) -> i64 {
    i64::from(if state.mode == PomodoroMode::Work {
        state.work_minutes
    } else {
        state.break_minutes
    }) * 60
}

fn unix_timestamp() -> Result<i64, String> {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_err(|error| format!("Sistem saati okunamadı: {error}"))?
        .as_secs();
    i64::try_from(seconds).map_err(|error| format!("Sistem saati dönüştürülemedi: {error}"))
}

fn background_monitor_key(monitor_id: Option<&str>) -> &str {
    monitor_id.unwrap_or(PRIMARY_BACKGROUND_KEY)
}

fn ensure_monitor_column(connection: &Connection) -> Result<(), String> {
    let mut statement = connection
        .prepare("PRAGMA table_info(app_settings)")
        .map_err(database_error)?;
    let column_names = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;

    if !column_names.iter().any(|name| name == "monitor_id") {
        connection
            .execute("ALTER TABLE app_settings ADD COLUMN monitor_id TEXT", [])
            .map_err(database_error)?;
    }
    Ok(())
}

fn ensure_auto_calm_column(connection: &Connection) -> Result<(), String> {
    let mut statement = connection
        .prepare("PRAGMA table_info(app_settings)")
        .map_err(database_error)?;
    let column_names = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;

    if !column_names.iter().any(|name| name == "auto_calm_minutes") {
        connection
            .execute(
                "ALTER TABLE app_settings ADD COLUMN auto_calm_minutes INTEGER DEFAULT 5",
                [],
            )
            .map_err(database_error)?;
    }
    Ok(())
}

fn ensure_theme_column(connection: &Connection) -> Result<(), String> {
    let mut statement = connection
        .prepare("PRAGMA table_info(app_settings)")
        .map_err(database_error)?;
    let column_names = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;

    if !column_names.iter().any(|name| name == "theme_mode") {
        connection
            .execute(
                "ALTER TABLE app_settings ADD COLUMN theme_mode TEXT NOT NULL DEFAULT 'system'",
                [],
            )
            .map_err(database_error)?;
    }
    Ok(())
}

fn ensure_language_column(connection: &Connection) -> Result<(), String> {
    let mut statement = connection
        .prepare("PRAGMA table_info(app_settings)")
        .map_err(database_error)?;
    let column_names = statement
        .query_map([], |row| row.get::<_, String>(1))
        .map_err(database_error)?
        .collect::<rusqlite::Result<Vec<_>>>()
        .map_err(database_error)?;

    if !column_names.iter().any(|name| name == "language") {
        connection
            .execute(
                "ALTER TABLE app_settings ADD COLUMN language TEXT NOT NULL DEFAULT 'system'",
                [],
            )
            .map_err(database_error)?;
    }
    Ok(())
}

fn migrate_widget_layouts(connection: &mut Connection) -> Result<(), String> {
    let transaction = connection.transaction().map_err(database_error)?;
    let migrated = transaction
        .query_row(
            "SELECT value FROM app_meta WHERE key = 'desktop_widgets_v1'",
            [],
            |row| row.get::<_, String>(0),
        )
        .optional()
        .map_err(database_error)?
        .is_some();
    if migrated {
        return Ok(());
    }

    transaction
        .execute(
            "INSERT INTO desktop_widgets
                (monitor_key, kind, x, y, width, height, locked, snap_to_grid, visible, sort_order)
             SELECT monitor_key, template, x, y, width, height, locked, snap_to_grid, 1,
                    ROW_NUMBER() OVER (PARTITION BY monitor_key ORDER BY template) - 1
             FROM widget_layouts",
            [],
        )
        .map_err(database_error)?;

    let count: i64 = transaction
        .query_row("SELECT COUNT(*) FROM desktop_widgets", [], |row| row.get(0))
        .map_err(database_error)?;
    if count == 0 {
        let (template, monitor_id): (String, Option<String>) = transaction
            .query_row(
                "SELECT template, monitor_id FROM app_settings WHERE id = 1",
                [],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .map_err(database_error)?;
        let kind = WidgetKind::from_database_value(&template)?;
        let widget = DesktopWidget::defaults_for(monitor_id.clone(), kind, 0).validate()?;
        insert_desktop_widget(
            &transaction,
            background_monitor_key(monitor_id.as_deref()),
            &widget,
        )?;
    }

    transaction
        .execute(
            "INSERT INTO app_meta (key, value) VALUES ('desktop_widgets_v1', 'done')",
            [],
        )
        .map_err(database_error)?;
    transaction.commit().map_err(database_error)
}

fn normalize_time(value: Option<String>) -> Option<String> {
    value.and_then(|time| {
        let trimmed = time.trim();
        (!trimmed.is_empty()).then(|| trimmed.to_owned())
    })
}

fn database_error(error: rusqlite::Error) -> String {
    format!("Veritabanı hatası: {error}")
}

fn task_not_found(id: i64) -> String {
    format!("{id} numaralı görev bulunamadı.")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    #[test]
    fn seeds_a_new_database() {
        let store = AppStore::in_memory().unwrap();

        assert_eq!(store.list().unwrap().len(), 4);
    }

    #[test]
    fn creates_and_reads_a_persistent_task() {
        let store = AppStore::in_memory().unwrap();
        let task = store.create("  Tauri öğren  ".into(), None).unwrap();
        let stored_task = store
            .list()
            .unwrap()
            .into_iter()
            .find(|item| item.id == task.id)
            .unwrap();

        assert_eq!(stored_task.title, "Tauri öğren");
        assert_eq!(stored_task.status, TaskStatus::Todo);
    }

    #[test]
    fn rejects_an_empty_title_without_writing_a_row() {
        let store = AppStore::in_memory().unwrap();
        let error = store.create("   ".into(), None).unwrap_err();

        assert_eq!(error, "Görev başlığı boş olamaz.");
        assert_eq!(store.list().unwrap().len(), 4);
    }

    #[test]
    fn toggles_moves_and_deletes_a_task() {
        let store = AppStore::in_memory().unwrap();

        assert_eq!(store.toggle(4).unwrap().status, TaskStatus::Done);
        assert_eq!(
            store.move_to(4, TaskStatus::InProgress).unwrap().status,
            TaskStatus::InProgress
        );
        store.delete(4).unwrap();
        assert_eq!(store.list().unwrap().len(), 3);
    }

    #[test]
    fn validates_and_updates_wallpaper_settings() {
        let store = AppStore::in_memory().unwrap();
        assert_eq!(store.get_settings().unwrap().opacity, 82);

        let updated = store
            .update_settings(AppSettings {
                template: WallpaperTemplate::Kanban,
                opacity: 76,
                edit_mode: true,
                monitor_id: Some("monitor:0:0:1920x1080".into()),
                auto_calm_minutes: Some(5),
                theme: ThemePreference::Dark,
                language: LanguagePreference::En,
            })
            .unwrap();
        assert_eq!(updated.template, WallpaperTemplate::Kanban);
        assert_eq!(updated.theme, ThemePreference::Dark);
        assert_eq!(updated.language, LanguagePreference::En);
        assert_eq!(store.get_settings().unwrap(), updated);

        let error = store
            .update_settings(AppSettings {
                opacity: 20,
                ..updated
            })
            .unwrap_err();
        assert_eq!(error, "Saydamlık değeri 40 ile 100 arasında olmalıdır.");
    }

    #[test]
    fn migrates_an_existing_settings_table_with_monitor_selection() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE app_settings (
                    id INTEGER PRIMARY KEY CHECK (id = 1),
                    template TEXT NOT NULL,
                    opacity INTEGER NOT NULL,
                    edit_mode INTEGER NOT NULL
                 );
                 INSERT INTO app_settings (id, template, opacity, edit_mode)
                 VALUES (1, 'focus', 82, 0);",
            )
            .unwrap();

        let store = AppStore::from_connection(connection).unwrap();
        assert_eq!(store.get_settings().unwrap().monitor_id, None);
        assert_eq!(store.get_settings().unwrap().auto_calm_minutes, Some(5));
        assert_eq!(store.get_settings().unwrap().theme, ThemePreference::System);
        assert_eq!(
            store.get_settings().unwrap().language,
            LanguagePreference::System
        );

        let updated = store
            .update_settings(AppSettings {
                template: WallpaperTemplate::Focus,
                opacity: 82,
                edit_mode: false,
                monitor_id: Some("display:0:0:2560x1440".into()),
                auto_calm_minutes: Some(10),
                theme: ThemePreference::Light,
                language: LanguagePreference::Tr,
            })
            .unwrap();
        assert_eq!(store.get_settings().unwrap().monitor_id, updated.monitor_id);
    }

    #[test]
    fn keeps_independent_background_settings_for_each_monitor() {
        let store = AppStore::in_memory().unwrap();
        let primary = store.get_background_settings(None).unwrap();
        assert_eq!(primary.preset, BackgroundPreset::FoldedHorizon);

        let display_two = BackgroundSettings {
            monitor_id: Some("display-two".into()),
            source: BackgroundSource::Custom,
            preset: BackgroundPreset::Midnight,
            custom_path: Some("C:\\managed\\wallpaper.webp".into()),
            fit: BackgroundFit::Contain,
            overlay: 32,
            blur: 6,
        };
        store
            .update_background_settings(display_two.clone())
            .unwrap();

        assert_eq!(
            store
                .get_background_settings(Some("display-two".into()))
                .unwrap(),
            display_two
        );
        assert_eq!(
            store.get_background_settings(None).unwrap(),
            BackgroundSettings::defaults_for(None)
        );
    }

    #[test]
    fn keeps_widget_layouts_separate_by_monitor_and_template() {
        let store = AppStore::in_memory().unwrap();
        let focus = WidgetLayout {
            monitor_id: Some("display-two".into()),
            template: WallpaperTemplate::Focus,
            x: 0.08,
            y: 0.12,
            width: 0.42,
            height: 0.58,
            locked: true,
            snap_to_grid: false,
        };
        store.update_widget_layout(focus.clone()).unwrap();

        assert_eq!(
            store
                .get_widget_layout(Some("display-two".into()), WallpaperTemplate::Focus)
                .unwrap(),
            focus
        );
        assert_eq!(
            store
                .get_widget_layout(Some("display-two".into()), WallpaperTemplate::Kanban)
                .unwrap(),
            WidgetLayout::defaults_for(Some("display-two".into()), WallpaperTemplate::Kanban)
        );

        let reset = store
            .reset_widget_layout(Some("display-two".into()), WallpaperTemplate::Focus)
            .unwrap();
        assert_eq!(
            reset,
            WidgetLayout::defaults_for(Some("display-two".into()), WallpaperTemplate::Focus)
        );
    }

    #[test]
    fn rejects_widget_layouts_outside_the_visible_surface() {
        let store = AppStore::in_memory().unwrap();
        let error = store
            .update_widget_layout(WidgetLayout {
                monitor_id: None,
                template: WallpaperTemplate::Focus,
                x: 0.80,
                y: 0.10,
                width: 0.30,
                height: 0.50,
                locked: false,
                snap_to_grid: true,
            })
            .unwrap_err();
        assert_eq!(
            error,
            "Widget yerleşimi görünür ekran sınırları içinde olmalıdır."
        );

        let oversized = store
            .update_widget_layout(WidgetLayout {
                monitor_id: None,
                template: WallpaperTemplate::Focus,
                x: 0.01,
                y: 0.01,
                width: 0.79,
                height: 0.50,
                locked: false,
                snap_to_grid: true,
            })
            .unwrap_err();
        assert_eq!(oversized, "Widget boyutu izin verilen aralıkta olmalıdır.");
    }

    #[test]
    fn manages_independent_desktop_widget_catalogs() {
        let store = AppStore::in_memory().unwrap();
        assert_eq!(store.list_desktop_widgets(None).unwrap().len(), 1);

        let clock = store
            .add_desktop_widget(Some("display-two".into()), WidgetKind::Clock)
            .unwrap();
        let duplicate = store.duplicate_desktop_widget(clock.id).unwrap();
        let pomodoro = store
            .add_desktop_widget(Some("display-two".into()), WidgetKind::Pomodoro)
            .unwrap();
        assert_eq!(store.list_desktop_widgets(None).unwrap().len(), 1);
        assert_eq!(
            store
                .list_desktop_widgets(Some("display-two".into()))
                .unwrap()
                .len(),
            3
        );

        let reordered = store
            .reorder_desktop_widgets(
                Some("display-two".into()),
                vec![pomodoro.id, duplicate.id, clock.id],
            )
            .unwrap();
        assert_eq!(
            reordered.iter().map(|widget| widget.id).collect::<Vec<_>>(),
            vec![pomodoro.id, duplicate.id, clock.id]
        );
        store.delete_desktop_widget(duplicate.id).unwrap();
        assert_eq!(
            store
                .list_desktop_widgets(Some("display-two".into()))
                .unwrap()
                .len(),
            2
        );
    }

    #[test]
    fn keeps_pomodoro_time_in_the_native_store() {
        let store = AppStore::in_memory().unwrap();
        let widget = store
            .add_desktop_widget(None, WidgetKind::Pomodoro)
            .unwrap();
        let configured = store.configure_pomodoro(widget.id, 50, 10).unwrap();
        assert_eq!(configured.remaining_seconds, 3_000);

        let running = store
            .update_pomodoro(widget.id, PomodoroAction::Start)
            .unwrap();
        assert!(running.running);
        assert!(running.ends_at.is_some());

        let paused = store
            .update_pomodoro(widget.id, PomodoroAction::Pause)
            .unwrap();
        assert!(!paused.running);
        assert!(paused.remaining_seconds > 0);

        let skipped = store
            .update_pomodoro(widget.id, PomodoroAction::Skip)
            .unwrap();
        assert_eq!(skipped.mode, PomodoroMode::Break);
        assert_eq!(skipped.remaining_seconds, 600);

        let reset = store
            .update_pomodoro(widget.id, PomodoroAction::Reset)
            .unwrap();
        assert_eq!(reset.mode, PomodoroMode::Work);
        assert_eq!(reset.remaining_seconds, 3_000);

        {
            let connection = store.lock_connection().unwrap();
            connection
                .execute(
                    "UPDATE pomodoro_states SET running = 1, ends_at = 0 WHERE widget_id = ?1",
                    [widget.id],
                )
                .unwrap();
        }
        let elapsed = store.get_pomodoro_state(widget.id).unwrap();
        assert_eq!(elapsed.mode, PomodoroMode::Break);
        assert_eq!(elapsed.remaining_seconds, 600);
        assert!(!elapsed.running);

        let duplicate = store.duplicate_desktop_widget(widget.id).unwrap();
        let duplicate_state = store.get_pomodoro_state(duplicate.id).unwrap();
        assert_eq!(duplicate_state.work_minutes, 50);
        assert_eq!(duplicate_state.break_minutes, 10);
    }

    #[test]
    fn migrates_legacy_widget_layouts_once() {
        let connection = Connection::open_in_memory().unwrap();
        connection
            .execute_batch(
                "CREATE TABLE app_settings (
                    id INTEGER PRIMARY KEY CHECK (id = 1), template TEXT NOT NULL,
                    opacity INTEGER NOT NULL, edit_mode INTEGER NOT NULL
                 );
                 INSERT INTO app_settings VALUES (1, 'focus', 82, 0);
                 CREATE TABLE widget_layouts (
                    monitor_key TEXT NOT NULL, template TEXT NOT NULL,
                    x REAL NOT NULL, y REAL NOT NULL, width REAL NOT NULL, height REAL NOT NULL,
                    locked INTEGER NOT NULL, snap_to_grid INTEGER NOT NULL,
                    PRIMARY KEY (monitor_key, template)
                 );
                 INSERT INTO widget_layouts VALUES ('display-two', 'kanban', .20, .15, .44, .54, 1, 0);",
            )
            .unwrap();
        let store = AppStore::from_connection(connection).unwrap();
        let widgets = store
            .list_desktop_widgets(Some("display-two".into()))
            .unwrap();
        assert_eq!(widgets.len(), 1);
        assert_eq!(widgets[0].kind, WidgetKind::Kanban);
        assert!(widgets[0].locked);
        assert!(!widgets[0].snap_to_grid);

        store.delete_desktop_widget(widgets[0].id).unwrap();
        assert!(
            store
                .list_desktop_widgets(Some("display-two".into()))
                .unwrap()
                .is_empty()
        );
    }

    #[test]
    fn keeps_tasks_after_the_database_is_reopened() {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let database_path = std::env::temp_dir().join(format!(
            "interactivebackground-{}-{unique_suffix}.db",
            std::process::id()
        ));

        {
            let store = AppStore::open(&database_path).unwrap();
            store
                .create(
                    "Yeniden açıldığında beni hatırla".into(),
                    Some("12:00".into()),
                )
                .unwrap();
            store
                .update_settings(AppSettings {
                    template: WallpaperTemplate::Kanban,
                    opacity: 74,
                    edit_mode: true,
                    monitor_id: Some("monitor:0:0:1920x1080".into()),
                    auto_calm_minutes: Some(15),
                    theme: ThemePreference::Dark,
                    language: LanguagePreference::En,
                })
                .unwrap();
        }

        {
            let reopened_store = AppStore::open(&database_path).unwrap();
            let tasks = reopened_store.list().unwrap();
            assert!(
                tasks
                    .iter()
                    .any(|task| task.title == "Yeniden açıldığında beni hatırla")
            );
            assert_eq!(
                reopened_store.get_settings().unwrap(),
                AppSettings {
                    template: WallpaperTemplate::Kanban,
                    opacity: 74,
                    edit_mode: true,
                    monitor_id: Some("monitor:0:0:1920x1080".into()),
                    auto_calm_minutes: Some(15),
                    theme: ThemePreference::Dark,
                    language: LanguagePreference::En,
                }
            );
        }

        std::fs::remove_file(database_path).unwrap();
    }
}
