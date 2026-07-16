use std::{fs, path::Path};

use rusqlite::{Connection, OptionalExtension, params};

const DATABASE_NAME: &str = "flowdesk.db";
const LEGACY_IDENTIFIER: &str = "com.flowdesk.app";
const MIGRATION_MARKER: &str = "legacy-flowdesk-migration-v1";

#[derive(Debug, PartialEq, Eq)]
pub enum MigrationOutcome {
    Migrated,
    LegacyDataNotFound,
    CurrentDataAlreadyExists,
}

/// Copies the previous Flowdesk application data into the directory derived
/// from the new Tauri identifier. The source is deliberately retained as a
/// rollback copy; repeated launches never overwrite a current database.
pub fn migrate_legacy_app_data(current_directory: &Path) -> Result<MigrationOutcome, String> {
    let roaming_directory = current_directory
        .parent()
        .ok_or_else(|| "Uygulama veri dizininin üst klasörü bulunamadı.".to_string())?;
    migrate_from_directory(
        &roaming_directory.join(LEGACY_IDENTIFIER),
        current_directory,
    )
}

fn migrate_from_directory(
    legacy_directory: &Path,
    current_directory: &Path,
) -> Result<MigrationOutcome, String> {
    let current_database = current_directory.join(DATABASE_NAME);
    if current_database.exists() {
        return Ok(MigrationOutcome::CurrentDataAlreadyExists);
    }

    let legacy_database = legacy_directory.join(DATABASE_NAME);
    if !legacy_database.is_file() {
        return Ok(MigrationOutcome::LegacyDataNotFound);
    }

    fs::create_dir_all(current_directory).map_err(file_error)?;
    let temporary_database = current_directory.join(format!("{DATABASE_NAME}.migrating"));
    if temporary_database.exists() {
        fs::remove_file(&temporary_database).map_err(file_error)?;
    }

    let migration_result = (|| {
        create_consistent_snapshot(&legacy_database, &temporary_database)?;
        rewrite_managed_background_paths(&temporary_database, legacy_directory, current_directory)?;

        let legacy_backgrounds = legacy_directory.join("backgrounds");
        if legacy_backgrounds.is_dir() {
            copy_directory_contents(&legacy_backgrounds, &current_directory.join("backgrounds"))?;
        }

        fs::rename(&temporary_database, &current_database).map_err(file_error)?;
        if let Err(error) = fs::write(
            current_directory.join(MIGRATION_MARKER),
            "Legacy com.flowdesk.app verisi kopyalandı; kaynak rollback için korundu.\n",
        ) {
            eprintln!("Legacy veri taşıma işareti yazılamadı: {error}");
        }
        Ok(MigrationOutcome::Migrated)
    })();

    if migration_result.is_err() && temporary_database.exists() {
        let _ = fs::remove_file(temporary_database);
    }
    migration_result
}

fn create_consistent_snapshot(source: &Path, destination: &Path) -> Result<(), String> {
    let connection = Connection::open(source).map_err(database_error)?;
    connection
        .execute("VACUUM INTO ?1", params![destination.to_string_lossy()])
        .map_err(database_error)?;
    Ok(())
}

fn rewrite_managed_background_paths(
    database: &Path,
    legacy_directory: &Path,
    current_directory: &Path,
) -> Result<(), String> {
    let mut connection = Connection::open(database).map_err(database_error)?;
    let has_background_table = connection
        .query_row(
            "SELECT 1 FROM sqlite_master WHERE type = 'table' AND name = 'monitor_backgrounds'",
            [],
            |_| Ok(()),
        )
        .optional()
        .map_err(database_error)?
        .is_some();
    if !has_background_table {
        return Ok(());
    }

    let replacements = {
        let mut statement = connection
            .prepare(
                "SELECT monitor_key, custom_path FROM monitor_backgrounds WHERE custom_path IS NOT NULL",
            )
            .map_err(database_error)?;
        let rows = statement
            .query_map([], |row| {
                Ok((row.get::<_, String>(0)?, row.get::<_, String>(1)?))
            })
            .map_err(database_error)?;

        rows.collect::<Result<Vec<_>, _>>()
            .map_err(database_error)?
            .into_iter()
            .filter_map(|(monitor_key, custom_path)| {
                let relative = Path::new(&custom_path)
                    .strip_prefix(legacy_directory)
                    .ok()?;
                Some((
                    monitor_key,
                    current_directory
                        .join(relative)
                        .to_string_lossy()
                        .into_owned(),
                ))
            })
            .collect::<Vec<_>>()
    };

    let transaction = connection.transaction().map_err(database_error)?;
    for (monitor_key, custom_path) in replacements {
        transaction
            .execute(
                "UPDATE monitor_backgrounds SET custom_path = ?1 WHERE monitor_key = ?2",
                params![custom_path, monitor_key],
            )
            .map_err(database_error)?;
    }
    transaction.commit().map_err(database_error)
}

