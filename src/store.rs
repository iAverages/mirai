use std::path::PathBuf;

use rusqlite::{Connection, Result, Statement, prepare_and_bind};
use thiserror::Error;

use crate::content_managers::ContentManagerTypes;
use crate::get_config;
use crate::wallpaper::Wallpaper;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("an error occured while generating query")]
    QueryError(String),
    #[error("failed to insert into store")]
    InsertFailed,
    #[error("failed to update row in store")]
    UpdateFailed,
    #[error("unknown error: {0}")]
    UnknownError(String),
}

pub struct Store {
    connection: Connection,
}

#[derive(Debug, Clone)]
pub struct DatabaseWallpaper {
    pub id: String,
    pub seen: bool,
    pub manager_id: u8,
}

impl TryInto<Wallpaper> for DatabaseWallpaper {
    type Error = ();

    fn try_into(self) -> Result<Wallpaper, Self::Error> {
        let manager_id = match self.manager_id {
            0 => Some(ContentManagerTypes::Local),
            _ => None,
        }
        .ok_or(())?;
        Ok(Wallpaper::new(self.id, manager_id))
    }
}

static SETUP_SQL: &str = "CREATE TABLE IF NOT EXISTS seen_wallpapers (
           id TEXT PRIMARY KEY,
           seen BOOLEAN,
           manager_id INTEGER 
        )";

fn make_have_seen_sql<'a>(db: &'a Connection, path: &str) -> Result<Statement<'a>> {
    Ok(prepare_and_bind!(
        db,
        "SELECT 1 FROM seen_wallpapers WHERE id = :path"
    ))
}

fn make_mark_sql<'a>(db: &'a Connection, id: &str, manager_id: &str) -> Result<Statement<'a>> {
    Ok(prepare_and_bind!(
        db,
        "INSERT INTO seen_wallpapers (id, seen, manager_id)
        VALUES (:id, 1, :manager_id)
        ON CONFLICT(id) DO UPDATE SET seen = 1"
    ))
}

fn make_insert_wallpaper_sql<'a>(
    db: &'a Connection,
    id: &str,
    manager_id: &str,
) -> Result<Statement<'a>> {
    Ok(prepare_and_bind!(
        db,
        "INSERT INTO seen_wallpapers (id, seen, manager_id)
         VALUES (:id, 0, :manager_id)
         ON CONFLICT(id) DO UPDATE SET
         manager_id = excluded.manager_id;"
    ))
}

fn make_get_all_wallpapers_sql(db: &Connection) -> Result<Statement<'_>> {
    db.prepare("SELECT * FROM seen_wallpapers")
}

fn make_get_unseen_wallpapers_sql(db: &Connection) -> Result<Statement<'_>> {
    db.prepare("SELECT * FROM seen_wallpapers WHERE seen = 0")
}

impl Store {
    pub fn new() -> Result<Store> {
        let conn = if cfg!(test) {
            Connection::open_in_memory()?
        } else {
            let data_dir_path: PathBuf = get_config().data_dir.clone().into();
            Connection::open(data_dir_path.join("data.sqlite"))?
        };
        let _ = conn.execute(SETUP_SQL, ())?;
        let store = Store { connection: conn };
        Ok(store)
    }

    pub fn mark_as_seen(&self, wallpaper: &Wallpaper) -> Result<(), StoreError> {
        println!("marking id as seen: {}", wallpaper.id);
        let manager_id: u8 = wallpaper.type_id.into();

        let mut stmt = make_mark_sql(
            &self.connection,
            wallpaper.id.as_str(),
            &manager_id.to_string(),
        )
        .map_err(|err| StoreError::UnknownError(err.to_string()))?;
        stmt.execute([wallpaper.id.as_str(), &manager_id.to_string()])
            .map_err(|err| {
                println!("{:?}", err);
                StoreError::UpdateFailed
            })?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn have_seen(&self, wallpaper: &Wallpaper) -> bool {
        // TODO: dont eat errors
        match make_have_seen_sql(&self.connection, wallpaper.id.as_str()) {
            Ok(mut stmt) => stmt
                .query_row([wallpaper.id.as_str()], |row| row.get::<_, u8>(0))
                .is_ok(),
            Err(_) => false,
        }
    }

    pub fn insert_wallpaper(&self, wallpaper: &Wallpaper) -> Result<(), StoreError> {
        let manager_id: u8 = wallpaper.type_id.into();
        make_insert_wallpaper_sql(
            &self.connection,
            wallpaper.id.as_str(),
            &manager_id.to_string(),
        )
        .map_err(|err| StoreError::QueryError(err.to_string()))?
        .execute([wallpaper.id.as_str(), &manager_id.to_string()])
        .map_err(|_| StoreError::InsertFailed)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_inserted_wallpapers(&self) -> Vec<DatabaseWallpaper> {
        // TODO: dont eat errors
        let mut stmt = make_get_all_wallpapers_sql(&self.connection).expect("failed to make query");
        stmt.query_map([], |row| {
            Ok(DatabaseWallpaper {
                id: row.get::<_, _>(0)?,
                manager_id: row.get::<_, _>(1)?,
                seen: row.get::<_, _>(2)?,
            })
        })
        .unwrap()
        .filter_map(|row| row.ok())
        .collect::<Vec<_>>()
    }

    pub fn get_unseen_wallpaperrs(&self) -> Vec<DatabaseWallpaper> {
        // TODO: dont eat errors
        let mut stmt =
            make_get_unseen_wallpapers_sql(&self.connection).expect("failed to make query");
        stmt.query_map([], |row| {
            Ok(DatabaseWallpaper {
                id: row.get::<_, _>(0)?,
                manager_id: row.get::<_, _>(1)?,
                seen: row.get::<_, _>(2)?,
            })
        })
        .unwrap()
        .filter_map(|row| row.ok())
        .collect::<Vec<_>>()
    }

    pub fn reset_seen_state(&self) {
        self.connection
            .execute("UPDATE seen_wallpapers SET seen = 0", [])
            .expect("failed to reset seen status");
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::sync::Once;

    use crate::CONFIG;
    use crate::config::Config;
    use crate::content_managers::ContentManagerTypes;

    use super::*;

    static SETUP: Once = Once::new();
    pub fn setup() {
        SETUP.call_once(|| {
            let config = Config::create_config();
            CONFIG.set(config).expect("failed to set ");
        });
    }

    #[test]
    fn test_mark_seen() -> Result<(), Box<dyn Error>> {
        setup();
        let store = Store::new()?;
        let wallpaper1 = Wallpaper::new("test".to_string(), ContentManagerTypes::Local);
        let wallpaper2 = Wallpaper::new("test2".to_string(), ContentManagerTypes::Local);

        let not_seen1 = store.have_seen(&wallpaper1);
        let not_seen2 = store.have_seen(&wallpaper2);

        store.mark_as_seen(&wallpaper1)?;
        let now_seen1 = store.have_seen(&wallpaper1);
        let still_not_seen2 = store.have_seen(&wallpaper2);

        assert!(!not_seen1);
        assert!(!not_seen2);

        assert!(now_seen1);
        assert!(!still_not_seen2);
        Ok(())
    }

    #[test]
    fn test_insert_wallpaper() -> Result<(), Box<dyn Error>> {
        setup();
        let store = Store::new()?;
        let wallpaper = Wallpaper::new("test".to_string(), ContentManagerTypes::Local);

        store.insert_wallpaper(&wallpaper)?;

        Ok(())
    }
}
