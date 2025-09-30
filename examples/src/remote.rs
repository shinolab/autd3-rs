mod tests;

use autd3::prelude::*;
use autd3_link_remote::Remote;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let autd = Controller::open(
        [AUTD3 {
            pos: Point3::origin(),
            rot: UnitQuaternion::identity(),
        }],
        Remote::new("127.0.0.1:8080".parse()?),
    )?;

    tests::run(autd)
}
