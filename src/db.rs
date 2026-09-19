use std::path::PathBuf;
use sqlx::{Pool, Sqlite, SqlitePool};
use sqlx::migrate::MigrateDatabase;
use crate::configuration::CONFIGURATION;

pub static DATABASE_CONNECTION: tokio::sync::OnceCell<RuaDatabaseConnection> = tokio::sync::OnceCell::const_new();

pub struct RuaDatabaseConnection {
    pub pool: Pool<Sqlite>,
}

pub async fn get_connection() -> &'static RuaDatabaseConnection {
    DATABASE_CONNECTION
        .get_or_init(async ||
            RuaDatabaseConnection::new().await.unwrap()
        )
        .await
}

impl RuaDatabaseConnection {
    async fn new() -> Result<Self,sqlx::Error> {
        let database_host_uri = &CONFIGURATION.database_host_uri;
        Self::create_database_if_needed(database_host_uri).await?;
        let database_connection = SqlitePool::connect(database_host_uri).await?;
        Self::run_all_migration(&database_connection).await?;

        Ok(RuaDatabaseConnection{ pool: database_connection })
    }

    async fn create_database_if_needed(database_host_uri: &str) -> Result<(), sqlx::Error> {
        if Sqlite::database_exists(database_host_uri).await? {
            // database already exists.
            Ok(())
        } else {
            Sqlite::create_database(database_host_uri).await?;

            Ok(())
        }
    }

    async fn run_all_migration(connection: &SqlitePool) -> Result<(), sqlx::Error> {
        let migration_directory = PathBuf::from("./migrations");

        sqlx::migrate::Migrator::new(migration_directory)
            .await?
            .run(connection)
            .await?;

        Ok(( ))
    }
}