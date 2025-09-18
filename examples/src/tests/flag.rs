use autd3::{core::link::Link, prelude::*};

pub fn flag(
    autd: &mut Controller<impl Link, firmware::Auto>,
) -> Result<(), Box<dyn std::error::Error>> {
    autd.send(ReadsFPGAState::new(|_dev| true))?;

    println!("press any key to force fan...");
    let mut _s = String::new();
    std::io::stdin().read_line(&mut _s)?;

    autd.send(ForceFan::new(|_dev| true))?;

    let (tx, rx) = std::sync::mpsc::channel();
    println!("press any key to stop checking FPGA status...");
    let fin_signal = std::thread::spawn(move || {
        let mut _s = String::new();
        std::io::stdin().read_line(&mut _s)?;
        _ = tx.send(());
        std::io::Result::Ok(())
    });

    let prompts = ['-', '/', '|', '\\'];
    let mut idx = 0;
    loop {
        if rx.try_recv().is_ok() {
            break;
        }
        let states = autd.fpga_state()?;
        println!("{} FPGA Status...", prompts[idx / 1000 % prompts.len()]);
        idx += 1;
        states.iter().enumerate().for_each(|(i, state)| {
            println!(
                "\x1b[0K[{}]: thermo = {}",
                i,
                state.map_or_else(|| "-".to_string(), |s| s.is_thermal_assert().to_string())
            );
        });
        print!("\x1b[{}A", states.len() + 1);
    }
    print!("\x1b[1F\x1b[0J");

    let _ = fin_signal.join();

    autd.send((
        ForceFan::new(|_dev| false),
        ReadsFPGAState::new(|_dev| false),
    ))?;

    Ok(())
}
