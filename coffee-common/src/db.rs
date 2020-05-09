// datastructures to be used with sqlx and helper functions.

use crypto::digest::Digest;
use crypto::sha1::Sha1;
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

#[derive(sqlx::FromRow, Debug)]
pub struct User {
    pub email: String,
    pub apikey: String,
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

    // Registers a user and returns the API key for that email address. If the
    // email already has an API key then that key will be returned.
    // i.e. There is a 1:1 mapping of email to api-key.
    pub async fn register_user(&self, email: &str) -> Result<User, DbError> {
        let user = sqlx::query_as::<_, User>("SELECT email, apikey FROM USERS WHERE email = ?;")
            .bind(email)
            .fetch_optional(&self.pool)
            .await?;
        match user {
            Some(u) => {
                // The user is already registered so return the User struct containing the API key.
                Ok(u)
            }
            None => {
                // The user needs to be registered.
                let mut hasher = Sha1::new();
                hasher.input_str(&email);

                let user = User {
                    email: email.into(),
                    apikey: hasher.result_str(),
                };

                sqlx::query("INSERT INTO USERS(email, apikey) VALUES (?, ?);")
                    .bind(&user.email)
                    .bind(&user.apikey)
                    .execute(&self.pool)
                    .await?;
                Ok(user)
            }
        }
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
