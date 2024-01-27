use autd3_protobuf::lightweight::LightweightServer;
use autd3_protobuf::*;

use tokio::{runtime::Runtime, sync::mpsc};
use tonic::transport::Server;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let server = LightweightServer::new(|| -> autd3_link_soem::local::link_soem::SOEMBuilder {
        autd3_link_soem::SOEM::builder()
            .with_timeout(std::time::Duration::from_millis(200))
            .with_on_err(|msg| {
                eprintln!("{}", msg);
            })
            .with_on_lost(|msg| {
                eprintln!("{}", msg);
                std::process::exit(-1);
            })
    });

    let (tx, mut rx) = mpsc::channel(1);
    ctrlc::set_handler(move || {
        let rt = Runtime::new().expect("failed to obtain a new Runtime object");
        rt.block_on(tx.send(())).unwrap();
    })
    .expect("Error setting Ctrl-C handler");

    Server::builder()
        .add_service(ecat_light_server::EcatLightServer::new(server))
        .serve_with_shutdown(format!("0.0.0.0:{}", 8080).parse()?, async {
            let _ = rx.recv().await;
        })
        .await?;

    Ok(())
}
