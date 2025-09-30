use std::{
    sync::{Arc, atomic::AtomicBool},
    time::Duration,
};

use autd3_core::{
    ethercat::{EC_INPUT_FRAME_SIZE, EC_OUTPUT_FRAME_SIZE},
    link::{LinkError, RxMessage, TxMessage},
    sleep::Sleep,
};

use async_channel::{Receiver, Sender, bounded};
use ethercrab::{
    DcSync, MainDevice, PduStorage, RegisterAddress, SubDeviceGroup,
    std::ethercat_now,
    subdevice_group::{HasDc, Op, PreOpPdi},
};
use tokio::sync::watch::{Receiver as WatchReceiver, Sender as WatchSender};

use tokio::{task::JoinHandle, time::Instant};
use zerocopy::FromZeros;

use crate::{
    error::EtherCrabError,
    inner::{EtherCrabOptionFull, status::Status},
};

pub const MAX_SUBDEVICES: usize = 32;
pub const MAX_PDU_DATA: usize =
    PduStorage::element_size((EC_OUTPUT_FRAME_SIZE + EC_INPUT_FRAME_SIZE) * MAX_SUBDEVICES);
pub const MAX_FRAMES: usize = 16;
pub const PDI_LEN: usize = (EC_OUTPUT_FRAME_SIZE + EC_INPUT_FRAME_SIZE) * MAX_SUBDEVICES;

static PDU_STORAGE: PduStorage<MAX_FRAMES, MAX_PDU_DATA> = PduStorage::new();

pub struct EtherCrabHandler {
    is_open: Arc<AtomicBool>,
    tx_rx_th: Option<std::thread::JoinHandle<Result<(), EtherCrabError>>>,
    main_th: Option<std::thread::JoinHandle<()>>,
    state_check_task: Option<JoinHandle<()>>,
    sender: Sender<Vec<TxMessage>>,
    buffer_queue_receiver: Receiver<Vec<TxMessage>>,
    interval: Duration,
    inputs_rx: WatchReceiver<Vec<u8>>,
}

