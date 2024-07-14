use std::env;

use dotenvy::dotenv;
use futures::executor::block_on;
use sea_orm::{Database, DbErr};

async fn run() -> Result<(), DbErr> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let db = Database::connect(database_url).await?;

    Ok(())
}

fn main() {
    if let Err(err) = block_on((run())) {
        panic!("{}", err);
    }
}
