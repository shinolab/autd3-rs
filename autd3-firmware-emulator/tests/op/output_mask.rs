use std::collections::HashMap;

use autd3_core::{
    firmware::{Drive, Intensity, Phase, Segment, transition_mode::Immediate},
    link::{MsgId, TxMessage},
};
use autd3_driver::datagram::{OutputMask, WithSegment};
use autd3_firmware_emulator::CPUEmulator;

use rand::*;

use crate::{create_geometry, send};

#[rstest::rstest]
#[case(Segment::S0)]
#[case(Segment::S1)]
#[test]
fn output_mask_unsafe(#[case] segment: Segment) -> Result<(), Box<dyn std::error::Error>> {
    let mut rng = rand::rng();

    let mut geometry = create_geometry(1);
    let mut cpu = CPUEmulator::new(0, geometry.num_transducers());
    let mut tx = vec![TxMessage::new(); 1];
    let mut msg_id = MsgId::new(0);

    let buf: Vec<_> = (0..geometry.num_transducers())
        .map(|_| rng.random())
        .collect();

    assert_eq!(
        Ok(()),
        send(
            &mut msg_id,
            &mut cpu,
            OutputMask::with_segment(|_| |tr| buf[tr.idx()], segment),
            &mut geometry,
            &mut tx
        )
    );
    assert_eq!(buf, cpu.fpga().output_mask(segment));

    assert_eq!(
        Ok(()),
        send(
            &mut msg_id,
            &mut cpu,
            WithSegment {
                inner: crate::op::gain::TestGain {
                    data: HashMap::from([(
                        0,
                        vec![
                            Drive {
                                phase: Phase::ZERO,
                                intensity: Intensity::MAX
                            };
                            geometry.num_transducers()
                        ],
                    )]),
                },
                segment,
                transition_mode: Immediate
            },
            &mut geometry,
            &mut tx
        )
    );
    assert_eq!(
        buf,
        cpu.fpga()
            .drives()
            .into_iter()
            .map(|d| d.intensity == Intensity::MAX)
            .collect::<Vec<_>>()
    );

    Ok(())
}