impl EtherCrabHandler {
    pub async fn open<F: Fn(usize, Status) + Send + Sync + 'static, S: Sleep + Send + 'static>(
        err_handler: F,
        geometry: &autd3_core::geometry::Geometry,
        option: EtherCrabOptionFull,
        sleeper: S,
    ) -> Result<EtherCrabHandler, EtherCrabError> {
        tracing::debug!(target: "autd3-link-ethercrab", "Opening EtherCrab link with option: {:?}", option);

        let interface = option.ifname().await?;
        let EtherCrabOptionFull {
            ifname: _,
            buf_size,
            timeouts,
            main_device_config,
            dc_configuration,
            sync_tolerance,
            sync_timeout,
            tx_rx_thread_builder,
            tx_rx_thread_affinity: tx_rx_thread_core,
            main_thread_builder,
            main_thread_affinity: main_thread_core,
            state_check_period,
        } = option;

        let (tx, rx, pdu_loop) = PDU_STORAGE
            .try_split()
            .map_err(|_| EtherCrabError::PduStorageError)?;

        let main_device = Arc::new(MainDevice::new(pdu_loop, timeouts, main_device_config));

        let is_open = Arc::new(AtomicBool::new(true));
        let tx_rx_th = tx_rx_thread_builder.spawn({
                #[cfg(target_os = "windows")]
                let is_open = is_open.clone();
                let interface = interface.clone();
                move |_| {
                    if let Some(tx_rx_core) = tx_rx_thread_core {
                        if core_affinity::set_for_current(tx_rx_core) {
                            tracing::info!(target: "autd3-link-ethercrab", "Set CPU affinity of TX/RX thread to core {}", tx_rx_core.id);
                        } else {
                            tracing::warn!(target: "autd3-link-ethercrab", "Failed to set CPU affinity of TX/RX thread to core {}", tx_rx_core.id);
                        }
                    }

                    tracing::info!(target: "autd3-link-ethercrab", "Starting TX/RX task");
                    #[cfg(target_os = "windows")]
                    let e = crate::inner::windows::tx_rx_task_blocking(&interface, tx, rx, is_open);
                    #[cfg(not(target_os = "windows"))]
                    let e = {
                        match ethercrab::std::tx_rx_task(&interface, tx, rx) {
                            Ok(fut) =>
                                tokio::runtime::Builder::new_current_thread()
                                    .build()
                                    .expect("Create runtime")
                                    .block_on(fut)
                                    .map_err(EtherCrabError::from),
                            Err(e) => {
                                tracing::trace!(target: "autd3-link-ethercrab", "Failed to start TX/RX task: {}", e);
                                Err(EtherCrabError::from(e))
                            }
                        }
                        // let ex = smol::LocalExecutor::new();
                        // match ethercrab::std::tx_rx_task(&interface, tx, rx) {
                        //     Ok(fut) => futures_lite::future::block_on(ex.run(fut)).map_err(EtherCrabError::from),
                        //     Err(e) => {
                        //         tracing::trace!(target: "autd3-link-ethercrab", "Failed to start TX/RX task: {}", e);
                        //         Err(EtherCrabError::from(e))
                        //     }
                        // }
                    };
                    tracing::debug!(target: "autd3-link-ethercrab", "TX/RX task exited");
                    e.map(|_| ())
                }
            })?;

        let mut group = main_device
            .init_single_group::<MAX_SUBDEVICES, PDI_LEN>(ethercat_now)
            .await?;

        if geometry.len() != group.len() {
            return Err(EtherCrabError::DeviceNumberMismatch(
                geometry.len(),
                group.len(),
            ));
        }
        group
            .iter(&main_device)
            .enumerate()
            .try_for_each(|(i, sub_device)| {
                if sub_device.name() == "AUTD" {
                    Ok(())
                } else {
                    tracing::error!(target: "autd3-link-ethercrab", "Device[{}] is not an AUTD device.", i);
                    Err(EtherCrabError::DeviceNotFound)
                }
            })?;
        tracing::info!(target: "autd3-link-ethercrab",
            "Found {} AUTD3 device{} on {}",
            group.len(),
            if group.len() > 1 { "s" } else { "" },
            interface
        );

        group.iter_mut(&main_device).for_each(|mut sub_device| {
            sub_device.set_dc_sync(DcSync::Sync0);
        });

        tracing::info!(target: "autd3-link-ethercrab", "Moving into PRE-OP with PDI");
        let group = group.into_pre_op_pdi(&main_device).await?;
        tracing::info!(target: "autd3-link-ethercrab", "Done. PDI available.");

        wait_for_align(&group, &main_device, sync_tolerance, sync_timeout).await?;

        tracing::info!(target: "autd3-link-ethercrab",
            "Configuring Sync0 with cycle time {:?}.",
            dc_configuration.sync0_period
        );
        let group = group
            .configure_dc_sync(&main_device, dc_configuration)
            .await?;

        tracing::info!(target: "autd3-link-ethercrab", "Checking if all devices are in SAFE-OP");
        let group = group.into_safe_op(&main_device).await?;
        tracing::info!(target: "autd3-link-ethercrab", "All devices are in SAFE-OP");

        tracing::info!(target: "autd3-link-ethercrab", "Setting all devices to OP");
        let group = Arc::new(group.request_into_op(&main_device).await?);
        let op_request = Instant::now();

        let all_op = Arc::new(AtomicBool::new(false));
        let (sender, receiver) = bounded(buf_size);
        let (buffer_queue_sender, buffer_queue_receiver) = bounded(buf_size);
        for _ in 0..buf_size {
            let _ = buffer_queue_sender
                .send(vec![TxMessage::new_zeroed(); group.len()])
                .await;
        }
        let (inputs_tx, inputs_rx) =
            tokio::sync::watch::channel(vec![0x00u8; group.len() * EC_INPUT_FRAME_SIZE]);
        let main_th = main_thread_builder.spawn({
                if let Some(main_thread_core) = main_thread_core {
                    if core_affinity::set_for_current(main_thread_core) {
                        tracing::info!(target: "autd3-link-ethercrab", "Set CPU affinity of main thread to {}", main_thread_core.id);
                    } else {
                        tracing::warn!(target: "autd3-link-ethercrab", "Failed to set CPU affinity of main thread to {}", main_thread_core.id);
                    }
                }
                let is_open = is_open.clone();
                let all_op = all_op.clone();
                let group = group.clone();
                let main_device = main_device.clone();
                move |_| {
                    tokio::runtime::Builder::new_current_thread()
                        .build()
                        .expect("Create runtime")
                        .block_on(async move {
                            tracing::info!(target: "autd3-link-ethercrab", "Starting main task");
                            send_loop(
                                is_open,
                                all_op,
                                main_device,
                                group,
                                buffer_queue_sender,
                                inputs_tx,
                                receiver,
                                sleeper,
                            )
                            .await;
                        });
                }
            })?;

        let run = Arc::new(AtomicBool::new(false));
        let state_check_task = tokio::task::spawn_blocking({
            let is_open = is_open.clone();
            let run = run.clone();
            move || {
                tokio::runtime::Handle::current().block_on(async {
                    error_handler(
                        is_open,
                        run,
                        main_device,
                        group,
                        err_handler,
                        state_check_period,
                    )
                    .await;
                })
            }
        });

        let start = Instant::now();
        while !all_op.load(std::sync::atomic::Ordering::Relaxed) {
            if start.elapsed() > timeouts.state_transition {
                return Err(EtherCrabError::NotResponding);
            }
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
        tracing::info!(target: "autd3-link-ethercrab", "All devices entered OP in {:?}", op_request.elapsed());
        run.store(true, std::sync::atomic::Ordering::Relaxed);

        Ok(EtherCrabHandler {
            is_open,
            tx_rx_th: Some(tx_rx_th),
            main_th: Some(main_th),
            state_check_task: Some(state_check_task),
            sender,
            buffer_queue_receiver,
            interval: dc_configuration.sync0_period,
            inputs_rx,
        })
    }

    pub async fn close(&mut self) -> Result<(), LinkError> {
        if !self.is_open() {
            return Ok(());
        }

        let start = Instant::now();
        while !self.sender.is_empty() {
            if start.elapsed() > Duration::from_secs(5) {
                tracing::warn!(target: "autd3-link-ethercrab", "Timeout while waiting for send buffer to be empty");
                break;
            }
            tokio::time::sleep(self.interval).await;
        }

        self.is_open
            .store(false, std::sync::atomic::Ordering::Relaxed);

        if let Some(tx_rx_th) = self.tx_rx_th.take() {
            #[cfg(target_os = "windows")]
            {
                tx_rx_th.thread().unpark();
                let _ = tx_rx_th.join();
            }
            #[cfg(not(target_os = "windows"))]
            {
                let _ = tx_rx_th;
            }
        }

        if let Some(task) = self.main_th.take() {
            let _ = task.join();
        }

        if let Some(state_check_task) = self.state_check_task.take() {
            let _ = state_check_task.await;
        }

        Ok(())
    }

    pub async fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, async_channel::RecvError> {
        self.buffer_queue_receiver.recv().await
    }

    pub async fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        self.sender
            .send(tx)
            .await
            .map_err(|_| LinkError::closed())?;
        Ok(())
    }

