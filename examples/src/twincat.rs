mod tests;

use anyhow::Result;

use autd3::prelude::*;
use autd3_link_twincat::TwinCAT;

fn main() -> Result<()> {
    let autd = Controller::open(
        [AUTD3 {
            pos: Point3::origin(),
            rot: UnitQuaternion::identity(),
        }],
        TwinCAT::new()?,
    )?;

    tests::run(autd)
}
