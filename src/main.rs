mod entities;
use dotenvy::dotenv;
use entities::{prelude::*, *};
use futures::executor::block_on;
use sea_orm::*;
use std::env;

async fn run() -> Result<(), DbErr> {
    dotenv().ok();
    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");

    let db: sea_orm::DatabaseConnection = Database::connect(&database_url).await?;
    let db = &match db.get_database_backend() {
        DbBackend::MySql => {
            let url = format!("{}", &database_url);
            Database::connect(&url).await?
        }
        DbBackend::Postgres => {
            let url = format!("{}", &database_url);
            Database::connect(&url).await?
        }
        DbBackend::Sqlite => db,
    };

    // Insert Bakery
    let happy_bakery = bakery::ActiveModel {
        name: ActiveValue::Set("Happy Bakery".to_owned()),
        profit_margin: ActiveValue::Set(0.0),
        ..Default::default()
    };
    let res = Bakery::insert(happy_bakery).exec(db).await?;

    // Update Bakery
    let sad_bakery = bakery::ActiveModel {
        id: ActiveValue::Set(res.last_insert_id),
        name: ActiveValue::Set("Sad Bakery".to_owned()),
        profit_margin: ActiveValue::NotSet,
    };
    sad_bakery.update(db).await?;

    // Insert Chef
    let john = chef::ActiveModel {
        name: ActiveValue::Set("John".to_owned()),
        bakery_id: ActiveValue::Set(res.last_insert_id),
        ..Default::default()
    };
    let chef_res = Chef::insert(john).exec(db).await?;

    // Select all bakeries
    let bakeries: Vec<bakery::Model> = Bakery::find().all(db).await?;
    assert_eq!(bakeries.last().unwrap().name, "Sad Bakery");

    // Select bakery by id
    let sad_bakery: Option<bakery::Model> = Bakery::find_by_id(res.last_insert_id).one(db).await?;
    assert_eq!(sad_bakery.unwrap().name, "Sad Bakery");

    // Selectby arbitrary column with filter()
    let sad_bakery: Option<bakery::Model> = Bakery::find()
        .filter(bakery::Column::Name.eq("Sad Bakery"))
        .one(db)
        .await?;
    assert!(sad_bakery.is_some());

    // Delete
    let john = chef::ActiveModel {
        id: ActiveValue::Set(chef_res.last_insert_id),
        ..Default::default()
    };
    john.delete(db).await?;

    let sad_bakery = bakery::ActiveModel {
        id: ActiveValue::Set(res.last_insert_id),
        ..Default::default()
    };
    sad_bakery.delete(db).await?;

    let bakeries: Vec<bakery::Model> = Bakery::find().all(db).await?;
    assert!(bakeries.is_empty());

    Ok(())
}

fn main() {
    if let Err(err) = block_on(run()) {
        panic!("{}", err);
    }
}
