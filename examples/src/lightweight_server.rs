use autd3_link_soem::{Status, SOEM};
use autd3_protobuf::{lightweight::LightweightServer, *};

use tokio::sync::mpsc;
use tonic::transport::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = LightweightServer::new(|| {
        SOEM::builder()
            .with_timeout(std::time::Duration::from_millis(200))
            .with_err_handler(|slave, status| match status {
                Status::Error(msg) => eprintln!("Error [{}]: {}", slave, msg),
                Status::Lost(msg) => {
                    eprintln!("Lost [{}]: {}", slave, msg);
                    std::process::exit(-1);
                }
                Status::StateChanged(msg) => eprintln!("StateChanged [{}]: {}", slave, msg),
            })
    });

    let (tx, mut rx) = mpsc::channel(1);
    ctrlc_async::set_async_handler(async move {
        let _ = tx.send(()).await;
    })
    .expect("Error setting Ctrl-C handler");

    println!("Starting server...");
    println!("Wainting client to connect...");
    println!("Press Ctrl-C to shutdown the server.");
    Server::builder()
        .add_service(ecat_light_server::EcatLightServer::new(server))
        .serve_with_shutdown(format!("0.0.0.0:{}", 8080).parse()?, async {
            let _ = rx.recv().await;
        })
        .await?;

    Ok(())
}
