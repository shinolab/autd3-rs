mod tests;

use anyhow::Result;

use autd3::prelude::*;

fn main() -> Result<()> {
    let autd = Controller::open(
        [AUTD3 {
            pos: Point3::origin(),
            rot: UnitQuaternion::identity(),
        }; 2],
        Nop::new(),
    )?;

    tests::run(autd)
}
