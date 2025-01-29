use autd3_link_twincat::TwinCAT;
use autd3_protobuf::{lightweight::LightweightServer, *};

use tokio::signal;
use tonic::transport::Server;

#[cfg(windows)]
async fn shutdown_signal() {
    signal::ctrl_c()
        .await
        .expect("failed to install SIGINT handler")
}

#[cfg(unix)]
async fn shutdown_signal() {
    let ctrl_c = async {
        signal::ctrl_c()
            .await
            .expect("failed to install SIGINT handler");
    };

    let terminate = async {
        signal::unix::signal(signal::unix::SignalKind::terminate())
            .expect("failed to install SIGTERM handler")
            .recv()
            .await;
    };

    tokio::select! {
        _ = ctrl_c => {},
        _ = terminate => {},
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::INFO)
        .init();

    let server = LightweightServer::new(TwinCAT::new);

    println!("Starting server...");
    println!("Wainting client to connect...");
    println!("Press Ctrl-C to shutdown the server.");
    let shutdown_signal = shutdown_signal();
    Server::builder()
        .add_service(ecat_light_server::EcatLightServer::new(server))
        .serve_with_shutdown(format!("0.0.0.0:{}", 8080).parse()?, shutdown_signal)
        .await?;

    Ok(())
}
