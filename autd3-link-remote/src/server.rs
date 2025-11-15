use std::future::Future;

use autd3_core::{
    geometry::{Geometry, Point3, Quaternion, UnitQuaternion},
    link::{Ack, AsyncLink, LinkError, RxMessage, TxMessage},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select,
};

use crate::{
    MSG_CLOSE, MSG_CONFIG_GEOMETRY, MSG_ERROR, MSG_HELLO, MSG_OK, MSG_READ_DATA, MSG_SEND_DATA,
    MSG_UPDATE_GEOMETRY, REMOTE_PROTOCOL_MAGIC, REMOTE_PROTOCOL_VERSION,
};

/// A server that accepts connections from [`Remote`](crate::Remote) clients and forwards requests to a link.
pub struct RemoteServer<L: AsyncLink, F: Fn() -> L> {
    link_factory: F,
    link: Option<L>,
    port: u16,
    rx_buf: Option<Vec<RxMessage>>,
    num_devices: usize,
    shutdown: Option<Box<dyn Future<Output = ()> + Send + Unpin>>,
    read_buffer: Vec<u8>,
}

impl<L: AsyncLink, F: Fn() -> L> RemoteServer<L, F> {
    /// Create a new [`RemoteServer`].
    ///
    /// # Arguments
    ///
    /// * `port` - The port to listen on
    /// * `link` - A factory function that creates a new link instance
    pub const fn new(port: u16, link_factory: F) -> Self {
        Self {
            link_factory,
            link: None,
            port,
            num_devices: 0,
            rx_buf: None,
            shutdown: None,
            read_buffer: Vec::new(),
        }
    }

    /// Configure graceful shutdown with a custom shutdown signal.
    ///
    /// # Arguments
    ///
    /// * `signal` - A future that completes when the server should shut down
    pub fn with_graceful_shutdown<S>(self, signal: S) -> Self
    where
        S: Future<Output = ()> + Send + 'static,
    {
        Self {
            shutdown: Some(Box::new(Box::pin(signal))),
            ..self
        }
    }

    /// Run the server.
    ///
    /// This method listens for incoming connections asynchronously.
    /// For each connection, it processes requests and forwards them to the link.
    ///
    /// If a shutdown signal is configured via [`with_graceful_shutdown`](Self::with_graceful_shutdown),
    /// the server will gracefully shut down when the signal completes.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - Failed to bind to the specified port
    /// - Failed to accept a connection
    /// - Failed to process a request
    pub async fn run(&mut self) -> Result<(), LinkError> {
        let listener = TcpListener::bind(("0.0.0.0", self.port)).await?;
        tracing::info!("Remote server listening on port {}", self.port);

        if let Some(shutdown) = self.shutdown.take() {
            select! {
                result = self.accept_loop(&listener) => result,
                _ = shutdown => {
                    tracing::info!("Shutdown signal received, stopping server");
                    Ok(())
                },
            }
        } else {
            self.accept_loop(&listener).await
        }
    }

    async fn accept_loop(&mut self, listener: &TcpListener) -> Result<(), LinkError> {
        loop {
            let (stream, _) = listener.accept().await?;
            tracing::info!("Client connected: {:?}", stream.peer_addr()?);
            self.handle_client(stream).await;
            tracing::info!("Client disconnected");
            if let Some(mut link) = self.link.take()
                && let Err(e) = AsyncLink::close(&mut link).await
            {
                tracing::error!("Error closing link: {}", e);
            }
        }
    }

