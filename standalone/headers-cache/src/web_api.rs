use std::sync::Mutex;

use rocket::response::status::NotFound;
use rocket::State;
use rocket::{get, routes};

use crate::db::CacheDB;
use crate::BlockNumber;

struct App {
    db: Mutex<CacheDB>,
}

#[get("/genesis/<block_number>")]
fn get_genesis(app: State<App>, block_number: BlockNumber) -> Result<Vec<u8>, NotFound<String>> {
    app.db
        .lock()
        .unwrap()
        .get_genesis(block_number)
        .ok_or(NotFound(format!("genesis not found")))
}

#[get("/header/<block_number>")]
fn get_header(app: State<App>, block_number: BlockNumber) -> Result<Vec<u8>, NotFound<String>> {
    let key = block_number.to_be_bytes();
    app.db
        .lock()
        .unwrap()
        .get(&key)
        .ok_or(NotFound(format!("header not found")))
}

pub(crate) fn serve(db: &str) -> anyhow::Result<()> {
    rocket::ignite()
        .manage(App {
            db: Mutex::new(CacheDB::open(db)?),
        })
        .mount("/", routes![get_genesis, get_header,])
        .launch();
    Ok(())
}
