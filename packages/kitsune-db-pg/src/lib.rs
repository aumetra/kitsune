use diesel::Connection;
use diesel_async::{
    AsyncPgConnection,
    async_connection_wrapper::AsyncConnectionWrapper,
    pooled_connection::{
        AsyncDieselConnectionManager,
        bb8::{self, Pool},
    },
};
use diesel_migrations::{EmbeddedMigrations, MigrationHarness, embed_migrations};

pub type PgPool = bb8::Pool<AsyncPgConnection>;

const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub mod repository;

mod schema;

pub async fn connect(db_url: &str) -> eyre::Result<PgPool> {
    let config = AsyncDieselConnectionManager::new(db_url);
    let pool = Pool::builder().build(config).await?;

    {
        let db_url = db_url.to_string();
        tokio::task::spawn_blocking(move || {
            let mut conn = AsyncConnectionWrapper::<AsyncPgConnection>::establish(&db_url)?;
            conn.run_pending_migrations(MIGRATIONS)
                .map_err(eyre::Report::msg)?;

            eyre::Ok(())
        })
        .await??;
    }

    Ok(pool)
}
