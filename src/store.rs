use std::path::PathBuf;

use chrono::{DateTime, Local};
use rusqlite::{Connection, Error, Result, Row};
use sea_query::{Expr, Iden, OnConflict, Query, SqliteQueryBuilder};
use sea_query_rusqlite::RusqliteBinder;
use thiserror::Error;

use crate::content_managers::ContentManagerTypes;
use crate::get_config;
use crate::wallpaper::Wallpaper;

#[derive(Debug, Error)]
pub enum StoreError {
    #[error("failed to insert into store")]
    InsertFailed,
    #[error("failed to update row in store")]
    UpdateFailed,
}

refinery::embed_migrations!("migrations");

pub struct Store {
    connection: Connection,
}

#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct DatabaseWallpaper {
    pub id: String,
    pub seen: bool,
    pub manager_id: u8,
}

impl From<&Row<'_>> for DatabaseWallpaper {
    fn from(row: &Row) -> Self {
        Self {
            id: row.get_unwrap("id"),
            seen: row.get_unwrap("seen"),
            manager_id: row.get_unwrap("manager_id"),
        }
    }
}

impl TryInto<Wallpaper> for DatabaseWallpaper {
    type Error = ();

    fn try_into(self) -> Result<Wallpaper, Self::Error> {
        let manager_id: ContentManagerTypes = self.manager_id.try_into()?;
        Ok(Wallpaper::new(self.id, manager_id))
    }
}

#[derive(Iden)]
enum SeenWallpapers {
    Table,
    Id,
    Seen,
    ManagerId,
}

#[derive(Iden)]
enum Meta {
    Table,
    Id,
    LastUpdate,
    LastUsed,
}

fn log_query_error(err: &Error) {
    if rusqlite::Error::QueryReturnedNoRows == *err {
        tracing::trace!("query returned no rows");
    } else {
        tracing::error!("query error: {}", err);
    }
}

impl Store {
    pub fn new() -> Result<Store> {
        let mut conn = if cfg!(test) {
            Connection::open_in_memory()?
        } else {
            let data_dir_path: PathBuf = get_config().data_dir.clone().into();
            Connection::open(data_dir_path.join("data.sqlite"))?
        };

        migrations::runner().run(&mut conn).unwrap();
        let store = Store { connection: conn };
        Ok(store)
    }

