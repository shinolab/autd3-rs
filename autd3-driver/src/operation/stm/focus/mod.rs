mod control_point;
mod focus_stm_op;

pub use control_point::ControlPoint;
pub use focus_stm_op::{FocusSTMChangeSegmentOp, FocusSTMOp};

use std::fmt;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct FocusSTMControlFlags(u8);

bitflags::bitflags! {
    impl FocusSTMControlFlags : u8 {
        const NONE            = 0;
        const BEGIN       = 1 << 0;
        const END         = 1 << 1;
        const UPDATE          = 1 << 2;
    }
}

impl fmt::Display for FocusSTMControlFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut flags = Vec::new();
        if self.contains(FocusSTMControlFlags::BEGIN) {
            flags.push("BEGIN")
        }
        if self.contains(FocusSTMControlFlags::END) {
            flags.push("END")
        }
        if self.contains(FocusSTMControlFlags::UPDATE) {
            flags.push("UPDATE")
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
    fn focus_stm_controll_flag() {
        assert_eq!(std::mem::size_of::<FocusSTMControlFlags>(), 1);

        let flags = FocusSTMControlFlags::BEGIN | FocusSTMControlFlags::END;

        let flagsc = Clone::clone(&flags);
        assert!(flagsc.contains(FocusSTMControlFlags::BEGIN));
        assert!(flagsc.contains(FocusSTMControlFlags::END));
        assert!(!flagsc.contains(FocusSTMControlFlags::UPDATE));
    }

    #[test]
    fn focus_stm_controll_flag_fmt() {
        assert_eq!(format!("{}", FocusSTMControlFlags::NONE), "NONE");
        assert_eq!(format!("{}", FocusSTMControlFlags::BEGIN), "BEGIN");
        assert_eq!(format!("{}", FocusSTMControlFlags::END), "END");
        assert_eq!(format!("{}", FocusSTMControlFlags::UPDATE), "UPDATE");

        assert_eq!(
            format!(
                "{}",
                FocusSTMControlFlags::BEGIN
                    | FocusSTMControlFlags::END
                    | FocusSTMControlFlags::UPDATE
            ),
            "BEGIN | END | UPDATE"
        );
    }
}
