mod tests;

use std::time::Duration;

use anyhow::Result;

use autd3::prelude::*;
use autd3_link_simulator::Simulator;

fn main() -> Result<()> {
    // use tracing_subscriber::{EnvFilter, fmt, prelude::*};

    // tracing_subscriber::registry()
    //     .with(fmt::layer())
    //     .with(EnvFilter::new("autd3=debug"))
    //     .init();

    let autd = Controller::open_with_option(
        [
            AUTD3 {
                pos: Point3::origin(),
                rot: UnitQuaternion::identity(),
            },
            AUTD3 {
                pos: Point3::new(AUTD3::DEVICE_WIDTH, 0.0, 0.0),
                rot: UnitQuaternion::identity(),
            },
        ],
        Simulator::new("127.0.0.1:8080".parse()?),
        SenderOption {
            timeout: Some(Duration::from_millis(200)),
            ..Default::default()
        },
        SpinSleeper::default(),
    )?;

    tests::run(autd)
}