    async fn handle_client(&mut self, mut stream: TcpStream) {
        let mut handshake_completed = false;

        loop {
            let mut msg_type = [0u8; size_of::<u8>()];
            if stream.read_exact(&mut msg_type).await.is_err() {
                break;
            }

            let msg = msg_type[0];
            let result = if msg == MSG_HELLO {
                tracing::info!("Handling handshake...");
                if handshake_completed {
                    tracing::error!("Handshake already completed");
                    Err(LinkError::new("Handshake already completed"))
                } else {
                    match Self::handle_handshake(&mut stream).await {
                        Ok(()) => {
                            tracing::info!("Handshake completed");
                            handshake_completed = true;
                            Ok(())
                        }
                        Err(e) => {
                            tracing::error!("Handshake failed: {}", e);
                            Err(e)
                        }
                    }
                }
            } else if !handshake_completed {
                Err(LinkError::new(
                    "Handshake is required before sending commands",
                ))
            } else {
                match msg {
                    MSG_CONFIG_GEOMETRY => self.handle_config_geometry(&mut stream).await,
                    MSG_UPDATE_GEOMETRY => self.handle_update_geometry(&mut stream).await,
                    MSG_SEND_DATA => self.handle_send_data(&mut stream).await,
                    MSG_READ_DATA => self.handle_read_data(&mut stream).await,
                    MSG_CLOSE => self.handle_close(&mut stream).await,
                    other => Err(LinkError::new(format!("Unknown message type: {}", other))),
                }
            };

            match result {
                Ok(()) => {
                    if msg == MSG_CLOSE {
                        break;
                    }
                }
                Err(e) => {
                    tracing::error!("Error handling client request: {}", e);
                    let _ = self.send_error(&mut stream, &e).await;
                    if !handshake_completed || msg == MSG_CLOSE {
                        break;
                    }
                }
            }
        }
    }

    async fn handle_handshake(stream: &mut TcpStream) -> Result<(), LinkError> {
        let mut version_buf = [0u8; size_of::<u16>()];
        stream.read_exact(&mut version_buf).await?;
        let version = u16::from_le_bytes(version_buf);
        if version != REMOTE_PROTOCOL_VERSION {
            return Err(LinkError::new(format!(
                "Unsupported protocol version: {}",
                version
            )));
        }
        tracing::info!("Client protocol version: {}", version);

        let mut magic_buf = [0u8; REMOTE_PROTOCOL_MAGIC.len()];
        stream.read_exact(&mut magic_buf).await?;
        if &magic_buf != REMOTE_PROTOCOL_MAGIC {
            tracing::error!("Invalid client magic: {:?}", magic_buf);
            return Err(LinkError::new("Invalid client magic"));
        }

        stream.write_all(&[MSG_OK]).await?;
        Ok(())
    }

    async fn handle_config_geometry(&mut self, stream: &mut TcpStream) -> Result<(), LinkError> {
        if self.link.is_some() {
            tracing::error!("Link is already open");
            Err(LinkError::new("Link is already opened"))
        } else {
            let geometry = Self::read_geometry(stream).await?;
            tracing::info!("Opening link...");

            let mut link = (self.link_factory)();
            AsyncLink::open(&mut link, &geometry).await?;
            self.num_devices = geometry.num_devices();
            tracing::info!(
                "Link opened with {} device{}",
                self.num_devices,
                if self.num_devices == 1 { "" } else { "s" }
            );

            stream.write_all(&[MSG_OK]).await?;

            self.link = Some(link);

            Ok(())
        }
    }

    async fn handle_update_geometry(&mut self, stream: &mut TcpStream) -> Result<(), LinkError> {
        if let Some(link) = self.link.as_mut() {
            let geometry = Self::read_geometry(stream).await?;
            AsyncLink::update(link, &geometry).await?;
            stream.write_all(&[MSG_OK]).await?;
            Ok(())
        } else {
            Err(LinkError::closed())
        }
    }

