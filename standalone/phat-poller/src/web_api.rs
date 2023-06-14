use crate::app::ArcApp;

use rocket::{get, routes, State};

#[get("/state")]
fn state(app: &State<ArcApp>) -> String {
    serde_json::to_string_pretty(&*app.state.lock().unwrap()).unwrap_or_else(|_| "null".to_string())
}

pub(crate) async fn serve(app: ArcApp) -> anyhow::Result<()> {
    let _rocket = rocket::build()
        .manage(app)
        .mount("/", routes![state,])
        .launch()
        .await?;
    Ok(())
}
