use autd3_link_remote::RemoteServer;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("autd3=info")
        .init();

    RemoteServer::new(8080, autd3::link::Nop::new)
        .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() })
        .run()
        .await?;

    Ok(())
}
