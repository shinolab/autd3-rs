mod tests;

use autd3::prelude::*;
use autd3_link_ethercrab::{EtherCrab, EtherCrabOption, Status};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_env_filter("autd3=info")
        .init();

    let autd = Controller::open(
        [AUTD3 {
            pos: Point3::origin(),
            rot: UnitQuaternion::identity(),
        }],
        EtherCrab::new(
            |idx, status| {
                eprintln!("Device[{idx}]: {status}");
                if status == Status::Lost {
                    // Currently, recovery from lost is not implemented, so you must either terminate the program or open a new Controller.
                    std::process::exit(1);
                }
            },
            EtherCrabOption::default(),
        ),
    )?;

    tests::run(autd)
}