    pub async fn receive(&mut self, rx: &mut [RxMessage]) {
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.inputs_rx.borrow().as_ptr() as *const RxMessage,
                rx.as_mut_ptr(),
                rx.len(),
            );
        }
    }

    pub fn is_open(&self) -> bool {
        self.is_open.load(std::sync::atomic::Ordering::Relaxed)
    }
}

async fn wait_for_align(
    group: &SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, PreOpPdi>,
    main_device: &Arc<MainDevice<'_>>,
    sync_tolerance: Duration,
    sync_timeout: Duration,
) -> Result<(), EtherCrabError> {
    tracing::info!(target: "autd3-link-ethercrab", "Waiting for devices to align");

    let mut averages = vec![super::smoothing::Smoothing::new(0.2); group.len()];
    let mut now = Instant::now();
    let start = Instant::now();
    loop {
        group.tx_rx_sync_system_time(main_device).await?;

        if now.elapsed() >= Duration::from_millis(10) {
            now = Instant::now();

            let mut max_deviation = Duration::ZERO;
            for (sub_device, ema) in group.iter(main_device).zip(averages.iter_mut()) {
                let diff = match sub_device
                    .register_read::<u32>(RegisterAddress::DcSystemTimeDifference)
                    .await
                {
                    Ok(value) => {
                        const MASK: u32 = 0x7FFFFFFF;
                        if value & !MASK != 0 {
                            -((value & MASK) as i32)
                        } else {
                            value as i32
                        }
                    }
                    Err(ethercrab::error::Error::WorkingCounter { .. }) => 0,
                    Err(e) => {
                        return Err(e.into());
                    }
                };
                let diff = Duration::from_nanos(ema.push(diff as _).abs() as _);
                max_deviation = max_deviation.max(diff);
            }

            tracing::debug!(target: "autd3-link-ethercrab", "Maximum system time difference is {:?}", max_deviation);
            if max_deviation < sync_tolerance {
                tracing::info!(target: "autd3-link-ethercrab", "Clocks settled after {:?}", start.elapsed());
                break;
            }
            if start.elapsed() > sync_timeout {
                return Err(EtherCrabError::SyncTimeout(max_deviation));
            }
        }
        #[cfg(target_os = "windows")]
        std::thread::sleep(Duration::from_millis(1));
        #[cfg(not(target_os = "windows"))]
        tokio::time::sleep(Duration::from_millis(1)).await;
    }
    tracing::info!(target: "autd3-link-ethercrab", "Alignment done");

    Ok(())
}

