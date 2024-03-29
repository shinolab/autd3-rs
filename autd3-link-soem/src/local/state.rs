use std::ffi::CStr;

use super::soem_bindings::*;

#[derive(Debug)]
pub struct State {
    status: String,
    ec_state: u16,
    al_status_code: u16,
}

impl std::fmt::Display for State {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        writeln!(
            f,
            "{} (State={:#02x}, StatusCode={:#04x})",
            self.status, self.ec_state, self.al_status_code
        )
    }
}

#[derive(Debug)]
pub struct EcStatus {
    states: Vec<State>,
}

impl EcStatus {
    pub fn new(n: usize) -> EcStatus {
        unsafe {
            EcStatus {
                states: (1..=n)
                    .map(|slave| {
                        let c_status: &CStr =
                            CStr::from_ptr(ec_ALstatuscode2string(ec_slave[slave].ALstatuscode));
                        let status: &str = c_status.to_str().unwrap_or("Unknown status");
                        State {
                            status: status.to_string(),
                            ec_state: ec_slave[slave].state,
                            al_status_code: ec_slave[slave].ALstatuscode,
                        }
                    })
                    .collect(),
            }
        }
    }

    pub fn states(&self) -> &[State] {
        &self.states
    }
}

impl std::fmt::Display for EcStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.states.iter().enumerate().try_for_each(|(i, state)| {
            writeln!(f, "Slave[{i}]: {state}")?;
            Ok(())
        })
    }
}
