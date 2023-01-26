use anzen_lib::anzen;
use tokio::sync::watch;
use tonic::Streaming;

pub async fn listen_shutdown(
    mut shutdown: anzen_lib::shutdown::Shutdown,
    shutdown_tx: watch::Sender<bool>,
    mut stream: Streaming<anzen::CommandResponse>,
)
{
    while let Some(data) = stream.message().await.unwrap() {
        if shutdown.is_shudown() {
            return;
        }
        let command_is_shutdown = match data.command {
            Some(c) => c.command_type == anzen_lib::anzen::CommandType::Shutdown as i32,
            None => continue,
        };

        if command_is_shutdown {
            shutdown_tx.send(true).unwrap();
        }
    }
}
