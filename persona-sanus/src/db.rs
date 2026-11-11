use std::env;
use std::path::Path;

use anyhow::{anyhow, Context, Result};
use diesel::prelude::*;
use diesel::r2d2::{ConnectionManager, Pool};
use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

use crate::schema::{sessions, users};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!("migrations");

pub type DbPool = Pool<ConnectionManager<SqliteConnection>>;

#[derive(Debug, Queryable, Selectable)]
#[diesel(table_name = users)]
pub struct User {
    pub id: String,
    pub email: String,
    pub password_hash: String,
    pub display_name: String,
    pub created_at: String,
    pub updated_at: String,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = users)]
pub struct NewUser<'a> {
    pub id: &'a str,
    pub email: &'a str,
    pub password_hash: &'a str,
    pub display_name: &'a str,
    pub created_at: &'a str,
    pub updated_at: &'a str,
}

#[derive(Debug, Insertable)]
#[diesel(table_name = sessions)]
pub struct NewSession<'a> {
    pub id: &'a str,
    pub token_hash: &'a str,
    pub user_id: &'a str,
    pub expires_at: &'a str,
    pub created_at: &'a str,
}

pub fn connect() -> Result<DbPool> {
    let database_url = env::var("DATABASE_URL").unwrap_or_else(|_| "sqlite://app.db".to_owned());
    ensure_parent_dir(&database_url)?;
    let manager = ConnectionManager::<SqliteConnection>::new(database_url);
    let pool = Pool::builder()
        .max_size(8)
        .build(manager)
        .context("could not open the SQLite database")?;
    run_migrations(&pool)?;
    Ok(pool)
}

fn ensure_parent_dir(database_url: &str) -> Result<()> {
    let path = database_url
        .strip_prefix("sqlite://")
        .or_else(|| database_url.strip_prefix("sqlite:"))
        .unwrap_or(database_url);
    let path = path.split('?').next().unwrap_or(path);
    if path.is_empty() || path == ":memory:" {
        return Ok(());
    }
    if let Some(parent) = Path::new(path).parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent).with_context(|| {
                format!(
                    "could not create the database directory {}",
                    parent.display()
                )
            })?;
        }
    }
    Ok(())
}

fn run_migrations(pool: &DbPool) -> Result<()> {
    let mut pooled = pool.get()?;
    let connection: &mut SqliteConnection = &mut pooled;
    connection
        .run_pending_migrations(MIGRATIONS)
        .map(|_| ())
        .map_err(|error| anyhow!("database migrations failed: {error}"))
}

pub fn find_user_by_email(
    connection: &mut SqliteConnection,
    email: &str,
) -> QueryResult<Option<User>> {
    users::table
        .filter(users::email.eq(email))
        .select(User::as_select())
        .first(connection)
        .optional()
}

pub fn find_user_by_id(connection: &mut SqliteConnection, id: &str) -> QueryResult<Option<User>> {
    users::table
        .filter(users::id.eq(id))
        .select(User::as_select())
        .first(connection)
        .optional()
}

pub fn insert_user(connection: &mut SqliteConnection, user: &NewUser<'_>) -> QueryResult<usize> {
    diesel::insert_into(users::table)
        .values(user)
        .execute(connection)
}

pub fn insert_session(
    connection: &mut SqliteConnection,
    session: &NewSession<'_>,
) -> QueryResult<usize> {
    diesel::insert_into(sessions::table)
        .values(session)
        .execute(connection)
}

pub fn find_session_by_token_hash(
    connection: &mut SqliteConnection,
    token_hash: &str,
) -> QueryResult<Option<(String, String, String)>> {
    sessions::table
        .filter(sessions::token_hash.eq(token_hash))
        .select((sessions::id, sessions::user_id, sessions::expires_at))
        .first(connection)
        .optional()
}

pub fn delete_session_by_token_hash(
    connection: &mut SqliteConnection,
    token_hash: &str,
) -> QueryResult<usize> {
    diesel::delete(sessions::table.filter(sessions::token_hash.eq(token_hash))).execute(connection)
}

pub fn delete_expired_sessions(connection: &mut SqliteConnection, now: &str) -> QueryResult<usize> {
    diesel::delete(sessions::table.filter(sessions::expires_at.lt(now))).execute(connection)
}
