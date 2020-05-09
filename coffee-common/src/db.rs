// datastructures to be used with sqlx and helper functions.

use sqlx::sqlite::{SqlitePool, SqliteQueryAs};

#[derive(Debug)]
pub enum DbError {
    InternalError(sqlx::error::Error),
}

impl std::error::Error for DbError {}

impl std::fmt::Display for DbError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> Result<(), std::fmt::Error> {
        write!(f, "{:?}", self)
    }
}

impl From<sqlx::error::Error> for DbError {
    fn from(e: sqlx::error::Error) -> Self {
        DbError::InternalError(e)
    }
}

impl From<DbError> for tonic::Status {
    fn from(e: DbError) -> Self {
        tonic::Status::internal(format!("Internal database error: #{:?}", e))
    }
}

#[derive(sqlx::FromRow, Debug)]
pub struct Coffee {
    pub id: i32,
    pub user: i32,
    pub shots: i32,
    pub utctime: i64,
}

#[derive(Debug)]
pub struct Db {
    pool: SqlitePool,
}

impl Db {
    pub async fn new(db_file: &str) -> Result<Self, DbError> {
        // TODO: yeahhhhh, we're gonna need a better way to do this...
        let db = Db {
            pool: SqlitePool::new(&format!("sqlite:{}", db_file)).await?,
        };

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS USERS(id INTEGER PRIMARY KEY ASC,
                                                   email TEXT NOT NULL UNIQUE,
                                                   apikey TEXT NOT NULL UNIQUE);",
        )
        .execute(&db.pool)
        .await?;

        sqlx::query(
            "CREATE TABLE IF NOT EXISTS COFFEE(id INTEGER PRIMARY KEY ASC,
                                                    user INTEGER NOT NULL,
                                                    utctime INTEGER NOT NULL,
                                                    shots INTEGER NOT NULL,
                                                    FOREIGN KEY(user) REFERENCES USERS(id));",
        )
        .execute(&db.pool)
        .await?;

        Ok(db)
    }

    pub async fn get_coffees(&self, api_key: &str) -> Result<Vec<Coffee>, DbError> {
        // TODO: Need to take the time as a constraint here for the query, dont want to return everything. But... here we are.

        let res: Vec<Coffee> = sqlx::query_as::<_, Coffee>(
            "SELECT COFFEE.id,
                  COFFEE.user,
                  utctime,
                  shots
                  FROM COFFEE
                  INNER JOIN USERS
                  ON COFFEE.user = USERS.id
                  WHERE USERS.apikey = ?",
        )
        .bind(api_key)
        .fetch_all(&self.pool)
        .await?;

        Ok(res)
    }
}
