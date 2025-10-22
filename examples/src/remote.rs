mod tests;

use autd3::prelude::*;
use autd3_link_remote::{Remote, RemoteOption};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let autd = Controller::open(
        [
            AUTD3 {
                pos: Point3::origin(),
                rot: UnitQuaternion::identity(),
            },
            AUTD3 {
                pos: Point3::new(AUTD3::DEVICE_WIDTH, 0., 0.),
                rot: UnitQuaternion::identity(),
            },
        ],
        Remote::new("127.0.0.1:8080".parse()?, RemoteOption::default()),
    )?;

    tests::run(autd)
}
