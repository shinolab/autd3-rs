use std::{
    sync::{
        Arc, RwLock,
        atomic::AtomicBool,
        mpsc::{Receiver, SyncSender, sync_channel},
    },
    time::Duration,
};

use crate::{
    error::EtherCrabError,
    inner::{EtherCrabOptionFull, status::Status, utils::PduStorageWrapper},
};

use autd3_core::{
    ethercat::{EC_INPUT_FRAME_SIZE, EC_OUTPUT_FRAME_SIZE},
    link::{LinkError, RxMessage, TxMessage},
};

use ethercrab::{
    DcSync, MainDevice, PduStorage, RegisterAddress, SubDeviceGroup,
    std::ethercat_now,
    subdevice_group::{HasDc, NoDc, Op, PreOp, PreOpPdi, SafeOp},
};
use futures_util::stream::{FuturesOrdered, FuturesUnordered, StreamExt};

#[cfg(not(feature = "tokio"))]
use std::time::Instant;
#[cfg(feature = "tokio")]
use tokio::time::Instant;

async fn sleep(duration: Duration) {
    #[cfg(feature = "tokio")]
    tokio::time::sleep(duration).await;
    #[cfg(not(feature = "tokio"))]
    super::timer::Timer::after(duration).await;
}

async fn sleep_until(deadline: Instant) {
    #[cfg(feature = "tokio")]
    tokio::time::sleep_until(deadline).await;
    #[cfg(not(feature = "tokio"))]
    super::timer::Timer::until(deadline).await;
}

#[cfg(not(feature = "tokio"))]
use std::thread::JoinHandle;
#[cfg(feature = "tokio")]
use tokio::task::JoinHandle;

pub const MAX_SUBDEVICES: usize = 32;
pub const MAX_PDU_DATA: usize =
    PduStorage::element_size((EC_OUTPUT_FRAME_SIZE + EC_INPUT_FRAME_SIZE) * MAX_SUBDEVICES);
pub const MAX_FRAMES: usize = 16;
pub const PDI_LEN: usize = (EC_OUTPUT_FRAME_SIZE + EC_INPUT_FRAME_SIZE) * MAX_SUBDEVICES;

pub struct EtherCrabHandler {
    is_open: Arc<AtomicBool>,
    tx_rx_th: Option<std::thread::JoinHandle<Result<(), EtherCrabError>>>,
    main_th: Option<JoinHandle<()>>,
    state_check_task: Option<JoinHandle<()>>,
    sender: SyncSender<Vec<TxMessage>>,
    buffer_queue_receiver: Receiver<Vec<TxMessage>>,
    inputs: Arc<RwLock<Vec<u8>>>,
    main_device: Option<Arc<MainDevice<'static>>>,
    pdu_storage: PduStorageWrapper,
}

const SUB_GROUP_PDI_LEN: usize = (EC_OUTPUT_FRAME_SIZE + EC_INPUT_FRAME_SIZE) * 2;

struct Groups<S = PreOp, DC = NoDc> {
    groups: Vec<SubDeviceGroup<2, SUB_GROUP_PDI_LEN, S, DC>>,
}

impl Default for Groups<PreOp, NoDc> {
    fn default() -> Self {
        Self {
            groups: Default::default(),
        }
    }
}

impl<S, DC> Groups<S, DC> {
    fn len(&self) -> usize {
        self.groups.iter().map(|g| g.len()).sum()
    }

    async fn transform<S2, DC2, E>(
        self,
        f: impl AsyncFn(
            SubDeviceGroup<2, SUB_GROUP_PDI_LEN, S, DC>,
        ) -> Result<SubDeviceGroup<2, SUB_GROUP_PDI_LEN, S2, DC2>, E>,
    ) -> Result<Groups<S2, DC2>, E> {
        #[allow(clippy::redundant_closure)]
        let mut tasks = self
            .groups
            .into_iter()
            .map(|group| f(group))
            .collect::<FuturesOrdered<_>>();
        let mut groups = Vec::with_capacity(tasks.len());
        while let Some(r) = tasks.next().await {
            groups.push(r?);
        }
        Ok(Groups { groups })
    }
}

