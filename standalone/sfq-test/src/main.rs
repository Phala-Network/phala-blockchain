use std::collections::HashMap;
use std::sync::Mutex;

use clap::Parser;
use fq::RequestScheduler;
use phala_scheduler as fq;
use rocket::http::Status;
use rocket::response::status::Custom;
use rocket::{get, launch, routes, State};

#[derive(Parser, Debug)]
#[clap(about = "SFQ test server", version, author)]
pub struct Args {
    #[clap(short, long, default_value = "32")]
    backlog: usize,
    #[clap(short, long, default_value = "3")]
    depth: u32,
}

#[derive(Debug, Default, Clone)]
struct Stats {
    accepted: u64,
    rejected: u64,
    time: u64,
}

struct App {
    queue: fq::RequestScheduler<String>,
    stats: Mutex<HashMap<String, Stats>>,
}

#[get("/<flow>/<weight>/<cost>")]
async fn test(
    state: &State<App>,
    flow: &str,
    weight: u32,
    cost: u32,
) -> Result<String, Custom<String>> {
    let flow_id = format!("{}/{}", flow, weight);
    let guard = state
        .queue
        .acquire(flow_id.clone(), weight)
        .await
        .map_err(|e| Custom(Status { code: 500 }, format!("{}", e)));
    {
        let mut stat = state.stats.lock().unwrap();
        match &guard {
            Ok(_) => stat.entry(flow_id.clone()).or_default().accepted += 1,
            Err(_) => stat.entry(flow_id.clone()).or_default().rejected += 1,
        }
    }
    let _guard = guard?;
    if cost > 0 {
        std::thread::sleep(std::time::Duration::from_millis(cost as u64));
    }
    state.stats.lock().unwrap().entry(flow_id).or_default().time += cost as u64;
    Ok("OK".to_string())
}

#[get("/dump")]
async fn dump(state: &State<App>) -> String {
    let info = state.queue.dump();

    let mut flow_stats = HashMap::<String, usize>::new();
    for (flow, _cost) in info.backlog.iter() {
        let cnt = flow_stats.get(flow).cloned().unwrap_or_default();
        flow_stats.insert(flow.to_owned(), cnt + 1);
    }

    let mut result = String::default();
    result.push_str(&format!("V time : {}\n", info.virtual_time));
    result.push_str(&format!("Serving: {}\n", info.serving));
    result.push_str(&format!("Backlog: {}\n", info.backlog.len()));
    result.push_str("Flow stats:\n");
    result.push_str(
        "      flow, weight,        v clock,      time used,  avg cost,   backlog,  accepted,  rejected,     total\n"
    );

    let stats = state.stats.lock().unwrap();

    for (flow, avg_cost, clock) in info.flows.iter() {
        let cnt = flow_stats.get(flow).cloned().unwrap_or_default();
        let Stats {
            accepted,
            rejected,
            time,
        } = stats.get(flow).cloned().unwrap_or_default();
        let total = accepted + rejected;
        let weight = flow.split('/').nth(1).unwrap_or("");
        result.push_str(&format!(
            "{flow:>10},{weight:>7},{clock:>15},{time:>15},{avg_cost:>10},{cnt:>10},{accepted:>10},{rejected:>10},{total:>10}\n",
        ));
    }
    result
}

#[launch]
fn rocket() -> _ {
    let args = Args::parse();
    println!("{:#?}", args);

    rocket::build()
        .manage(App {
            queue: RequestScheduler::new(args.backlog, args.depth),
            stats: Default::default(),
        })
        .mount("/test", routes![test, dump])
}
