use crate::api::ApiError::{PoolNotFound, WriteFailed};
use crate::api::OkResponse;
use crate::cli::{AccountType, ConfigCliArgs, ConfigCommands};
use crate::db;
use crate::db::{
    add_worker, get_all_pools, get_all_pools_with_workers, get_pool_by_pid,
    get_pool_by_pid_with_workers, get_worker_by_name, remove_worker, setup_inventory_db,
    update_worker, WrappedDb,
};
use crate::tx::{get_options, PoolOperator, PoolOperatorAccess, PoolOperatorForSerialize, DB};
use anyhow::{anyhow, Context, Result};
use schnorrkel::SecretKey;
use sp_core::crypto::{AccountId32, Ss58Codec};
use sp_core::sr25519::Pair as Sr22519Pair;
use sp_core::Pair;
use std::path::Path;
use std::sync::Arc;

pub async fn cli_main(args: ConfigCliArgs) -> Result<()> {
    let db = setup_inventory_db(&args.db_path);
    let po_db = get_options(None);
    let po_db = DB::open(&po_db, Path::new(&args.db_path).join("po"))?;

    match &args.command {
        ConfigCommands::AddPool { pid, .. } => {
            db::add_pool(db.clone(), args.command.clone())?;
            let p = get_pool_by_pid(db.clone(), *pid)?;
            if let Some(p) = p {
                let p = serde_json::to_string_pretty(&p)?;
                println!("{p}");
            }
        }
        ConfigCommands::RemovePool { pid } => {
            db::remove_pool(db.clone(), *pid)?;
        }
        ConfigCommands::UpdatePool { pid, .. } => {
            db::update_pool(db.clone(), args.command.clone())?;
            let p = get_pool_by_pid(db.clone(), *pid)?;
            if let Some(p) = p {
                let p = serde_json::to_string_pretty(&p)?;
                println!("{p}");
            }
        }
        ConfigCommands::GetPool { pid } => {
            let p = get_pool_by_pid(db, *pid)?;
            if let Some(p) = p {
                let p = serde_json::to_string_pretty(&p)?;
                println!("{p}");
            }
        }
        ConfigCommands::GetPoolWithWorkers { pid } => {
            let p = get_pool_by_pid_with_workers(db, *pid)?;
            if let Some(p) = p {
                let p = serde_json::to_string_pretty(&p)?;
                println!("{p}");
            }
        }
        ConfigCommands::GetAllPools => {
            let v = get_all_pools(db)?;
            let v = serde_json::to_string_pretty(&v)?;
            println!("{v}");
        }
        ConfigCommands::GetAllPoolsWithWorkers => {
            let v = get_all_pools_with_workers(db)?;
            let v = serde_json::to_string_pretty(&v)?;
            println!("{v}");
        }
        ConfigCommands::AddWorker { name, pid, .. } => {
            add_worker(db.clone(), args.command.clone())?;
            let mut v =
                get_worker_by_name(db.clone(), name.to_string())?.context("Failed to add!")?;
            v.pid = Some(*pid);
            let v = serde_json::to_string_pretty(&v)?;
            println!("{v}");
        }
        ConfigCommands::UpdateWorker {
            name,
            new_name,
            pid,
            ..
        } => {
            update_worker(db.clone(), args.command.clone())?;
            let new_name = match new_name {
                None => name.to_string(),
                Some(nn) => nn.to_string(),
            };
            let mut v = get_worker_by_name(db.clone(), new_name)?.context("Failed to add!")?;
            v.pid = Some(*pid);
            let v = serde_json::to_string_pretty(&v)?;
            println!("{v}");
        }
        ConfigCommands::RemoveWorker { name } => {
            remove_worker(db, name.clone())?;
        }
        ConfigCommands::GetAllPoolOperators => {
            let l = po_db.get_all_po()?;
            let l = l
                .iter()
                .map(|i| i.into())
                .collect::<Vec<PoolOperatorForSerialize>>();
            let l = serde_json::to_string_pretty(&l)?;
            println!("{l}");
        }
        ConfigCommands::GetPoolOperator { pid } => {
            let pid = *pid;
            let po = po_db.get_po(pid)?;
            if let Some(po) = po {
                let po = serde_json::to_string_pretty::<PoolOperatorForSerialize>(&(&po).into())?;
                println!("{po}");
            } else {
                return Err(anyhow!("Record not found!"));
            };
        }
        ConfigCommands::SetPoolOperator {
            pid,
            account,
            proxied_account_id,
            account_type,
        } => {
            let pid = *pid;
            let po = PoolOperator {
                pid,
                pair: match account_type {
                    AccountType::Seed => Sr22519Pair::from_string(account, None)?,
                    AccountType::SecretKey => {
                        let bytes = hex::decode(account)?;
                        let bytes = bytes.as_slice();
                        let key = SecretKey::from_ed25519_bytes(bytes)
                            .map_err(|e| anyhow!(e.to_string()))?;
                        Sr22519Pair::from(key)
                    }
                },
                proxied: match proxied_account_id.clone() {
                    None => None,
                    Some(i) => Some(AccountId32::from_string(&i)?),
                },
            };
            let po = po_db.set_po(pid, po)?;
            let po = serde_json::to_string_pretty::<PoolOperatorForSerialize>(&(&po).into())?;
            println!("{po}");
        }
    };
    Ok(())
}

