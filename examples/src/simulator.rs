mod tests;

use anyhow::Result;

use autd3::prelude::*;
use autd3_link_simulator::Simulator;

fn main() -> Result<()> {
    let autd = Controller::builder([
        AUTD3::new(Point3::origin()),
        AUTD3::new(Point3::new(AUTD3::DEVICE_WIDTH, 0.0, 0.0)),
    ])
    .open(Simulator::builder("127.0.0.1:8080".parse()?))?;

    tests::run(autd)
}
