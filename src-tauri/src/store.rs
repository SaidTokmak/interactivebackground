use std::{path::Path, sync::Mutex};

use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::{
    model::{Task, TaskStatus},
    settings::{
        AppSettings, BackgroundFit, BackgroundPreset, BackgroundSettings, BackgroundSource,
        LanguagePreference, ThemePreference, WallpaperTemplate,
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

    fn from_connection(connection: Connection) -> Result<Self, String> {
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