    async fn read_geometry(stream: &mut TcpStream) -> std::io::Result<Geometry> {
        let mut num_devices_buf = [0u8; size_of::<u32>()];
        stream.read_exact(&mut num_devices_buf).await?;
        let num_devices = u32::from_le_bytes(num_devices_buf);

        let mut devices = Vec::new();
        for _ in 0..num_devices {
            let mut pos_buf = [0u8; size_of::<f32>() * 3];
            stream.read_exact(&mut pos_buf).await?;
            let x = f32::from_le_bytes([pos_buf[0], pos_buf[1], pos_buf[2], pos_buf[3]]);
            let y = f32::from_le_bytes([pos_buf[4], pos_buf[5], pos_buf[6], pos_buf[7]]);
            let z = f32::from_le_bytes([pos_buf[8], pos_buf[9], pos_buf[10], pos_buf[11]]);

            let mut rot_buf = [0u8; size_of::<f32>() * 4];
            stream.read_exact(&mut rot_buf).await?;
            let w = f32::from_le_bytes([rot_buf[0], rot_buf[1], rot_buf[2], rot_buf[3]]);
            let i = f32::from_le_bytes([rot_buf[4], rot_buf[5], rot_buf[6], rot_buf[7]]);
            let j = f32::from_le_bytes([rot_buf[8], rot_buf[9], rot_buf[10], rot_buf[11]]);
            let k = f32::from_le_bytes([rot_buf[12], rot_buf[13], rot_buf[14], rot_buf[15]]);

            devices.push(
                autd3_core::devices::AUTD3 {
                    pos: Point3::new(x, y, z),
                    rot: UnitQuaternion::new_unchecked(Quaternion::new(w, i, j, k)),
                }
                .into(),
            );
        }

        Ok(Geometry::new(devices))
    }

    async fn handle_send_data(&mut self, stream: &mut TcpStream) -> Result<(), LinkError> {
        if let Some(link) = self.link.as_mut() {
            let mut tx = AsyncLink::alloc_tx_buffer(link).await?;

            for tx_msg in tx.iter_mut() {
                let bytes = unsafe {
                    std::slice::from_raw_parts_mut(
                        tx_msg as *mut TxMessage as *mut u8,
                        size_of::<TxMessage>(),
                    )
                };
                stream.read_exact(bytes).await?;
            }

            AsyncLink::send(link, tx).await?;

            stream.write_all(&[MSG_OK]).await?;

            Ok(())
        } else {
            Err(LinkError::closed())
        }
    }

    async fn handle_read_data(&mut self, stream: &mut TcpStream) -> Result<(), LinkError> {
        let num_devices = self.num_devices;
        let mut rx = match self.rx_buf.take() {
            Some(buf) if buf.len() == num_devices => buf,
            _ => vec![RxMessage::new(0, Ack::new(0, 0)); num_devices],
        };
        if let Some(link) = self.link.as_mut() {
            AsyncLink::receive(link, &mut rx).await?;

            let buffer_size = size_of::<u8>() + size_of::<RxMessage>() * rx.len();
            if self.read_buffer.len() < buffer_size {
                self.read_buffer.resize(buffer_size, 0);
            }

            self.read_buffer[0] = MSG_OK;
            unsafe {
                std::ptr::copy_nonoverlapping(
                    rx.as_ptr() as *const u8,
                    self.read_buffer.as_mut_ptr().add(1),
                    size_of::<RxMessage>() * rx.len(),
                );
            }
            self.rx_buf = Some(rx);
            stream.write_all(&self.read_buffer).await?;

            Ok(())
        } else {
            Err(LinkError::closed())
        }
    }

    async fn handle_close(&mut self, stream: &mut TcpStream) -> Result<(), LinkError> {
        if let Some(link) = self.link.as_mut() {
            AsyncLink::close(link).await?;
            stream.write_all(&[MSG_OK]).await?;
            Ok(())
        } else {
            Err(LinkError::closed())
        }
    }

    async fn send_error(&self, stream: &mut TcpStream, error: &LinkError) -> std::io::Result<()> {
        let error_msg = error.to_string();
        let error_bytes = error_msg.as_bytes();
        let error_len = error_bytes.len() as u32;

        let mut buffer = Vec::with_capacity(size_of::<u8>() + size_of::<u32>() + error_bytes.len());
        buffer.push(MSG_ERROR);
        buffer.extend_from_slice(&error_len.to_le_bytes());
        buffer.extend_from_slice(error_bytes);

        stream.write_all(&buffer).await
    }
}
