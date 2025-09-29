mod tests;

use autd3::prelude::*;
use autd3_link_ethercrab::{EtherCrab, EtherCrabOption};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let autd = Controller::open(
        [AUTD3 {
            pos: Point3::origin(),
            rot: UnitQuaternion::identity(),
        }; 2],
        EtherCrab::new(
            |idx, status| {
                tracing::info!("Device[{idx}]: {status}");
            },
            EtherCrabOption::default(),
        ),
    )?;

    tests::run(autd)
}
