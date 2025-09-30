mod tests;

use autd3::prelude::*;
use autd3_link_ethercrab::{EtherCrab, EtherCrabOption};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("autd3-link-ethercrab=info")
        .init();

    let autd = Controller::open(
        [AUTD3 {
            pos: Point3::origin(),
            rot: UnitQuaternion::identity(),
        }],
        EtherCrab::new(
            |idx, status| {
                tracing::info!("Device[{idx}]: {status}");
            },
            EtherCrabOption {
                ..Default::default()
            },
        ),
    )?;

    tests::run(autd)
}