    pub fn mark_as_seen(&self, wallpaper: &Wallpaper) -> Result<(), StoreError> {
        tracing::info!("marking id as seen: {}", wallpaper.id);
        let manager_id: u8 = wallpaper.type_id.into();

        let (sql, values) = Query::insert()
            .into_table(SeenWallpapers::Table)
            .columns([
                SeenWallpapers::Id,
                SeenWallpapers::Seen,
                SeenWallpapers::ManagerId,
            ])
            .values_panic([(&wallpaper.id).into(), 1.into(), manager_id.into()])
            .on_conflict(
                OnConflict::column(SeenWallpapers::Id)
                    .update_column(SeenWallpapers::Seen)
                    .to_owned(),
            )
            .build_rusqlite(SqliteQueryBuilder);

        self.connection
            .execute(sql.as_str(), &*values.as_params())
            .inspect_err(log_query_error)
            .map_err(|_| StoreError::UpdateFailed)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn have_seen(&self, wallpaper: &Wallpaper) -> bool {
        let (sql, values) = Query::select()
            .column(SeenWallpapers::Id)
            .from(SeenWallpapers::Table)
            .and_where(Expr::col(SeenWallpapers::Id).eq(wallpaper.id.as_str()))
            .and_where(Expr::col(SeenWallpapers::Seen).eq(1))
            .build_rusqlite(SqliteQueryBuilder);

        let stmt = self.connection.prepare(sql.as_str()).inspect_err(|err| {
            tracing::error!("error preparing query: {}", err);
        });
        if let Ok(mut stmt) = stmt {
            return stmt
                .query_row(&*values.as_params(), |row| row.get::<_, String>(0))
                .inspect_err(log_query_error)
                .is_ok();
        }

        false
    }

    pub fn insert_wallpaper(&self, wallpaper: &Wallpaper) -> Result<(), StoreError> {
        let manager_id: u8 = wallpaper.type_id.into();

        let (sql, values) = Query::insert()
            .into_table(SeenWallpapers::Table)
            .columns([
                SeenWallpapers::Id,
                SeenWallpapers::Seen,
                SeenWallpapers::ManagerId,
            ])
            .values_panic([(&wallpaper.id).into(), 0.into(), manager_id.into()])
            .on_conflict(
                OnConflict::column(SeenWallpapers::Id)
                    // TODO: make primary key for this table a composite key with id:manager_id
                    .update_column(SeenWallpapers::ManagerId)
                    .to_owned(),
            )
            .build_rusqlite(SqliteQueryBuilder);

        self.connection
            .execute(sql.as_str(), &*values.as_params())
            .inspect_err(log_query_error)
            .map_err(|_| StoreError::InsertFailed)?;

        Ok(())
    }

    #[allow(dead_code)]
    pub fn get_inserted_wallpapers(&self) -> Vec<DatabaseWallpaper> {
        let (sql, _) = Query::select()
            .from(SeenWallpapers::Table)
            .columns([
                SeenWallpapers::Id,
                SeenWallpapers::ManagerId,
                SeenWallpapers::Seen,
            ])
            .build_rusqlite(SqliteQueryBuilder);
        let stmt = self.connection.prepare(sql.as_str()).inspect_err(|err| {
            tracing::error!("error preparing query: {}", err);
        });

        if let Ok(mut stmt) = stmt {
            return stmt
                .query_map([], |row| Ok(DatabaseWallpaper::from(row)))
                .inspect_err(log_query_error)
                .unwrap() // TODO: dont unwrap
                .filter_map(|row| match row {
                    Ok(row) => Some(row),
                    Err(error) => {
                        tracing::error!("got error from insert: {}", error);
                        None
                    }
                })
                .collect::<Vec<_>>();
        }

        vec![]
    }

    pub fn get_unseen_wallpaperrs(&self) -> Vec<DatabaseWallpaper> {
        let (sql, values) = Query::select()
            .from(SeenWallpapers::Table)
            .columns([
                SeenWallpapers::Id,
                SeenWallpapers::ManagerId,
                SeenWallpapers::Seen,
            ])
            .and_where(Expr::column(SeenWallpapers::Seen).eq(0))
            .build_rusqlite(SqliteQueryBuilder);
        let stmt = self.connection.prepare(sql.as_str()).inspect_err(|err| {
            tracing::error!("error preparing query: {}", err);
        });

        if let Ok(mut stmt) = stmt {
            return stmt
                .query_map(&*values.as_params(), |row| Ok(DatabaseWallpaper::from(row)))
                .inspect_err(log_query_error)
                .unwrap() // TODO: dont unwrap
                .filter_map(|row| match row {
                    Ok(row) => Some(row),
                    Err(error) => {
                        tracing::debug!("got error from insert: {}", error);
                        None
                    }
                })
                .collect::<Vec<_>>();
        }

        vec![]
    }

    pub fn reset_seen_state(&self) {
        let (sql, _) = Query::update()
            .table(SeenWallpapers::Table)
            .value(SeenWallpapers::Seen, 0)
            .build_rusqlite(SqliteQueryBuilder);
        self.connection
            .execute(sql.as_str(), [])
            .expect("failed to reset seen status");
    }

    pub fn get_last_update(&self) -> Option<DateTime<Local>> {
        let (sql, values) = Query::select()
            .from(Meta::Table)
            .column(Meta::LastUpdate)
            .limit(1)
            .build_rusqlite(SqliteQueryBuilder);

        self.connection
            .query_row(sql.as_str(), &*values.as_params(), |row| row.get(0))
            .ok()
    }

    pub fn update_last_run(&self) {
        let now = Local::now();
        tracing::info!("updating last run to {}", now);

        let (sql, values) = Query::insert()
            .into_table(Meta::Table)
            .columns([Meta::Id, Meta::LastUpdate])
            .values_panic([1.into(), now.naive_local().into()])
            .on_conflict(
                OnConflict::column(Meta::Id)
                    .update_column(Meta::LastUpdate)
                    .to_owned(),
            )
            .build_rusqlite(SqliteQueryBuilder);

        let _ = self
            .connection
            .execute(sql.as_str(), &*values.as_params())
            .inspect_err(log_query_error);
    }

    pub fn set_last_used(&self, wallpaper: &Wallpaper) {
        tracing::info!("updating last used wallpaper to {}", &wallpaper.id);

        let (sql, values) = Query::insert()
            .into_table(Meta::Table)
            .columns([Meta::Id, Meta::LastUsed])
            .values_panic([1.into(), (&wallpaper.id).into()])
            .on_conflict(
                OnConflict::column(Meta::Id)
                    .update_column(Meta::LastUsed)
                    .to_owned(),
            )
            .build_rusqlite(SqliteQueryBuilder);

        let _ = self
            .connection
            .execute(sql.as_str(), &*values.as_params())
            .inspect_err(log_query_error);
    }

    pub fn get_meta(&self) -> Option<MetaData> {
        let (sql, values) = Query::select()
            .from(Meta::Table)
            .columns([Meta::LastUsed, Meta::LastUpdate])
            .limit(1)
            .and_where(Expr::col(Meta::Id).eq(1))
            .build_rusqlite(SqliteQueryBuilder);

        self.connection
            .query_row(sql.as_str(), &*values.as_params(), |row| {
                Ok(MetaData {
                    last_update: row.get(Meta::LastUpdate.to_string().as_str())?,
                    last_used: row.get(Meta::LastUsed.to_string().as_str())?,
                })
            })
            .inspect_err(log_query_error)
            .ok()
    }

    pub fn get_wallpaper(&self, id: &str) -> Option<DatabaseWallpaper> {
        let manager_id: u8 = get_config().file_config.content_manager_type.into();
        let (sql, values) = Query::select()
            .from(SeenWallpapers::Table)
            .columns([
                SeenWallpapers::Id,
                SeenWallpapers::Seen,
                SeenWallpapers::ManagerId,
            ])
            .and_where(Expr::col(SeenWallpapers::Id).eq(id).to_owned())
            .and_where(
                Expr::col(SeenWallpapers::ManagerId)
                    .eq(manager_id)
                    .to_owned(),
            )
            .build_rusqlite(SqliteQueryBuilder);
        self.connection
            .query_row(sql.as_str(), &*values.as_params(), |row| {
                Ok(DatabaseWallpaper::from(row))
            })
            .ok()
    }
}

#[derive(Debug, Clone)]
pub struct MetaData {
    #[allow(dead_code)]
    pub last_update: Option<DateTime<Local>>,
    pub last_used: String,
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
    pub fn setup() -> Result<Store, Box<dyn Error>> {
        SETUP.call_once(|| {
            let config = Config::create_config();
            CONFIG.set(config).expect("failed to set ");
        });

        Ok(Store::new()?)
    }

