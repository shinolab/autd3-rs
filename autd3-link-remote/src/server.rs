use std::future::Future;

use autd3_core::{
    geometry::{Geometry, Quaternion},
    link::{Link, LinkError, RxMessage, TxMessage},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    select,
};

use crate::{
    MSG_CLOSE, MSG_CONFIG_GEOMETRY, MSG_ERROR, MSG_OK, MSG_READ_DATA, MSG_SEND_DATA,
    MSG_UPDATE_GEOMETRY,
};

/// A server that accepts connections from [`Remote`](crate::Remote) clients and forwards requests to a link.
pub struct RemoteServer<L: Link> {
    link: L,
    port: u16,
    shutdown: Option<Box<dyn Future<Output = ()> + Send + Unpin>>,
}

impl<L: Link> RemoteServer<L> {
    /// Create a new [`RemoteServer`].
    ///
    /// # Arguments
    ///
    /// * `port` - The port to listen on
    /// * `link` - The link to forward requests to
    pub const fn new(port: u16, link: L) -> Self {
        Self {
            link,
            port,
            shutdown: None,
        }
    }

    /// Configure graceful shutdown with a custom shutdown signal.
    ///
    /// # Arguments
    ///
    /// * `signal` - A future that completes when the server should shut down
    pub fn with_graceful_shutdown<F>(mut self, signal: F) -> Self
    where
        F: Future<Output = ()> + Send + 'static,
    {
        self.shutdown = Some(Box::new(Box::pin(signal)));
        self
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

        let res = if let Some(shutdown) = self.shutdown.take() {
            select! {
                result = self.accept_loop(&listener) => result,
                _ = shutdown => Ok(()),
            }
        } else {
            self.accept_loop(&listener).await
        };

        self.link.close()?;

        res
    }

    async fn accept_loop(&mut self, listener: &TcpListener) -> Result<(), LinkError> {
        loop {
            let (stream, _) = listener.accept().await?;
            self.handle_client(stream).await?;
        }
    }

    async fn handle_client(&mut self, mut stream: TcpStream) -> Result<(), LinkError> {
        loop {
            let mut msg_type = [0u8; 1];
            if stream.read_exact(&mut msg_type).await.is_err() {
                break;
            }

            let result = match msg_type[0] {
                MSG_CONFIG_GEOMETRY => self.handle_config_geometry(&mut stream).await,
                MSG_UPDATE_GEOMETRY => self.handle_update_geometry(&mut stream).await,
                MSG_SEND_DATA => self.handle_send_data(&mut stream).await,
                MSG_READ_DATA => self.handle_read_data(&mut stream).await,
                MSG_CLOSE => {
                    let result = self.handle_close(&mut stream).await;
                    if result.is_ok() {
                        break;
                    }
                    result
                }
                _ => Err(LinkError::new(format!(
                    "Unknown message type: {}",
                    msg_type[0]
                ))),
            };

            if let Err(e) = result {
                let _ = self.send_error(&mut stream, &e).await;
                return Err(e);
            }
        }

        Ok(())
    }

    async fn handle_config_geometry(&mut self, stream: &mut TcpStream) -> Result<(), LinkError> {
        let geometry = self.read_geometry(stream).await?;

        self.link.open(&geometry)?;

        stream.write_all(&[MSG_OK]).await?;
        Ok(())
    }

    async fn handle_update_geometry(&mut self, stream: &mut TcpStream) -> Result<(), LinkError> {
        let geometry = self.read_geometry(stream).await?;

        self.link.update(&geometry)?;

        stream.write_all(&[MSG_OK]).await?;
        Ok(())
    }

