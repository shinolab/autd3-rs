/// Synchronization modes of an EtherCAT slave
/// See [Beckhoff's document](https://infosys.beckhoff.com/english.php?content=../content/1033/ethercatsystem/2469122443.html&id=) for more details.
#[derive(Debug, Clone, Copy, PartialEq)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
#[repr(u8)]
pub enum SyncMode {
    /// DC sync mode
    DC = 0,
    /// Free run mode
    FreeRun = 1,
}
