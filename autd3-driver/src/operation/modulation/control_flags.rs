use std::fmt;

#[derive(Clone, Copy)]
#[repr(C)]
pub struct ModulationControlFlags(u8);

bitflags::bitflags! {
    impl ModulationControlFlags : u8 {
        const NONE           = 0;
        const MOD_BEGIN      = 1 << 0;
        const MOD_END        = 1 << 1;
        const UPDATE_SEGMENT = 1 << 2;
    }
}

impl fmt::Display for ModulationControlFlags {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut flags = Vec::new();
        if self.contains(ModulationControlFlags::MOD_BEGIN) {
            flags.push("MOD_BEGIN")
        }
        if self.contains(ModulationControlFlags::MOD_END) {
            flags.push("MOD_END")
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
        assert_eq!(std::mem::size_of::<ModulationControlFlags>(), 1);
    }

    #[test]
    fn test_fmt() {
        assert_eq!(format!("{}", ModulationControlFlags::NONE), "NONE");
        assert_eq!(
            format!("{}", ModulationControlFlags::MOD_BEGIN),
            "MOD_BEGIN"
        );
        assert_eq!(format!("{}", ModulationControlFlags::MOD_END), "MOD_END");
        assert_eq!(
            format!(
                "{}",
                ModulationControlFlags::MOD_BEGIN | ModulationControlFlags::MOD_END
            ),
            "MOD_BEGIN | MOD_END"
        );
    }
}
