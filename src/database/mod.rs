use mongodb::sync::{Client, Collection, Database};
use mongodb::bson::doc;
use std::env;

use once_cell::sync::OnceCell;
static DBCONN: OnceCell<Client> = OnceCell::new();

pub fn connect() {
    let client =
        Client::with_uri_str(&env::var("DB_URI").expect("DB_URI not in environment variables!"))
            .expect("Failed to init db connection.");

    client.database("revolt").collection("migrations").find(doc! { }, None).expect("Failed to get migration data from database.");

    DBCONN.set(client).unwrap();
}

pub fn get_connection() -> &'static Client {
    DBCONN.get().unwrap()
}

pub fn get_db() -> Database {
    get_connection().database("revolt")
}

pub fn get_collection(collection: &str) -> Collection {
    get_db().collection(collection)
}

pub mod channel;
pub mod guild;
pub mod message;
pub mod mutual;
pub mod permissions;
pub mod user;

pub use permissions::*;
