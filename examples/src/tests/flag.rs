use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc,
};
use tokio::io::AsyncBufReadExt;

use autd3::{driver::link::Link, prelude::*};

pub async fn flag(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(ReadsFPGAState::new(|_dev| true)).await?;

    println!("press any key to force fan...");
    let mut _s = String::new();
    std::io::stdin().read_line(&mut _s)?;

    autd.send(ForceFan::new(|_dev| true)).await?;

    let fin = Arc::new(AtomicBool::new(false));
    println!("press any key to stop checking FPGA status...");
    let fin_signal = tokio::spawn({
        let fin = fin.clone();
        async move {
            let mut _s = String::new();
            tokio::io::BufReader::new(tokio::io::stdin())
                .read_line(&mut _s)
                .await?;
            fin.store(true, Ordering::Relaxed);
            std::io::Result::Ok(())
        }
    });

    let prompts = ['-', '/', '|', '\\'];
    let mut idx = 0;
    while !fin.load(Ordering::Relaxed) {
        let states = autd.fpga_state().await?;
        println!("{} FPGA Status...", prompts[idx / 1000 % prompts.len()]);
        idx += 1;
        states
            .iter()
            .enumerate()
            .for_each(|(i, state)| match state {
                Some(state) => {
                    println!("\x1b[0K[{}]: thermo = {}", i, state.is_thermal_assert())
                }
                None => {
                    println!("\x1b[0K[{}]: -", i);
                }
            });
        print!("\x1b[{}A", states.len() + 1);
    }
    print!("\x1b[1F\x1b[0J");

    fin_signal.await??;

    autd.send((
        ForceFan::new(|_dev| false),
        ReadsFPGAState::new(|_dev| false),
    ))
    .await?;

    Ok(true)
}
