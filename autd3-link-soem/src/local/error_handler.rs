use std::sync::{
    atomic::{AtomicBool, AtomicI32, Ordering},
    Arc,
};

use crate::local::soem_bindings::*;

#[derive(Debug, Clone, PartialEq)]
#[repr(u8)]
pub enum Status {
    Error,
    Lost,
    StateChanged,
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Status::Error => write!(f, "slave is in SAFE_OP + ERROR, attempting ack"),
            Status::Lost => write!(f, "slave is lost"),
            Status::StateChanged => write!(f, "slave is in SAFE_OP, change to OPERATIONAL"),
        }
    }
}

pub type ErrHandler = Box<dyn Fn(usize, Status) + Send + Sync>;

pub struct EcatErrorHandler<F: Fn(usize, Status)> {
    pub(crate) err_handler: Option<F>,
}

impl<F: Fn(usize, Status)> EcatErrorHandler<F> {
    pub(crate) fn run(
        &self,
        is_open: Arc<AtomicBool>,
        wkc: Arc<AtomicI32>,
        expected_wkc: i32,
        state_check_interval: std::time::Duration,
    ) {
        unsafe {
            while is_open.load(Ordering::Acquire) {
                if wkc.load(Ordering::Relaxed) < expected_wkc || ec_group[0].docheckstate != 0 {
                    self.handle();
                }
                std::thread::sleep(state_check_interval);
            }
        }
    }

    fn handle(&self) -> bool {
        unsafe {
            ec_group[0].docheckstate = 0;
            ec_readstate();
            ec_slave
                .iter_mut()
                .enumerate()
                .skip(1)
                .take(ec_slavecount as usize)
                .for_each(|(i, slave)| {
                    if slave.state != ec_state_EC_STATE_OPERATIONAL as u16 {
                        ec_group[0].docheckstate = 1;
                        if slave.state
                            == ec_state_EC_STATE_SAFE_OP as u16 + ec_state_EC_STATE_ERROR as u16
                        {
                            if let Some(f) = &self.err_handler {
                                f(i - 1, Status::Error);
                            }
                            slave.state =
                                ec_state_EC_STATE_SAFE_OP as u16 + ec_state_EC_STATE_ACK as u16;
                            ec_writestate(i as _);
                        } else if slave.state == ec_state_EC_STATE_SAFE_OP as u16 {
                            if let Some(f) = &self.err_handler {
                                f(i - 1, Status::StateChanged);
                            }
                            slave.state = ec_state_EC_STATE_OPERATIONAL as _;
                            ec_writestate(i as _);
                        } else if slave.state > ec_state_EC_STATE_NONE as _ {
                            if ec_reconfig_slave(i as _, 500) != 0 {
                                slave.islost = 0;
                            }
                        } else if slave.islost == 0 {
                            ec_statecheck(
                                i as _,
                                ec_state_EC_STATE_OPERATIONAL as _,
                                EC_TIMEOUTRET as _,
                            );
                            if slave.state == ec_state_EC_STATE_NONE as u16 {
                                slave.islost = 1;
                                if let Some(f) = &self.err_handler {
                                    f(i - 1, Status::Lost);
                                }
                            }
                        }
                    }
                    if slave.islost != 0 {
                        if slave.state == ec_state_EC_STATE_NONE as u16 {
                            if ec_recover_slave(i as _, 500) != 0 {
                                slave.islost = 0;
                            }
                        } else {
                            slave.islost = 0;
                        }
                    }
                });

            if ec_group[0].docheckstate == 0 {
                return true;
            }

            ec_slave
                .iter()
                .skip(1)
                .take(ec_slavecount as usize)
                .all(|slave| slave.islost == 0)
        }
    }
}
