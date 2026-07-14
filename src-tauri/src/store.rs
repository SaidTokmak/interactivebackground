use std::{path::Path, sync::Mutex};

use rusqlite::{Connection, OptionalExtension, Row, params};

use crate::{
    model::{Task, TaskStatus},
    settings::{AppSettings, WallpaperTemplate},
};

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
                     monitor_id TEXT
                 );

                 INSERT OR IGNORE INTO app_settings (id, template, opacity, edit_mode)
                 VALUES (1, 'focus', 82, 0);",
            )
            .map_err(database_error)?;

        // `CREATE TABLE IF NOT EXISTS` mevcut tabloya yeni sütun eklemez. Eski
        // Flowdesk veritabanlarını ileri taşımak için sütunu ayrıca kontrol ederiz.
        ensure_monitor_column(&connection)?;

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
                "SELECT template, opacity, edit_mode, monitor_id
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
                 SET template = ?1, opacity = ?2, edit_mode = ?3, monitor_id = ?4
                 WHERE id = 1",
                params![
                    settings.template.as_database_value(),
                    i64::from(settings.opacity),
                    settings.edit_mode,
                    settings.monitor_id,
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

    Ok(AppSettings {
        template,
        opacity,
        edit_mode: row.get(2)?,
        monitor_id: row.get(3)?,
    })
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
            })
            .unwrap();
        assert_eq!(updated.template, WallpaperTemplate::Kanban);
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

        let updated = store
            .update_settings(AppSettings {
                template: WallpaperTemplate::Focus,
                opacity: 82,
                edit_mode: false,
                monitor_id: Some("display:0:0:2560x1440".into()),
            })
            .unwrap();
        assert_eq!(store.get_settings().unwrap().monitor_id, updated.monitor_id);
    }

    #[test]
    fn keeps_tasks_after_the_database_is_reopened() {
        let unique_suffix = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let database_path = std::env::temp_dir().join(format!(
            "flowdesk-{}-{unique_suffix}.db",
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
                }
            );
        }

        std::fs::remove_file(database_path).unwrap();
    }
}