    async fn read_geometry(&self, stream: &mut TcpStream) -> std::io::Result<Geometry> {
        let mut num_devices_buf = [0u8; 4];
        stream.read_exact(&mut num_devices_buf).await?;
        let num_devices = u32::from_le_bytes(num_devices_buf);

        let mut devices = Vec::new();

        for _ in 0..num_devices {
            let mut pos_buf = [0u8; 12];
            stream.read_exact(&mut pos_buf).await?;
            let x = f32::from_le_bytes([pos_buf[0], pos_buf[1], pos_buf[2], pos_buf[3]]);
            let y = f32::from_le_bytes([pos_buf[4], pos_buf[5], pos_buf[6], pos_buf[7]]);
            let z = f32::from_le_bytes([pos_buf[8], pos_buf[9], pos_buf[10], pos_buf[11]]);

            let mut rot_buf = [0u8; 16];
            stream.read_exact(&mut rot_buf).await?;
            let w = f32::from_le_bytes([rot_buf[0], rot_buf[1], rot_buf[2], rot_buf[3]]);
            let i = f32::from_le_bytes([rot_buf[4], rot_buf[5], rot_buf[6], rot_buf[7]]);
            let j = f32::from_le_bytes([rot_buf[8], rot_buf[9], rot_buf[10], rot_buf[11]]);
            let k = f32::from_le_bytes([rot_buf[12], rot_buf[13], rot_buf[14], rot_buf[15]]);

            devices.push(
                autd3_core::devices::AUTD3 {
                    pos: autd3_core::geometry::Point3::new(x, y, z),
                    rot: autd3_core::geometry::UnitQuaternion::new_unchecked(Quaternion::new(
                        w, i, j, k,
                    )),
                }
                .into(),
            );
        }

        Ok(autd3_core::geometry::Geometry::new(devices))
    }

    async fn handle_send_data(&mut self, stream: &mut TcpStream) -> Result<(), LinkError> {
        let mut _num_devices = [0u8; 4];
        stream.read_exact(&mut _num_devices).await?;

        let mut tx = self.link.alloc_tx_buffer()?;

        let data_size = std::mem::size_of::<TxMessage>();
        for tx_msg in tx.iter_mut() {
            let bytes = unsafe {
                std::slice::from_raw_parts_mut(tx_msg as *mut TxMessage as *mut u8, data_size)
            };
            stream.read_exact(bytes).await?;
        }

        self.link.send(tx)?;

        stream.write_all(&[MSG_OK]).await?;

        Ok(())
    }

    async fn handle_read_data(&mut self, stream: &mut TcpStream) -> Result<(), LinkError> {
        let num_devices = self.link.alloc_tx_buffer()?.len();
        let mut rx = Vec::with_capacity(num_devices);
        #[allow(clippy::uninit_vec)]
        unsafe {
            rx.set_len(num_devices);
        }

        self.link.receive(&mut rx)?;

        let num_devices = rx.len() as u32;
        let data_size = std::mem::size_of::<RxMessage>();

        let mut buffer = Vec::with_capacity(1 + 4 + data_size * rx.len());
        buffer.push(MSG_OK);
        buffer.extend_from_slice(&num_devices.to_le_bytes());

        for msg in &rx {
            let bytes = unsafe {
                std::slice::from_raw_parts(msg as *const RxMessage as *const u8, data_size)
            };
            buffer.extend_from_slice(bytes);
        }

        stream.write_all(&buffer).await?;

        Ok(())
    }

    async fn handle_close(&mut self, stream: &mut TcpStream) -> Result<(), LinkError> {
        self.link.close()?;

        stream.write_all(&[MSG_OK]).await?;

        Ok(())
    }

    async fn send_error(&self, stream: &mut TcpStream, error: &LinkError) -> std::io::Result<()> {
        let error_msg = error.to_string();
        let error_bytes = error_msg.as_bytes();
        let error_len = error_bytes.len() as u32;

        let mut buffer = Vec::with_capacity(1 + 4 + error_bytes.len());
        buffer.push(MSG_ERROR);
        buffer.extend_from_slice(&error_len.to_le_bytes());
        buffer.extend_from_slice(error_bytes);

        stream.write_all(&buffer).await
    }
}
