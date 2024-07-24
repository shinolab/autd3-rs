mod tests;

use std::net::{IpAddr, Ipv4Addr, SocketAddr};

use anyhow::Result;

use autd3::prelude::*;
use autd3_link_simulator::Simulator;

#[tokio::main]
async fn main() -> Result<()> {
    let autd = Controller::builder([
        AUTD3::new(Vector3::zeros()),
        AUTD3::new(Vector3::new(AUTD3::DEVICE_WIDTH, 0.0, 0.0)),
    ])
    .open(Simulator::builder(SocketAddr::new(
        IpAddr::V4(Ipv4Addr::LOCALHOST),
        8080,
    )))
    .await?;

    tests::run(autd).await
}
