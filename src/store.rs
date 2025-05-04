use rusqlite::{Connection, Result, Statement, prepare_and_bind};
use thiserror::Error;

use crate::config::Config;
use crate::wallpaper::Wallpaper;

#[derive(Debug, Error)]
pub enum StoreError {
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

static SETUP_SQL: &str = "CREATE TABLE IF NOT EXISTS seen_wallpapers (
           wallpaper TEXT PRIMARY KEY,
           seen BOOLEAN
        )";

fn make_have_seen_sql<'a>(db: &'a Connection, path: &str) -> Result<Statement<'a>> {
    Ok(prepare_and_bind!(
        db,
        "SELECT 1 FROM seen_wallpapers WHERE wallpaper = :path"
    ))
}

fn make_mark_sql<'a>(db: &'a Connection, path: &str) -> Result<Statement<'a>> {
    Ok(prepare_and_bind!(
        db,
        "INSERT INTO seen_wallpapers (wallpaper, seen)
        VALUES (:path, 1)
        ON CONFLICT(wallpaper) DO UPDATE SET seen = 1"
    ))
}

fn make_insert_wallpaper_sql<'a>(db: &'a Connection, path: &str) -> Result<Statement<'a>> {
    Ok(prepare_and_bind!(
        db,
        "INSERT INTO seen_wallpapers (wallpaper, seen) VALUES(:path, 0)"
    ))
}

impl Store {
    pub fn new(config: &Config) -> Result<Store> {
        let conn = Connection::open_in_memory()?;
        let _ = conn.execute(SETUP_SQL, ())?;
        let store = Store { connection: conn };
        Ok(store)
    }

    pub fn mark_as_seen(&self, wallpaper: &Wallpaper) -> Result<(), StoreError> {
        let mut stmt = make_mark_sql(&self.connection, &wallpaper.path)
            .map_err(|err| StoreError::UnknownError(err.to_string()))?;
        stmt.execute([wallpaper.path.clone()])
            .map_err(|_| StoreError::UpdateFailed)?;
        Ok(())
    }

    pub fn have_seen(&self, wallpaper: &Wallpaper) -> bool {
        match make_have_seen_sql(&self.connection, &wallpaper.path) {
            Ok(mut stmt) => !stmt
                .query_map([wallpaper.path.clone()], |row| row.get::<_, u8>(0))
                .unwrap()
                .filter_map(|row| row.ok())
                .collect::<Vec<u8>>()
                .is_empty(),
            Err(_) => false,
        }
    }

    pub fn insert_wallpaper(&self, wallpaper: &Wallpaper) -> Result<(), StoreError> {
        make_insert_wallpaper_sql(&self.connection, &wallpaper.path)
            .map_err(|_| StoreError::InsertFailed)?;
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_mark_seen() {
        let store = Store::new().unwrap();
        let wallpaper = Wallpaper {
            path: "test".to_string(),
        };

        let not_seen = store.have_seen(&wallpaper);
        let _ = store.mark_as_seen(&wallpaper);
        let now_seen = store.have_seen(&wallpaper);
        assert!(!not_seen);
        assert!(now_seen);
    }

    #[test]
    fn test_insert_wallpaper() {
        let store = Store::new().unwrap();
        let wallpaper = Wallpaper {
            path: "test".to_string(),
        };

        let _ = store.insert_wallpaper(&wallpaper);
    }
}