pub async fn api_handler(db: WrappedDb, po_db: Arc<DB>, command: ConfigCommands) -> Result<String> {
    let ok = OkResponse::default();
    match command.clone() {
        ConfigCommands::AddPool { pid, .. } => {
            db::add_pool(db.clone(), command)?;
            let p = get_pool_by_pid(db.clone(), pid)?;
            if let Some(p) = p {
                let p = serde_json::to_string_pretty(&p)?;
                return Ok(p);
            }
            anyhow::bail!(WriteFailed);
        }
        ConfigCommands::RemovePool { pid } => {
            db::remove_pool(db.clone(), pid)?;
            Ok(serde_json::to_string_pretty(&ok)?)
        }
        ConfigCommands::UpdatePool { pid, .. } => {
            db::update_pool(db.clone(), command)?;
            let p = get_pool_by_pid(db.clone(), pid)?;
            if let Some(p) = p {
                let p = serde_json::to_string_pretty(&p)?;
                return Ok(p);
            }
            anyhow::bail!(WriteFailed);
        }
        ConfigCommands::GetPool { pid } => {
            let p = get_pool_by_pid(db, pid)?;
            if let Some(p) = p {
                let p = serde_json::to_string_pretty(&p)?;
                return Ok(p);
            }
            anyhow::bail!(PoolNotFound(pid))
        }
        ConfigCommands::GetPoolWithWorkers { pid } => {
            let p = get_pool_by_pid_with_workers(db, pid)?;
            if let Some(p) = p {
                let p = serde_json::to_string_pretty(&p)?;
                return Ok(p);
            }
            anyhow::bail!(PoolNotFound(pid))
        }
        ConfigCommands::GetAllPools => {
            let v = get_all_pools(db)?;
            let v = serde_json::to_string_pretty(&v)?;
            Ok(v)
        }
        ConfigCommands::GetAllPoolsWithWorkers => {
            let v = get_all_pools_with_workers(db)?;
            let v = serde_json::to_string_pretty(&v)?;
            Ok(v)
        }
        ConfigCommands::AddWorker { name, pid, .. } => {
            add_worker(db.clone(), command)?;
            let mut v = get_worker_by_name(db.clone(), name)?.context("Failed to add!")?;
            v.pid = Some(pid);
            let v = serde_json::to_string_pretty(&v)?;
            Ok(v)
        }
        ConfigCommands::UpdateWorker {
            name,
            new_name,
            pid,
            ..
        } => {
            update_worker(db.clone(), command)?;
            let new_name = match new_name {
                None => name,
                Some(nn) => nn,
            };
            let mut v = get_worker_by_name(db.clone(), new_name)?.context("Failed to add!")?;
            v.pid = Some(pid);
            let v = serde_json::to_string_pretty(&v)?;
            Ok(v)
        }
        ConfigCommands::RemoveWorker { name } => {
            remove_worker(db, name)?;
            Ok(serde_json::to_string_pretty(&ok)?)
        }
        ConfigCommands::GetAllPoolOperators => {
            let l = po_db.get_all_po()?;
            let l = l
                .iter()
                .map(|i| i.into())
                .collect::<Vec<PoolOperatorForSerialize>>();
            let l = serde_json::to_string_pretty(&l)?;
            Ok(l)
        }
        ConfigCommands::GetPoolOperator { pid } => {
            let po = po_db.get_po(pid)?;
            if let Some(po) = po {
                Ok(serde_json::to_string_pretty::<PoolOperatorForSerialize>(&(&po).into())?)
            } else {
                Err(anyhow!("Record not found!"))
            }
        }
        ConfigCommands::SetPoolOperator {
            pid,
            account,
            proxied_account_id,
            account_type,
        } => {
            let po = PoolOperator {
                pid,
                pair: match account_type {
                    AccountType::Seed => Sr22519Pair::from_string(account.as_str(), None)?,
                    AccountType::SecretKey => {
                        let bytes = hex::decode(account)?;
                        let bytes = bytes.as_slice();
                        let key = SecretKey::from_ed25519_bytes(bytes)
                            .map_err(|e| anyhow!(e.to_string()))?;
                        Sr22519Pair::from(key)
                    }
                },
                proxied: match proxied_account_id {
                    None => None,
                    Some(i) => Some(AccountId32::from_string(&i)?),
                },
            };
            let po = po_db.set_po(pid, po)?;
            let po = serde_json::to_string_pretty::<PoolOperatorForSerialize>(&(&po).into())?;
            Ok(po)
        }
    }
}
