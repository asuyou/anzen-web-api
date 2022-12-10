use anzen_lib::ClientRef;

use crate::{model, ResultT};

mod auth;
mod cors;
mod data;
mod errors;
pub mod returns;
mod state;

pub async fn launch(
    config: crate::config::Config,
    token: String,
    name: String,
    client: ClientRef,
) -> ResultT<()> {
    let validation = state::Validation::init(config.key, config.auth_users);
    let core_api = state::CoreAPI::init(token, client, name);
    let db_state = model::AnzenDB::init(config.db_uri).await?;

    let _ = rocket::build()
        .mount("/api/v1/auth", routes![auth::login, auth::register,])
        .mount(
            "/api/v1/data",
            routes![data::stats, data::test, data::toggle],
        )
        .mount("/", routes![cors::resp_options])
        .manage(validation)
        .manage(db_state)
        .manage(core_api)
        .attach(cors::CORS)
        .launch()
        .await?;
    Ok(())
}