#[allow(clippy::too_many_arguments)]
async fn send_loop<S: Sleep>(
    running: Arc<AtomicBool>,
    all_op: Arc<AtomicBool>,
    main_device: Arc<MainDevice<'_>>,
    group: Arc<SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, Op, HasDc>>,
    buffer_queue_sender: Sender<Vec<TxMessage>>,
    inputs_tx: WatchSender<Vec<u8>>,
    receiver: Receiver<Vec<TxMessage>>,
    sleeper: S,
) {
    let mut inputs_buf = vec![0u8; group.len() * EC_INPUT_FRAME_SIZE];
    while running.load(std::sync::atomic::Ordering::Relaxed) {
        let start = Instant::now();
        let dctime = match group.tx_rx_dc(&main_device).await {
            Ok(resp) => {
                all_op.store(resp.all_op(), std::sync::atomic::Ordering::Relaxed);
                {
                    group
                        .iter(&main_device)
                        .enumerate()
                        .for_each(|(idx, sub_device)| {
                            let offset = idx * EC_INPUT_FRAME_SIZE;
                            inputs_buf[offset..offset + EC_INPUT_FRAME_SIZE]
                                .copy_from_slice(&sub_device.inputs_raw());
                        });
                    inputs_tx.send_modify(|v| {
                        v.copy_from_slice(&inputs_buf);
                    });
                }
                start + resp.extra.next_cycle_wait
            }
            Err(ethercrab::error::Error::WorkingCounter { .. }) => {
                tracing::warn!(target: "autd3-link-ethercrab", "Working counter error occurred");
                continue;
            }
            Err(e) => {
                if running.load(std::sync::atomic::Ordering::Relaxed) {
                    tracing::error!(target: "autd3-link-ethercrab", "Failed to perform DC synchronized Tx/Rx: {}", e);
                }
                continue;
            }
        };

        if let Ok(tx) = receiver.try_recv() {
            group
                .iter(&main_device)
                .enumerate()
                .for_each(|(idx, sub_device)| {
                    let mut o = sub_device.outputs_raw_mut();
                    unsafe {
                        std::ptr::copy_nonoverlapping(
                            tx.as_ptr().add(idx) as *const u8,
                            o.as_mut_ptr(),
                            std::mem::size_of::<TxMessage>(),
                        );
                    }
                });
            let _ = buffer_queue_sender.send(tx).await;
        }

        sleeper.sleep(dctime.saturating_duration_since(Instant::now()));
    }
}

