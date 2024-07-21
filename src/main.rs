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

    // Relational Select
    let la_boulangerie = bakery::ActiveModel {
        name: ActiveValue::Set("La Boulangerie".to_owned()),
        profit_margin: ActiveValue::Set(0.0),
        ..Default::default()
    };
    let boulan_res = Bakery::insert(la_boulangerie).exec(db).await?;

    let mut chefs: Vec<chef::ActiveModel> = vec![];
    let original_chef_names = ["Charles", "Frederic", "Jolie", "Madeleine"];
    for chef_name in original_chef_names {
        let chef = chef::ActiveModel {
            name: ActiveValue::Set(chef_name.to_owned()),
            bakery_id: ActiveValue::Set(boulan_res.last_insert_id),
            ..Default::default()
        };
        chefs.push(chef);
    }
    Chef::insert_many(chefs).exec(db).await?;

    let la_boulangerie: bakery::Model = Bakery::find_by_id(boulan_res.last_insert_id)
        .one(db)
        .await?
        .unwrap();
    let chefs: Vec<chef::Model> = la_boulangerie.find_related(Chef).all(db).await?;
    let mut chef_names: Vec<String> = chefs.into_iter().map(|b| b.name).collect();
    chef_names.sort_unstable();

    assert_eq!(chef_names, original_chef_names);

    // Use Loader
    let arte_by_padaria = bakery::ActiveModel {
        name: ActiveValue::Set("Arte by Padaria".to_owned()),
        profit_margin: ActiveValue::Set(0.2),
        ..Default::default()
    };
    let padaria_res = Bakery::insert(arte_by_padaria).exec(db).await?;
    let mut padaria_chefs = vec![];
    let original_padaria_chef_names = ["Brian", "Charles", "Kate", "Samantha"];
    for chef_name in original_padaria_chef_names {
        let chef = chef::ActiveModel {
            name: ActiveValue::Set(chef_name.to_owned()),
            bakery_id: ActiveValue::Set(padaria_res.last_insert_id),
            ..Default::default()
        };
        padaria_chefs.push(chef);
    }
    Chef::insert_many(padaria_chefs).exec(db).await?;

    let bakeries: Vec<bakery::Model> = Bakery::find()
        .filter(
            Condition::any()
                .add(bakery::Column::Id.eq(boulan_res.last_insert_id))
                .add(bakery::Column::Id.eq(padaria_res.last_insert_id)),
        )
        .all(db)
        .await?;

    let chefs: Vec<Vec<chef::Model>> = bakeries.load_many(Chef, db).await?;
    let mut boulan_chef_names: Vec<String> =
        chefs[0].to_owned().into_iter().map(|b| b.name).collect();
    boulan_chef_names.sort_unstable();
    let mut padaria_chef_names: Vec<String> =
        chefs[1].to_owned().into_iter().map(|b| b.name).collect();
    padaria_chef_names.sort_unstable();

    assert_eq!(boulan_chef_names, original_chef_names);
    assert_eq!(padaria_chef_names, original_padaria_chef_names);

    // Delete All
    chef::Entity::delete_many().exec(db).await?;
    bakery::Entity::delete_many().exec(db).await?;

    Ok(())
}

fn main() {
    if let Err(err) = block_on(run()) {
        panic!("{}", err);
    }
}
