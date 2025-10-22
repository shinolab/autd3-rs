use autd3_link_remote::RemoteServer;

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    RemoteServer::new(8080, autd3::link::Nop::new())
        .with_graceful_shutdown(async { tokio::signal::ctrl_c().await.unwrap() })
        .run()
        .await?;

    Ok(())
}
