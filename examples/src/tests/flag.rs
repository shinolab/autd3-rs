use tokio::io::AsyncBufReadExt;

use autd3::{driver::link::Link, prelude::*};

pub async fn flag(autd: &mut Controller<impl Link>) -> anyhow::Result<bool> {
    autd.send(ReadsFPGAState::new(|_dev| true)).await?;

    println!("press any key to force fan...");
    let mut _s = String::new();
    std::io::stdin().read_line(&mut _s)?;

    autd.send(ForceFan::new(|_dev| true)).await?;

    let (tx, mut rx) = tokio::sync::oneshot::channel();
    println!("press any key to stop checking FPGA status...");
    let fin_signal = tokio::spawn(async move {
        let mut _s = String::new();
        tokio::io::BufReader::new(tokio::io::stdin())
            .read_line(&mut _s)
            .await?;
        _ = tx.send(());
        tokio::io::Result::Ok(())
    });

    let prompts = ['-', '/', '|', '\\'];
    let mut idx = 0;
    loop {
        if rx.try_recv().is_ok() {
            break;
        }
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