async fn error_handler<F: Fn(usize, Status) + Send + Sync + 'static>(
    is_open: Arc<AtomicBool>,
    run: Arc<AtomicBool>,
    main_device: Arc<MainDevice<'_>>,
    group: Arc<SubDeviceGroup<MAX_SUBDEVICES, PDI_LEN, Op, HasDc>>,
    err_handler: F,
    interval: Duration,
) {
    use super::ext::{State, SubDeviceExt};

    let mut do_check_state = false;
    while is_open.load(std::sync::atomic::Ordering::Relaxed) {
        let mut all_op = true;
        for (idx, sub_device) in group.iter(&main_device).enumerate() {
            match sub_device.read_state().await {
                Ok(state) => {
                    if state != State::OPERATIONAL {
                        all_op = false;
                        do_check_state = true;
                        if state.is_safe_op() && state.is_error() {
                            match sub_device
                                .write_state(&main_device, State::SAFE_OP + State::ACK)
                                .await
                            {
                                Ok(_) => {
                                    if run.load(std::sync::atomic::Ordering::Relaxed) {
                                        err_handler(idx + 1, Status::Error);
                                    }
                                }
                                Err(ethercrab::error::Error::WorkingCounter {
                                    expected,
                                    received,
                                }) => {
                                    tracing::trace!(target: "autd3-link-ethercrab",
                                        "Write state failed: WorkingCounter {{ expected: {}, received: {} }}",
                                        expected,
                                        received
                                    );
                                }
                                Err(e) => {
                                    tracing::error!(target: "autd3-link-ethercrab", "Write state failed: {:?}", e);
                                }
                            }
                        } else if state.is_safe_op() {
                            match sub_device
                                .write_state(&main_device, State::OPERATIONAL)
                                .await
                            {
                                Ok(_) => {
                                    if run.load(std::sync::atomic::Ordering::Relaxed) {
                                        err_handler(idx + 1, Status::StateChanged);
                                    }
                                }
                                Err(ethercrab::error::Error::WorkingCounter {
                                    expected,
                                    received,
                                }) => {
                                    tracing::trace!(target: "autd3-link-ethercrab",
                                        "Write state failed: WorkingCounter {{ expected: {}, received: {} }}",
                                        expected,
                                        received
                                    );
                                }
                                Err(e) => {
                                    tracing::error!(target: "autd3-link-ethercrab", "Write state failed: {:?}", e);
                                }
                            }
                        } else {
                            // TODO: reconfigure sub devices
                            tracing::error!(target: "autd3-link-ethercrab", "Unknown state: {}", state);
                        }
                    }
                }
                Err(ethercrab::error::Error::WorkingCounter { expected, received }) => {
                    all_op = false;
                    do_check_state = true;
                    tracing::trace!(target: "autd3-link-ethercrab",
                        "Read state failed: WorkingCounter {{ expected: {}, received: {} }}",
                        expected,
                        received
                    );
                }
                Err(e) => {
                    all_op = false;
                    do_check_state = true;
                    if is_open.load(std::sync::atomic::Ordering::Relaxed) {
                        tracing::error!(target: "autd3-link-ethercrab", "Read state failed: {}", e);
                        continue;
                    } else {
                        break;
                    }
                }
            }
        }
        if do_check_state && all_op {
            do_check_state = false;
            if run.load(std::sync::atomic::Ordering::Relaxed) {
                err_handler(0, Status::Resumed);
            }
        }
        tokio::time::sleep(interval).await;
    }
}
