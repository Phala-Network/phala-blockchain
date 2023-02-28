use crate::cli::{ConfigCliArgs, ConfigCommands};
use crate::db;
use crate::db::{
    add_worker, get_all_pools, get_all_pools_with_workers, get_pool_by_pid,
    get_pool_by_pid_with_workers, get_worker_by_name, remove_worker, setup_inventory_db,
    update_worker,
};
use anyhow::{Context, Result};

pub async fn cli_main(args: ConfigCliArgs) -> Result<()> {
    let db = setup_inventory_db(&args.db_path);

    match &args.command {
        ConfigCommands::AddPool { pid, .. } => {
            db::add_pool(db.clone(), args.command.clone())?;
            let p = get_pool_by_pid(db.clone(), *pid)?;
            match p {
                Some(p) => {
                    let p = serde_json::to_string_pretty(&p)?;
                    println!("{}", p);
                }
                None => {}
            }
        }
        ConfigCommands::RemovePool { pid } => {
            db::remove_pool(db.clone(), *pid)?;
        }
        ConfigCommands::UpdatePool { pid, .. } => {
            db::update_pool(db.clone(), args.command.clone())?;
            let p = get_pool_by_pid(db.clone(), *pid)?;
            match p {
                Some(p) => {
                    let p = serde_json::to_string_pretty(&p)?;
                    println!("{}", p);
                }
                None => {}
            }
        }
        ConfigCommands::GetPool { pid } => {
            let p = get_pool_by_pid(db, *pid)?;
            match p {
                Some(p) => {
                    let p = serde_json::to_string_pretty(&p)?;
                    println!("{}", p);
                }
                None => {}
            }
        }
        ConfigCommands::GetPoolWithWorkers { pid } => {
            let p = get_pool_by_pid_with_workers(db, *pid)?;
            match p {
                Some(p) => {
                    let p = serde_json::to_string_pretty(&p)?;
                    println!("{}", p);
                }
                None => {}
            }
        }
        ConfigCommands::GetAllPools => {
            let v = get_all_pools(db)?;
            let v = serde_json::to_string_pretty(&v)?;
            println!("{}", v);
        }
        ConfigCommands::GetAllPoolsWithWorkers => {
            let v = get_all_pools_with_workers(db)?;
            let v = serde_json::to_string_pretty(&v)?;
            println!("{}", v);
        }
        ConfigCommands::AddWorker { name, pid, .. } => {
            add_worker(db.clone(), args.command.clone())?;
            let mut v =
                get_worker_by_name(db.clone(), name.to_string())?.context("Failed to add!")?;
            v.pid = Some(*pid);
            let v = serde_json::to_string_pretty(&v)?;
            println!("{}", v);
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
            println!("{}", v);
        }
        ConfigCommands::RemoveWorker { name } => {
            remove_worker(db, name.clone())?;
        }
    };
    Ok(())
}
