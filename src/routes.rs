use crate::{ResultT, model};

mod auth;
mod data;
mod state;
mod errors;
mod cors;
pub mod returns;

pub async fn launch(config: crate::config::Config) -> ResultT<()>
{
    let validation = state::Validation::init(config.key, config.auth_users);
    let db_state = model::AnzenDB::init(config.db_uri).await?;
    let _ = rocket::build()
        .mount("/api/v1/auth", routes![
            auth::login,
            auth::register,
        ])
        .mount("/api/v1/data", routes![
            data::stats,
            data::test
        ])
        .mount("/", routes![
            cors::resp_options
        ])
        .manage(validation)
        .manage(db_state)
        .attach(cors::CORS)
        .launch()
        .await?;
    Ok(())
}