impl EtherCrabHandler {
    pub async fn open<F: Fn(usize, Status) + Send + Sync + 'static>(
        err_handler: F,
        geometry: &autd3_core::geometry::Geometry,
        option: EtherCrabOptionFull,
    ) -> Result<EtherCrabHandler, EtherCrabError> {
        #[cfg(feature = "tracing")]
        tracing::debug!("Opening EtherCrab link with option: {:?}", option);

        let interface = match option.ifname.as_ref() {
            Some(ifname) => ifname.clone(),
            None => {
                #[cfg(feature = "tracing")]
                tracing::info!("No interface name is specified. Looking for AUTD device...");
                let ifname = crate::inner::utils::lookup_autd().await?;
                #[cfg(feature = "tracing")]
                tracing::info!("Found EtherCAT device on {:?}", ifname);
                ifname
            }
        };

        let EtherCrabOptionFull {
            ifname: _,
            buf_size,
            timeouts,
            main_device_config,
            dc_configuration,
            sync_tolerance,
            sync_timeout,
            tx_rx_thread_builder,
            #[cfg(feature = "core_affinity")]
            tx_rx_thread_affinity,
            #[cfg(not(feature = "tokio"))]
            main_thread_builder,
            #[cfg(all(not(feature = "tokio"), feature = "core_affinity"))]
            main_thread_affinity,
            state_check_period,
        } = option;

        let pdu_storage = PduStorageWrapper::new();
        let (tx, rx, pdu_loop) = pdu_storage
            .try_split()
            .map_err(|_| EtherCrabError::PduStorageError)?;

        let main_device = Arc::new(MainDevice::new(pdu_loop, timeouts, main_device_config));

        let is_open = Arc::new(AtomicBool::new(true));
        let tx_rx_th = tx_rx_thread_builder.spawn({
            #[cfg(target_os = "windows")]
            let is_open = is_open.clone();
            let interface = interface.clone();
            move |_| {
                #[cfg(feature = "core_affinity")]
                if let Some(affinity) = tx_rx_thread_affinity {
                    if core_affinity::set_for_current(affinity) {
                        #[cfg(feature = "tracing")]
                        tracing::info!("Set CPU affinity of TX/RX thread to core {}", affinity.id);
                    } else {
                        #[cfg(feature = "tracing")]
                        tracing::warn!(
                            "Failed to set CPU affinity of TX/RX thread to core {}",
                            affinity.id
                        );
                    }
                }

                #[cfg(feature = "tracing")]
                tracing::info!("Starting TX/RX task");
                #[cfg(target_os = "windows")]
                let e = crate::inner::windows::tx_rx_task_blocking(&interface, tx, rx, is_open)
                    .map(|_| ());
                #[cfg(not(target_os = "windows"))]
                let e = {
                    match ethercrab::std::tx_rx_task(&interface, tx, rx) {
                        #[cfg(feature = "tokio")]
                        Ok(fut) => tokio::runtime::Builder::new_current_thread()
                            .build()
                            .expect("Failed to build runtime")
                            .block_on(fut)
                            .map_err(EtherCrabError::from),
                        #[cfg(not(feature = "tokio"))]
                        Ok(fut) => super::executor::block_on(fut).map_err(EtherCrabError::from),
                        Err(e) => {
                            #[cfg(feature = "tracing")]
                            tracing::trace!("Failed to start TX/RX task: {}", e);
                            Err(EtherCrabError::from(e))
                        }
                    }
                }
                .map(|_| ());
                #[cfg(feature = "tracing")]
                tracing::debug!("TX/RX task exited: {:?}", e);
                e
            }
        })?;

        // With `init_single_group`, using three or more AUTD3 devices results in transmission frame sizes becoming excessively large, causing errors. Therefore, divide them into groups of two.
        let mut groups = {
            #[derive(Default)]
            struct GroupsArray {
                groups: [SubDeviceGroup<2, SUB_GROUP_PDI_LEN>; MAX_SUBDEVICES / 2],
            }
            let mut idx = 0;
            Groups {
                groups: main_device
                    .init::<MAX_SUBDEVICES, _>(ethercat_now, |group: &GroupsArray, _sub_device| {
                        let g = &group.groups[idx / 2];
                        idx += 1;
                        Ok(g)
                    })
                    .await?
                    .groups
                    .into_iter()
                    .filter(|g| !g.is_empty())
                    .collect(),
            }
        };

        if geometry.len() != groups.len() {
            return Err(EtherCrabError::DeviceNumberMismatch(
                geometry.len(),
                groups.len(),
            ));
        }
        groups
            .groups
            .iter()
            .flat_map(|g| g.iter(&main_device))
            .enumerate()
            .try_for_each(|(_i, sub_device)| {
                if sub_device.name() == "AUTD" {
                    Ok(())
                } else {
                    #[cfg(feature = "tracing")]
                    tracing::error!("Device[{}] is not an AUTD device.", _i);
                    Err(EtherCrabError::DeviceNotFound)
                }
            })?;
        #[cfg(feature = "tracing")]
        tracing::info!(
            "Found {} AUTD3 device{} on {}",
            groups.len(),
            if groups.len() > 1 { "s" } else { "" },
            interface
        );

        groups
            .groups
            .iter_mut()
            .flat_map(|g| g.iter_mut(&main_device))
            .for_each(|mut sub_device| {
                sub_device.set_dc_sync(DcSync::Sync0);
            });

        #[cfg(feature = "tracing")]
        tracing::info!("Moving into PRE-OP with PDI");
        let groups: Groups<PreOpPdi, NoDc> = groups
            .transform(|group: SubDeviceGroup<_, _>| group.into_pre_op_pdi(&main_device))
            .await?;
        #[cfg(feature = "tracing")]
        tracing::info!("Done. PDI available.");

        wait_for_align(&groups, &main_device, sync_tolerance, sync_timeout).await?;

        #[cfg(feature = "tracing")]
        tracing::info!(
            "Configuring Sync0 with cycle time {:?}.",
            dc_configuration.sync0_period
        );
        let groups: Groups<PreOpPdi, HasDc> = groups
            .transform(|group: SubDeviceGroup<_, _, _, _>| {
                group.configure_dc_sync(&main_device, dc_configuration)
            })
            .await?;

        #[cfg(feature = "tracing")]
        tracing::info!("Checking if all devices are in SAFE-OP");
        let groups: Groups<SafeOp, HasDc> = groups
            .transform(|group: SubDeviceGroup<_, _, PreOpPdi, HasDc>| {
                group.into_safe_op(&main_device)
            })
            .await?;
        #[cfg(feature = "tracing")]
        tracing::info!("All devices are in SAFE-OP");

        #[cfg(feature = "tracing")]
        tracing::info!("Setting all devices to OP");
        let groups: Arc<Groups<Op, HasDc>> = Arc::new(
            groups
                .transform(|group: SubDeviceGroup<_, _, SafeOp, HasDc>| {
                    group.request_into_op(&main_device)
                })
                .await?,
        );

        let _op_request = Instant::now();
        let all_op = Arc::new(AtomicBool::new(false));
        let buf_size = buf_size.get();
        let (sender, receiver) = sync_channel(buf_size);
        let (buffer_queue_sender, buffer_queue_receiver) = sync_channel(buf_size);
        (0..buf_size).for_each(|_| {
            let _ = buffer_queue_sender.send(vec![TxMessage::new(); groups.len()]);
        });
        let inputs = Arc::new(RwLock::new(vec![
            0x00u8;
            groups.len() * EC_INPUT_FRAME_SIZE
        ]));
        #[cfg(feature = "tokio")]
        let main_th = tokio::task::spawn({
            let is_open = is_open.clone();
            let all_op = all_op.clone();
            let groups = groups.clone();
            let main_device = main_device.clone();
            let inputs = inputs.clone();
            async move {
                #[cfg(feature = "tracing")]
                tracing::info!("Starting main task");
                send_loop(
                    is_open,
                    all_op,
                    main_device,
                    groups,
                    buffer_queue_sender,
                    inputs,
                    receiver,
                )
                .await;
            }
        });
        #[cfg(not(feature = "tokio"))]
        let main_th = main_thread_builder.spawn({
            #[cfg(feature = "core_affinity")]
            if let Some(affinity) = main_thread_affinity {
                if core_affinity::set_for_current(affinity) {
                    #[cfg(feature = "tracing")]
                    tracing::info!("Set CPU affinity of main thread to {}", affinity.id);
                } else {
                    #[cfg(feature = "tracing")]
                    tracing::warn!(
                        "Failed to set CPU affinity of main thread to {}",
                        affinity.id
                    );
                }
            }
            let is_open = is_open.clone();
            let all_op = all_op.clone();
            let groups = groups.clone();
            let main_device = main_device.clone();
            let inputs = inputs.clone();
            move |_| {
                #[cfg(feature = "tracing")]
                tracing::info!("Starting main task");
                super::executor::block_on(send_loop(
                    is_open,
                    all_op,
                    main_device,
                    groups,
                    buffer_queue_sender,
                    inputs,
                    receiver,
                ));
            }
        })?;

        let run = Arc::new(AtomicBool::new(false));
        #[cfg(feature = "tokio")]
        let state_check_task = tokio::task::spawn({
            let is_open = is_open.clone();
            let main_device = main_device.clone();
            let run = run.clone();
            async move {
                #[cfg(feature = "tracing")]
                tracing::info!("Starting state check task");
                error_handler(
                    is_open,
                    run,
                    main_device,
                    groups,
                    err_handler,
                    state_check_period,
                )
                .await;
            }
        });
        #[cfg(not(feature = "tokio"))]
        let state_check_task = std::thread::spawn({
            let is_open = is_open.clone();
            let main_device = main_device.clone();
            let run = run.clone();
            move || {
                #[cfg(feature = "tracing")]
                tracing::info!("Starting state check task");
                super::executor::block_on(error_handler(
                    is_open,
                    run,
                    main_device,
                    groups,
                    err_handler,
                    state_check_period,
                ));
            }
        });

        let start = Instant::now();
        while !all_op.load(std::sync::atomic::Ordering::Relaxed) {
            if start.elapsed() > timeouts.state_transition {
                return Err(EtherCrabError::NotResponding);
            }
            sleep(Duration::from_millis(10)).await;
        }
        sleep(Duration::from_millis(100)).await;
        #[cfg(feature = "tracing")]
        tracing::info!("All devices entered OP in {:?}", _op_request.elapsed());
        run.store(true, std::sync::atomic::Ordering::Relaxed);

        Ok(EtherCrabHandler {
            is_open,
            tx_rx_th: Some(tx_rx_th),
            main_th: Some(main_th),
            state_check_task: Some(state_check_task),
            sender,
            buffer_queue_receiver,
            inputs,
            main_device: Some(main_device),
            pdu_storage,
        })
    }

    pub async fn close(&mut self) -> Result<(), LinkError> {
        if !self.is_open() {
            return Ok(());
        }

        // TODO: flush sender

        self.is_open
            .store(false, std::sync::atomic::Ordering::Relaxed);

        #[cfg(feature = "tracing")]
        tracing::trace!("Waiting for state check task to finish");
        if let Some(state_check_task) = self.state_check_task.take() {
            #[cfg(feature = "tokio")]
            let _ = state_check_task.await;
            #[cfg(not(feature = "tokio"))]
            let _ = state_check_task.join();
        }

        #[cfg(feature = "tracing")]
        tracing::trace!("Waiting for main task to finish");
        if let Some(task) = self.main_th.take() {
            #[cfg(feature = "tokio")]
            let _ = task.await;
            #[cfg(not(feature = "tokio"))]
            let _ = task.join();
        }

        if let Some(main_device) = self.main_device.take() {
            match Arc::try_unwrap(main_device) {
                Ok(main_device) => {
                    #[cfg(feature = "tracing")]
                    tracing::trace!("Releasing all devices");
                    unsafe {
                        main_device.release_all();
                    }
                }
                Err(_arc) => {
                    #[cfg(feature = "tracing")]
                    tracing::error!(
                        "Cannot release devices because there are {} strong references",
                        Arc::strong_count(&_arc),
                    );
                }
            }
        }

        #[cfg(feature = "tracing")]
        tracing::trace!("Waiting for TX/RX task to finish");
        if let Some(tx_rx_th) = self.tx_rx_th.take() {
            tx_rx_th.thread().unpark();
            let _ = tx_rx_th.join();
        }

        #[cfg(feature = "tracing")]
        tracing::trace!("Releasing PDU storage");
        self.pdu_storage.release();

        Ok(())
    }

    pub fn alloc_tx_buffer(&mut self) -> Result<Vec<TxMessage>, LinkError> {
        self.buffer_queue_receiver
            .recv()
            .map_err(|e| LinkError::new(format!("Failed to allocate TX buffer: {}", e)))
    }

    pub fn send(&mut self, tx: Vec<TxMessage>) -> Result<(), LinkError> {
        self.sender.send(tx).map_err(|_| LinkError::closed())?;
        Ok(())
    }

    pub fn receive(&mut self, rx: &mut [RxMessage]) {
        unsafe {
            std::ptr::copy_nonoverlapping(
                self.inputs.read().unwrap().as_ptr() as *const RxMessage,
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
    group: &Groups<PreOpPdi, NoDc>,
    main_device: &MainDevice<'_>,
    sync_tolerance: Duration,
    sync_timeout: Duration,
) -> Result<(), EtherCrabError> {
    #[cfg(feature = "tracing")]
    tracing::info!("Waiting for devices to align");

    let mut averages = vec![super::smoothing::Smoothing::new(0.2); group.len()];
    let mut now = Instant::now();
    let start = Instant::now();
    loop {
        let mut tasks = group
            .groups
            .iter()
            .map(|g| g.tx_rx_sync_system_time(main_device))
            .collect::<FuturesUnordered<_>>();
        while let Some(r) = tasks.next().await {
            r?;
        }

        if now.elapsed() >= Duration::from_millis(10) {
            now = Instant::now();

            let mut max_deviation = Duration::ZERO;
            for (sub_device, ema) in group
                .groups
                .iter()
                .flat_map(|g| g.iter(main_device))
                .zip(averages.iter_mut())
            {
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

            #[cfg(feature = "tracing")]
            tracing::debug!("Maximum system time difference is {:?}", max_deviation);
            if max_deviation < sync_tolerance {
                #[cfg(feature = "tracing")]
                tracing::info!("Clocks settled after {:?}", start.elapsed());
                break;
            }
            if start.elapsed() > sync_timeout {
                return Err(EtherCrabError::SyncTimeout(max_deviation));
            }
        }
        sleep(Duration::from_millis(1)).await;
    }

    #[cfg(feature = "tracing")]
    tracing::info!("Alignment done");

    Ok(())
}

async fn send_task(
    main_device: &MainDevice<'_>,
    group: &SubDeviceGroup<2, SUB_GROUP_PDI_LEN, Op, HasDc>,
) -> Result<bool, ethercrab::error::Error> {
    let start = Instant::now();
    let resp = group.tx_rx_dc(main_device).await?;
    sleep_until(start + resp.extra.next_cycle_wait).await;
    Ok(resp.all_op())
}

async fn send_loop(
    running: Arc<AtomicBool>,
    all_op: Arc<AtomicBool>,
    main_device: Arc<MainDevice<'_>>,
    group: Arc<Groups<Op, HasDc>>,
    buffer_queue_sender: SyncSender<Vec<TxMessage>>,
    inputs: Arc<RwLock<Vec<u8>>>,
    receiver: Receiver<Vec<TxMessage>>,
) {
    let mut inputs_buf = vec![0u8; group.len() * EC_INPUT_FRAME_SIZE];
    while running.load(std::sync::atomic::Ordering::Relaxed) {
        if let Ok(tx) = receiver.try_recv() {
            group
                .groups
                .iter()
                .flat_map(|g| g.iter(&main_device))
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
            let _ = buffer_queue_sender.send(tx);
        }

        {
            group
                .groups
                .iter()
                .flat_map(|g| g.iter(&main_device))
                .enumerate()
                .for_each(|(idx, sub_device)| {
                    let offset = idx * EC_INPUT_FRAME_SIZE;
                    inputs_buf[offset..offset + EC_INPUT_FRAME_SIZE]
                        .copy_from_slice(&sub_device.inputs_raw());
                });
            inputs.write().unwrap().copy_from_slice(&inputs_buf);
        }

        let mut tasks = group
            .groups
            .iter()
            .map(|g| send_task(&main_device, g))
            .collect::<FuturesUnordered<_>>();
        let mut res = Vec::with_capacity(group.groups.len());
        while let Some(r) = tasks.next().await {
            res.push(r);
        }
        match res.into_iter().collect::<Result<Vec<_>, _>>() {
            Ok(v) => {
                all_op.store(
                    v.into_iter().all(|r| r),
                    std::sync::atomic::Ordering::Relaxed,
                );
            }
            Err(ethercrab::error::Error::WorkingCounter { .. }) => {
                #[cfg(feature = "tracing")]
                tracing::warn!("Working counter error occurred");
                continue;
            }
            Err(_e) => {
                if running.load(std::sync::atomic::Ordering::Relaxed) {
                    #[cfg(feature = "tracing")]
                    tracing::error!("Failed to perform DC synchronized Tx/Rx: {}", _e);
                }
                continue;
            }
        };
    }
}

async fn error_handler<F: Fn(usize, Status) + Send + Sync + 'static>(
    is_open: Arc<AtomicBool>,
    run: Arc<AtomicBool>,
    main_device: Arc<MainDevice<'_>>,
    group: Arc<Groups<Op, HasDc>>,
    err_handler: F,
    interval: Duration,
) {
    use super::ext::{State, SubDeviceExt};

    let mut do_check_state = false;
    while is_open.load(std::sync::atomic::Ordering::Relaxed) {
        let mut all_op = true;
        for (idx, sub_device) in group
            .groups
            .iter()
            .flat_map(|g| g.iter(&main_device))
            .enumerate()
        {
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
                                    expected: _expected,
                                    received: _received,
                                }) => {
                                    #[cfg(feature = "tracing")]
                                    tracing::trace!(
                                        "Write state failed: WorkingCounter {{ expected: {}, received: {} }}",
                                        _expected,
                                        _received
                                    );
                                }
                                Err(_e) => {
                                    #[cfg(feature = "tracing")]
                                    tracing::error!("Write state failed: {:?}", _e);
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
                                    expected: _expected,
                                    received: _received,
                                }) => {
                                    #[cfg(feature = "tracing")]
                                    tracing::trace!(
                                        "Write state failed: WorkingCounter {{ expected: {}, received: {} }}",
                                        _expected,
                                        _received
                                    );
                                }
                                Err(_e) => {
                                    #[cfg(feature = "tracing")]
                                    tracing::error!("Write state failed: {:?}", _e);
                                }
                            }
                        } else {
                            // TODO: reconfigure sub devices
                            #[cfg(feature = "tracing")]
                            tracing::error!("Unknown state: {}", state);
                        }
                    }
                }
                Err(ethercrab::error::Error::WorkingCounter {
                    expected: _expected,
                    received: _received,
                }) => {
                    all_op = false;
                    do_check_state = true;
                    #[cfg(feature = "tracing")]
                    tracing::trace!(
                        "Read state failed: WorkingCounter {{ expected: {}, received: {} }}",
                        _expected,
                        _received
                    );
                }
                Err(_e) => {
                    all_op = false;
                    do_check_state = true;
                    if is_open.load(std::sync::atomic::Ordering::Relaxed) {
                        #[cfg(feature = "tracing")]
                        tracing::error!("Read state failed: {}", _e);
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
        sleep(interval).await;
    }
}
