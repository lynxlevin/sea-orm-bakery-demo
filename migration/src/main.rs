use dotenvy::dotenv;
use sea_orm_migration::prelude::*;
use std::env;

#[async_std::main]
async fn main() {
    dotenv().ok();
    let DATABASE_URL = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    cli::run_cli(migration::Migrator).await;
}