    #[test]
    fn test_mark_seen() -> Result<(), Box<dyn Error>> {
        let store = setup()?;
        let wallpaper1 = Wallpaper::new("test".to_string(), ContentManagerTypes::Local);
        let wallpaper2 = Wallpaper::new("test2".to_string(), ContentManagerTypes::Local);

        store.insert_wallpaper(&wallpaper1)?;
        store.insert_wallpaper(&wallpaper2)?;

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
        let store = setup()?;
        let wallpaper = Wallpaper::new("test".to_string(), ContentManagerTypes::Local);

        store.insert_wallpaper(&wallpaper)?;

        let all = store.get_inserted_wallpapers();
        assert!(!all.is_empty());

        Ok(())
    }

    #[test]
    fn test_wallpaper_inserts_read_all() -> Result<(), Box<dyn Error>> {
        let store = setup()?;
        let all = store.get_inserted_wallpapers();
        assert!(all.is_empty());

        for idx in 0..10 {
            let wallpaper = Wallpaper::new(
                format!("test{idx:?}").to_string(),
                ContentManagerTypes::Local,
            );
            store.insert_wallpaper(&wallpaper)?;

            let all2 = store.get_inserted_wallpapers();
            assert_eq!(all2.len(), idx + 1);
        }

        Ok(())
    }

    #[test]
    fn test_last_update() -> Result<(), Box<dyn Error>> {
        let store = setup()?;
        assert!(store.get_last_update().is_none());
        store.update_last_run();
        let last_update = store.get_last_update();

        if let Some(first) = last_update {
            store.update_last_run();
            let last_update = store.get_last_update();

            if let Some(second) = last_update {
                assert_ne!(first, second);
                return Ok(());
            }
        }

        panic!("last_update not set");
    }

    #[test]
    fn test_last_used() -> Result<(), Box<dyn Error>> {
        let store = setup()?;
        assert!(store.get_meta().is_none());

        let wallpaper = Wallpaper::new("first".to_string(), ContentManagerTypes::Local);
        store.insert_wallpaper(&wallpaper)?;
        store.set_last_used(&wallpaper);

        let meta = store.get_meta();

        if let Some(first) = meta {
            let wallpaper2 = Wallpaper::new("second".to_string(), ContentManagerTypes::Local);
            store.insert_wallpaper(&wallpaper2)?;
            store.set_last_used(&wallpaper2);
            let meta2 = store.get_meta();

            if let Some(second) = meta2 {
                assert_ne!(first.last_used, second.last_used);
                return Ok(());
            }
        }

        panic!("meta not set");
    }
}
