mod tests;

use anyhow::Result;

use autd3::prelude::*;
use autd3_link_twincat::RemoteTwinCAT;

fn main() -> Result<()> {
    let autd = Controller::open(
        [AUTD3 {
            pos: Point3::origin(),
            rot: UnitQuaternion::identity(),
        }],
        RemoteTwinCAT::new("0.0.0.0.0.0", Default::default()),
    )?;

    tests::run(autd)
}
