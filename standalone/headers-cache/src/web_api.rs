use rocket::response::status::NotFound;
use rocket::State;
use rocket::{get, routes};

use scale::{Decode, Encode};

use crate::db::CacheDB;
use crate::BlockNumber;

struct App {
    db: CacheDB,
}

#[get("/genesis/<block_number>")]
fn get_genesis(app: &State<App>, block_number: BlockNumber) -> Result<Vec<u8>, NotFound<String>> {
    app.db
        .get_genesis(block_number)
        .ok_or_else(|| NotFound("genesis not found".into()))
}

#[get("/header/<block_number>")]
fn get_header(app: &State<App>, block_number: BlockNumber) -> Result<Vec<u8>, NotFound<String>> {
    app.db
        .get_header(block_number)
        .ok_or_else(|| NotFound("header not found".into()))
}

#[get("/headers/<start>")]
fn get_headers(app: &State<App>, start: BlockNumber) -> Result<Vec<u8>, NotFound<String>> {
    let mut headers = vec![];
    for block in start..start + 10000 {
        match app.db.get_header(block) {
            Some(data) => {
                let info = crate::cache::BlockInfo::decode(&mut &data[..])
                    .map_err(|_| NotFound("Codec error".into()))?;
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

#[get("/parachain-headers/<start>/<count>")]
fn get_parachain_headers(
    app: &State<App>,
    start: BlockNumber,
    count: BlockNumber,
) -> Result<Vec<u8>, NotFound<String>> {
    let mut headers = vec![];
    for block in start..start + count {
        match app.db.get_para_header(block) {
            Some(data) => {
                use pherry::types::Header;
                let header =
                    Header::decode(&mut &data[..]).map_err(|_| NotFound("Codec error".into()))?;
                headers.push(header);
            }
            None => {
                log::warn!("header at {} not found", block);
                return Err(NotFound("header not found".into()));
            }
        }
    }
    log::info!("Got {} parachain headers", headers.len());
    Ok(headers.encode())
}

#[get("/storage-changes/<start>/<count>")]
fn get_storage_changes(
    app: &State<App>,
    start: BlockNumber,
    count: BlockNumber,
) -> Result<Vec<u8>, NotFound<String>> {
    let mut changes = vec![];
    for block in start..start + count {
        match app.db.get_storage_changes(block) {
            Some(data) => {
                let header = crate::cache::BlockHeaderWithChanges::decode(&mut &data[..])
                    .map_err(|_| NotFound("Codec error".into()))?;
                changes.push(header);
            }
            None => {
                log::warn!("changes at {} not found", block);
                return Err(NotFound("header not found".into()));
            }
        }
    }
    log::info!("Got {} storage changes", changes.len());
    Ok(changes.encode())
}

pub(crate) async fn serve(db: &str) -> anyhow::Result<()> {
    let _rocket = rocket::build()
        .manage(App {
            db: CacheDB::open(db)?,
        })
        .mount(
            "/",
            routes![
                get_genesis,
                get_header,
                get_headers,
                get_parachain_headers,
                get_storage_changes,
            ],
        )
        .attach(phala_rocket_middleware::TimeMeter)
        .launch()
        .await?;
    Ok(())
}