fn copy_directory_contents(source: &Path, destination: &Path) -> Result<(), String> {
    fs::create_dir_all(destination).map_err(file_error)?;
    for entry in fs::read_dir(source).map_err(file_error)? {
        let entry = entry.map_err(file_error)?;
        let source_path = entry.path();
        let destination_path = destination.join(entry.file_name());
        if source_path.is_dir() {
            copy_directory_contents(&source_path, &destination_path)?;
        } else if source_path.is_file() {
            if !destination_path.exists() {
                fs::copy(source_path, destination_path).map_err(file_error)?;
            }
        }
    }
    Ok(())
}

fn database_error(error: rusqlite::Error) -> String {
    format!("Legacy veri taşınırken SQLite hatası oluştu: {error}")
}

fn file_error(error: std::io::Error) -> String {
    format!("Legacy veri taşınırken dosya hatası oluştu: {error}")
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    fn test_directory(name: &str) -> PathBuf {
        let nonce = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("interactivebackground-{name}-{nonce}"))
    }

    fn create_legacy_database(directory: &Path) {
        fs::create_dir_all(directory.join("backgrounds")).unwrap();
        fs::write(directory.join("backgrounds/wallpaper.webp"), b"image").unwrap();
        let connection = Connection::open(directory.join(DATABASE_NAME)).unwrap();
        connection
            .execute_batch(
                "CREATE TABLE tasks (id INTEGER PRIMARY KEY, title TEXT NOT NULL);
                 CREATE TABLE monitor_backgrounds (
                    monitor_key TEXT PRIMARY KEY,
                    custom_path TEXT
                 );
                 INSERT INTO tasks (id, title) VALUES (1, 'Korunan görev');",
            )
            .unwrap();
        connection
            .execute(
                "INSERT INTO monitor_backgrounds (monitor_key, custom_path) VALUES ('primary', ?1)",
                [directory
                    .join("backgrounds/wallpaper.webp")
                    .to_string_lossy()
                    .into_owned()],
            )
            .unwrap();
    }

    #[test]
    fn migrates_database_backgrounds_and_absolute_paths_without_deleting_source() {
        let root = test_directory("legacy-copy");
        let legacy = root.join(LEGACY_IDENTIFIER);
        let current = root.join("com.saidtokmak.interactivebackground");
        create_legacy_database(&legacy);

        assert_eq!(
            migrate_from_directory(&legacy, &current).unwrap(),
            MigrationOutcome::Migrated
        );
        assert!(legacy.join(DATABASE_NAME).exists());
        assert!(current.join("backgrounds/wallpaper.webp").exists());
        assert!(current.join(MIGRATION_MARKER).exists());

        let connection = Connection::open(current.join(DATABASE_NAME)).unwrap();
        let title: String = connection
            .query_row("SELECT title FROM tasks WHERE id = 1", [], |row| row.get(0))
            .unwrap();
        let custom_path: String = connection
            .query_row(
                "SELECT custom_path FROM monitor_backgrounds WHERE monitor_key = 'primary'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(title, "Korunan görev");
        assert_eq!(
            PathBuf::from(custom_path),
            current.join("backgrounds/wallpaper.webp")
        );

        drop(connection);
        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn never_overwrites_a_current_database() {
        let root = test_directory("idempotent");
        let legacy = root.join(LEGACY_IDENTIFIER);
        let current = root.join("com.saidtokmak.interactivebackground");
        create_legacy_database(&legacy);
        fs::create_dir_all(&current).unwrap();
        fs::write(current.join(DATABASE_NAME), b"current").unwrap();

        assert_eq!(
            migrate_from_directory(&legacy, &current).unwrap(),
            MigrationOutcome::CurrentDataAlreadyExists
        );
        assert_eq!(fs::read(current.join(DATABASE_NAME)).unwrap(), b"current");

        fs::remove_dir_all(root).unwrap();
    }

    #[test]
    fn leaves_new_installs_untouched_when_legacy_data_is_missing() {
        let root = test_directory("new-install");
        let legacy = root.join(LEGACY_IDENTIFIER);
        let current = root.join("com.saidtokmak.interactivebackground");

        assert_eq!(
            migrate_from_directory(&legacy, &current).unwrap(),
            MigrationOutcome::LegacyDataNotFound
        );
        assert!(!current.exists());
    }
}
