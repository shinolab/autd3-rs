#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum ProcessPriority {
    Idle = 0,
    BelowNormal = 1,
    Normal = 2,
    AboveNormal = 3,
    High = 4,
    Realtime = 5,
}

impl From<ProcessPriority> for windows::Win32::System::Threading::PROCESS_CREATION_FLAGS {
    fn from(val: ProcessPriority) -> windows::Win32::System::Threading::PROCESS_CREATION_FLAGS {
        match val {
            ProcessPriority::Idle => windows::Win32::System::Threading::IDLE_PRIORITY_CLASS,
            ProcessPriority::BelowNormal => {
                windows::Win32::System::Threading::BELOW_NORMAL_PRIORITY_CLASS
            }
            ProcessPriority::Normal => windows::Win32::System::Threading::NORMAL_PRIORITY_CLASS,
            ProcessPriority::AboveNormal => {
                windows::Win32::System::Threading::ABOVE_NORMAL_PRIORITY_CLASS
            }
            ProcessPriority::High => windows::Win32::System::Threading::HIGH_PRIORITY_CLASS,
            ProcessPriority::Realtime => windows::Win32::System::Threading::REALTIME_PRIORITY_CLASS,
        }
    }
}
