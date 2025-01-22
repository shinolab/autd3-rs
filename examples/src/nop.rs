mod tests;

use anyhow::Result;

use autd3::prelude::*;

fn main() -> Result<()> {
    tracing_subscriber::fmt()
        .with_max_level(tracing::Level::DEBUG)
        .init();

    let autd = Controller::open(
        [AUTD3 {
            pos: Point3::origin(),
            rot: UnitQuaternion::identity(),
        }; 2],
        Nop::builder(),
    )?;

    tests::run(autd)
}
