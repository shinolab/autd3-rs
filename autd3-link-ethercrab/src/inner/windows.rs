use ethercrab::ReceiveAction;
use ethercrab::{PduRx, PduTx};
use std::io;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use crate::error::EtherCrabError;

pub fn tx_rx_task_blocking<'sto>(
    device: &str,
    mut pdu_tx: PduTx<'sto>,
    mut pdu_rx: PduRx<'sto>,
    running: Arc<AtomicBool>,
) -> Result<(PduTx<'sto>, PduRx<'sto>), EtherCrabError> {
    let waker_impl = Arc::new(super::waker::Waker::new());
    let waker = std::task::Waker::from(Arc::clone(&waker_impl));

    let mut cap = pcap::Capture::from_device(device)?
        .immediate_mode(true)
        .timeout(-1)
        .open()?
        .setnonblock()?;

    let mut sq = pcap::sendqueue::SendQueue::new(32 * 1024)?;

    let mut in_flight = 0usize;

    while running.load(Ordering::Relaxed) {
        pdu_tx.replace_waker(&waker);

        let mut sent_this_iter = 0usize;

        while let Some(frame) = pdu_tx.next_sendable_frame() {
            frame
                .send_blocking(|frame_bytes| {
                    sq.queue(None, frame_bytes)
                        .map_err(|_| ethercrab::error::Error::SendFrame)?;
                    Ok(frame_bytes.len())
                })
                .map_err(std::io::Error::other)?;

            sent_this_iter += 1;
        }

        if sent_this_iter > 0 {
            sq.transmit(&mut cap, pcap::sendqueue::SendSync::Off)?;
            in_flight += sent_this_iter;
        }

        if in_flight > 0 {
            while running.load(Ordering::Relaxed) {
                match cap.next_packet() {
                    Ok(packet) => {
                        let frame_buf = packet.data;

                        let res = pdu_rx.receive_frame(frame_buf).map_err(io::Error::other)?;

                        if res == ReceiveAction::Processed {
                            in_flight -= 1;
                        }
                    }
                    Err(pcap::Error::NoMorePackets) | Err(pcap::Error::TimeoutExpired) => {
                        break;
                    }
                    Err(e) => {
                        return Err(io::Error::other(e).into());
                    }
                }
            }
        } else {
            waker_impl.wait();
            if pdu_tx.should_exit() {
                break;
            }
        }
    }

    Ok((pdu_tx.release(), pdu_rx.release()))
}
