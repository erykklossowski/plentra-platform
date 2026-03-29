use sqlx::PgPool;

pub async fn connect(database_url: &str) -> anyhow::Result<PgPool> {
    let pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(5)
        .acquire_timeout(std::time::Duration::from_secs(10))
        .connect(database_url)
        .await?;

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await?;

    tracing::info!("Database connected and migrations applied");

    // Data integrity check — log row counts to detect data loss on deploy
    check_data_integrity(&pool).await;

    Ok(pool)
}

async fn check_data_integrity(pool: &PgPool) {
    let tables = [
        "fuel_daily",
        "reserve_prices_hourly",
        "curtailment_15min",
        "price_hourly",
        "generation_hourly",
        "api_cache",
    ];

    for table in &tables {
        let query = format!("SELECT COUNT(*) as cnt FROM {}", table);
        match sqlx::query_scalar::<_, i64>(&query).fetch_one(pool).await {
            Ok(count) => {
                if count == 0 {
                    tracing::warn!("DATA INTEGRITY: {} is EMPTY — data may have been lost", table);
                } else {
                    tracing::info!("DATA INTEGRITY: {} has {} rows", table, count);
                }
            }
            Err(e) => {
                tracing::error!("DATA INTEGRITY: failed to check {}: {}", table, e);
            }
        }
    }
}
