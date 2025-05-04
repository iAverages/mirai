use rusqlite::{Connection, Result, Statement, prepare_and_bind};
use thiserror::Error;

use crate::config::Config;
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

#[derive(Debug)]
pub struct DatabaseWallpaper {
    pub id: String,
    pub seen: bool,
    pub manager_id: u32,
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
        "INSERT INTO seen_wallpapers (id, seen, manager_id) VALUES(:id, 0, :manager_id)"
    ))
}

fn make_get_all_wallpapers_sql(db: &Connection) -> Result<Statement<'_>> {
    db.prepare("SELECT * FROM seen_wallpapers")
}

impl Store {
    pub fn new() -> Result<Store> {
        let conn = Connection::open_in_memory()?;
        let _ = conn.execute(SETUP_SQL, ())?;
        let store = Store { connection: conn };
        Ok(store)
    }

    pub fn mark_as_seen(&self, wallpaper: &impl Wallpaper) -> Result<(), StoreError> {
        println!("marking id as seen: {}", wallpaper.get_id());
        let manager_id: u8 = wallpaper.get_type_id().into();

        let mut stmt = make_mark_sql(
            &self.connection,
            wallpaper.get_id(),
            &manager_id.to_string(),
        )
        .map_err(|err| StoreError::UnknownError(err.to_string()))?;
        let a = stmt
            .execute([wallpaper.get_id(), &manager_id.to_string()])
            .map_err(|err| {
                println!("{:?}", err);
                StoreError::UpdateFailed
            })?;

        println!("size?: {:?}", a);
        Ok(())
    }

    pub fn have_seen(&self, wallpaper: &impl Wallpaper) -> bool {
        // TODO: dont eat errors
        match make_have_seen_sql(&self.connection, wallpaper.get_id()) {
            Ok(mut stmt) => stmt
                .query_row([wallpaper.get_id()], |row| row.get::<_, u8>(0))
                .is_ok(),
            Err(_) => false,
        }
    }

    pub fn insert_wallpaper(&self, wallpaper: &impl Wallpaper) -> Result<(), StoreError> {
        let manager_id: u8 = wallpaper.get_type_id().into();
        make_insert_wallpaper_sql(
            &self.connection,
            wallpaper.get_id(),
            &manager_id.to_string(),
        )
        .map_err(|err| StoreError::QueryError(err.to_string()))?
        .execute([wallpaper.get_id(), &manager_id.to_string()])
        .map_err(|_| StoreError::InsertFailed)?;

        Ok(())
    }

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
        .filter_map(|row| {
            println!("is_ok {:?}", row.is_ok());
            row.ok()
        })
        .collect::<Vec<_>>()
    }
}

#[cfg(test)]
mod tests {
    use std::error::Error;
    use std::sync::Once;

    use crate::CONFIG;
    use crate::content_managers::local::LocalWallpaper;

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
        let wallpaper = LocalWallpaper::new("test");

        let not_seen = store.have_seen(&wallpaper);
        store.mark_as_seen(&wallpaper)?;
        let now_seen = store.have_seen(&wallpaper);
        assert!(!not_seen);
        assert!(now_seen);
        Ok(())
    }

    #[test]
    fn test_insert_wallpaper() -> Result<(), Box<dyn Error>> {
        setup();
        let store = Store::new()?;
        let wallpaper = LocalWallpaper::new("test");

        store.insert_wallpaper(&wallpaper)?;

        Ok(())
    }
}
