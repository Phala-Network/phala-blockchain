use std::sync::Mutex;

use rocket::response::status::NotFound;
use rocket::State;
use rocket::{get, routes};

use scale::{Decode, Encode};

use crate::db::CacheDB;
use crate::BlockNumber;

struct App {
    db: Mutex<CacheDB>,
}

#[get("/genesis/<block_number>")]
fn get_genesis(app: &State<App>, block_number: BlockNumber) -> Result<Vec<u8>, NotFound<String>> {
    app.db
        .lock()
        .unwrap()
        .get_genesis(block_number)
        .ok_or(NotFound(format!("genesis not found")))
}

#[get("/header/<block_number>")]
fn get_header(app: &State<App>, block_number: BlockNumber) -> Result<Vec<u8>, NotFound<String>> {
    app.db
        .lock()
        .unwrap()
        .get_header(block_number)
        .ok_or(NotFound(format!("header not found")))
}

#[get("/headers/<start>")]
fn get_headers(app: &State<App>, start: BlockNumber) -> Result<Vec<u8>, NotFound<String>> {
    let mut headers = vec![];
    let mut db = app.db.lock().unwrap();
    for block in start..start + 10000 {
        match db.get_header(block) {
            Some(data) => {
                let info = crate::cache::BlockInfo::decode(&mut &data[..])
                    .or(Err(NotFound("Codec error".into())))?;
                let end = info.justification.is_some();
                headers.push(info);
                if end {
                    break;
                }
            }
            None => {
                log::warn!("{} not found", block);
                return Err(NotFound("header not found".into()));
            }
        }
    }
    log::info!("Got {} headers", headers.len());
    Ok(headers.encode())
}

#[get("/parachain_headers/<start>/<count>")]
fn get_parachain_headers(
    app: &State<App>,
    start: BlockNumber,
    count: BlockNumber,
) -> Result<Vec<u8>, NotFound<String>> {
    let mut headers = vec![];
    let mut db = app.db.lock().unwrap();
    for block in start..start + count {
        match db.get_para_header(block) {
            Some(data) => {
                use pherry::types::Header;
                let header =
                    Header::decode(&mut &data[..]).or(Err(NotFound("Codec error".into())))?;
                headers.push(header);
            }
            None => {
                log::warn!("{} not found", block);
                return Err(NotFound("header not found".into()));
            }
        }
    }
    log::info!("Got {} parachain headers", headers.len());
    Ok(headers.encode())
}

pub(crate) async fn serve(db: &str) -> anyhow::Result<()> {
    rocket::build()
        .manage(App {
            db: Mutex::new(CacheDB::open(db)?),
        })
        .mount(
            "/",
            routes![get_genesis, get_header, get_headers, get_parachain_headers],
        )
        .launch()
        .await?;
    Ok(())
}
