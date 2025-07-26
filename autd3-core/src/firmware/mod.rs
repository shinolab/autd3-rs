mod cpu_gpio;
mod drive;
mod fpga_gpio;
mod intensity;
mod limits;
mod phase;
mod pulse_width;
mod sampling_config;
mod segment;
/// Transition odes for segment switching.
pub mod transition_mode;

pub use cpu_gpio::CpuGPIOPort;
pub use drive::Drive;
pub use fpga_gpio::{GPIOIn, GPIOOut};
pub use intensity::Intensity;
pub use limits::FirmwareLimits;
pub use phase::Phase;
pub use pulse_width::{PulseWidth, PulseWidthError};
pub use sampling_config::{SamplingConfig, SamplingConfigError};
pub use segment::Segment;
