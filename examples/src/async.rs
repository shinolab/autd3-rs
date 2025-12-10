use autd3::prelude::*;

// `AsyncController` requires a sleeper that implements `AsyncSleeper`.
struct TokioSleeper;

impl AsyncSleeper for TokioSleeper {
    async fn sleep(&self, duration: core::time::Duration) {
        tokio::time::sleep(duration).await;
    }
}

#[tokio::main(flavor = "current_thread")]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut autd = AsyncController::open_with(
        [AUTD3 {
            pos: Point3::origin(),
            rot: UnitQuaternion::identity(),
        }; 2],
        Nop::new(),
        SenderOption::default(),
        TokioSleeper,
    )
    .await?;

    autd.send(
        (
            Sine {
                freq: 150. * Hz,
                option: Default::default(),
            },
            Focus {
                pos: autd.center() + Vector3::new(0., 0., 150. * mm),
                option: Default::default(),
            },
        ),
        TokioSleeper,
    )
    .await?;

    println!("Press Enter to quit.");
    let mut _s = String::new();
    std::io::stdin().read_line(&mut _s)?;

    autd.close(TokioSleeper).await?;

    Ok(())
}
