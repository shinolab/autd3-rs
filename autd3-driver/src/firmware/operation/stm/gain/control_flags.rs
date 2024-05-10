use std::fmt;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct GainSTMControlFlags(u8);

bitflags::bitflags! {
    impl GainSTMControlFlags : u8 {
        const NONE       = 0;
        const BEGIN      = 1 << 0;
        const END        = 1 << 1;
        const TRANSITION = 1 << 2;
        const SEGMENT    = 1 << 3;
        const SEND_BIT0  = 1 << 6;
        const SEND_BIT1  = 1 << 7;
    }
}

impl fmt::Display for GainSTMControlFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut flags = Vec::new();
        if self.contains(GainSTMControlFlags::BEGIN) {
            flags.push("BEGIN")
        }
        if self.contains(GainSTMControlFlags::END) {
            flags.push("END")
        }
        if self.contains(GainSTMControlFlags::TRANSITION) {
            flags.push("TRANSITION")
        }
        if self.is_empty() {
            flags.push("NONE")
        }
        write!(
            f,
            "{}",
            flags
                .iter()
                .map(|s| s.to_string())
                .collect::<Vec<_>>()
                .join(" | ")
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_size() {
        assert_eq!(std::mem::size_of::<GainSTMControlFlags>(), 1);
    }

    #[test]
    fn test_fmt() {
        assert_eq!(format!("{}", GainSTMControlFlags::NONE), "NONE");
        assert_eq!(format!("{}", GainSTMControlFlags::BEGIN), "BEGIN");
        assert_eq!(format!("{}", GainSTMControlFlags::END), "END");
        assert_eq!(format!("{}", GainSTMControlFlags::TRANSITION), "TRANSITION");
        assert_eq!(
            format!(
                "{}",
                GainSTMControlFlags::BEGIN
                    | GainSTMControlFlags::END
                    | GainSTMControlFlags::TRANSITION
            ),
            "BEGIN | END | TRANSITION"
        );
    }
}
