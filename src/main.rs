#[macro_use]
extern crate rocket;
extern crate argon2;
use anzen_lib::{self, anzen, PluginData};
mod command;
mod config;
mod model;
mod routes;

pub type ResultT<T> = Result<T, Box<dyn std::error::Error>>;

#[tokio::main]
async fn main() -> ResultT<()> {
    // NOTE:
    // - Recieves authorized account names
    // - Recieves JWT key
    // Implement bcrypt

    let register_data = PluginData {
        name: "web-api".into(),
        login_key: "12345".into(),
        plugin_type: anzen::PluginType::Output,
        server_socket: "grpc://[::1]:50000".into(),
    };

    let (client, resp) = anzen_lib::register(&register_data).await.unwrap();

    let config = config::get_config(&resp.plugin_opts)?;

    let token = anzen_lib::get_login_key(&resp.token);

    routes::launch(config, token, register_data.name, client).await?;

    Ok(())
}
