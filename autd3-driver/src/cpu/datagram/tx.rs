use crate::cpu::{Header, EC_OUTPUT_FRAME_SIZE};

const PAYLOAD_SIZE: usize = EC_OUTPUT_FRAME_SIZE - std::mem::size_of::<Header>();
type Payload = [u8; PAYLOAD_SIZE];

#[repr(C)]
#[derive(Clone)]
pub struct TxMessage {
    pub header: Header,
    pub payload: Payload,
}

#[derive(Clone)]
pub struct TxDatagram {
    data: Vec<TxMessage>,
    num_devices: usize,
}

impl TxDatagram {
    pub fn new(num_devices: usize) -> Self {
        Self {
            num_devices,
            data: vec![
                TxMessage {
                    header: Header {
                        msg_id: 0,
                        _pad: 0,
                        slot_2_offset: 0,
                    },
                    payload: [0; PAYLOAD_SIZE],
                };
                num_devices
            ],
        }
    }

    pub const fn num_devices(&self) -> usize {
        self.num_devices
        // TODO@23.0.0: remove const and replace with this
        // self.data.len()
    }

    pub fn all_data(&self) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(
                self.data.as_ptr() as *const u8,
                self.data.len() * EC_OUTPUT_FRAME_SIZE,
            )
        }
    }

    pub fn all_data_mut(&mut self) -> &mut [u8] {
        unsafe {
            std::slice::from_raw_parts_mut(
                self.data.as_mut_ptr() as *mut u8,
                self.data.len() * EC_OUTPUT_FRAME_SIZE,
            )
        }
    }

    pub fn data(&self, i: usize) -> &[u8] {
        unsafe {
            std::slice::from_raw_parts(&self.data[i] as *const _ as *const u8, EC_OUTPUT_FRAME_SIZE)
        }
    }

    #[deprecated(note = "use indexer and accsess header directly", since = "22.0.2")]
    pub fn headers(&self) -> impl Iterator<Item = &Header> {
        (0..self.num_devices).map(|i| &self[i].header)
    }

    #[deprecated(note = "use indexer and accsess header directly", since = "22.0.2")]
    pub fn header_mut(&mut self, i: usize) -> &mut Header {
        &mut self[i].header
    }

    #[deprecated(note = "use indexer and accsess payload directly", since = "22.0.2")]
    pub fn payload_mut(&mut self, i: usize) -> &mut [u8] {
        &mut self.data[i].payload
    }

    #[deprecated(note = "use indexer and accsess payload directly", since = "22.0.2")]
    pub fn payloads(&self) -> impl Iterator<Item = &[u8]> {
        (0..self.num_devices).map(|i| &self[i].payload[..])
    }
}

impl std::ops::Deref for TxDatagram {
    type Target = [TxMessage];

    fn deref(&self) -> &Self::Target {
        &self.data
    }
}

impl std::ops::DerefMut for TxDatagram {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.data
    }
}

#[cfg(test)]
mod tests {
    use itertools::Itertools;

    use super::*;

    #[rstest::fixture]
    fn tx() -> TxDatagram {
        let mut tx = TxDatagram::new(2);
        tx.all_data_mut().iter_mut().enumerate().for_each(|(i, d)| {
            *d = i as u8;
        });
        tx
    }

    #[rstest::rstest]
    #[test]
    fn test_num_devices(tx: TxDatagram) {
        assert_eq!(2, tx.num_devices());
    }

    #[rstest::rstest]
    #[test]
    fn test_all_data(tx: TxDatagram) {
        assert_eq!(2 * EC_OUTPUT_FRAME_SIZE, tx.all_data().len());
    }

    #[rstest::rstest]
    #[test]
    #[case::device_0((0..EC_OUTPUT_FRAME_SIZE).map(|i| i as u8).collect_vec(), 0)]
    #[case::device_1((EC_OUTPUT_FRAME_SIZE..2*EC_OUTPUT_FRAME_SIZE).map(|i| i as u8).collect_vec(), 1)]
    fn test_data(#[case] expect: Vec<u8>, #[case] dev: usize, tx: TxDatagram) {
        assert_eq!(expect, tx.data(dev));
    }
}
