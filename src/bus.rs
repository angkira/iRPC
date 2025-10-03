use crate::protocol::{Message, DeviceId};

#[cfg(feature = "joint_api")]
use crate::protocol::ProtocolError;

#[cfg(not(feature = "arm_api"))]
extern crate alloc;

#[cfg(all(feature = "arm_api", not(feature = "joint_api")))]
use std::vec::Vec;

/// Device information for discovery
#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: DeviceId,
    pub entity_type: u16,
}

// ============================================================================
// ARM API: Async communication adapter (for host std environment)
// ============================================================================

#[cfg(feature = "arm_api")]
use async_trait::async_trait;

#[cfg(feature = "arm_api")]
#[async_trait]
pub trait CommunicationAdapter: Send + Sync {
    type Error: core::fmt::Debug;

    async fn transmit(&self, message: &Message) -> Result<(), Self::Error>;
    async fn receive(&self) -> Result<Option<Message>, Self::Error>;
    async fn discover_devices(&self) -> Result<Vec<DeviceInfo>, Self::Error>;
    fn is_connected(&self) -> bool;
}

// ============================================================================
// JOINT API: Embedded transport trait (for no_std firmware)
// ============================================================================

/// Low-level embedded transport for no_std environments
///
/// This trait provides blocking send/receive operations for raw byte buffers.
/// It's designed for use with CAN, SPI, UART, or other embedded communication buses.
#[cfg(feature = "joint_api")]
pub trait EmbeddedTransport {
    /// Transport-specific error type
    type Error: core::fmt::Debug;

    /// Send raw bytes over the transport (blocking)
    fn send_blocking(&mut self, data: &[u8]) -> Result<(), Self::Error>;

    /// Receive raw bytes from the transport (blocking, non-blocking returns None)
    ///
    /// Returns Ok(Some(data)) if data is available, Ok(None) if no data ready.
    fn receive_blocking(&mut self) -> Result<Option<&[u8]>, Self::Error>;

    /// Check if transport is ready for communication
    fn is_ready(&self) -> bool {
        true
    }
}

// ============================================================================
// Transport Layer: High-level wrapper with automatic serialization
// ============================================================================

/// High-level transport layer that handles message serialization/deserialization
///
/// This wrapper provides a simple API for sending and receiving Messages,
/// automatically handling the encoding/decoding internally.
///
/// # Example
/// ```no_run
/// use irpc::{TransportLayer, Message};
///
/// // Assuming you have a CAN or UART transport implementing EmbeddedTransport
/// let mut transport = TransportLayer::new(my_can_bus);
///
/// // Send a message - serialization is automatic
/// transport.send_message(&message)?;
///
/// // Receive a message - deserialization is automatic
/// if let Some(msg) = transport.receive_message()? {
///     // Process message
/// }
/// ```
#[cfg(feature = "joint_api")]
pub struct TransportLayer<T: EmbeddedTransport> {
    transport: T,
    rx_buffer: [u8; Message::max_size()],
}

#[cfg(feature = "joint_api")]
impl<T: EmbeddedTransport> TransportLayer<T> {
    /// Create a new transport layer wrapping an embedded transport
    pub fn new(transport: T) -> Self {
        Self {
            transport,
            rx_buffer: [0u8; Message::max_size()],
        }
    }

    /// Send a message (automatically serializes)
    ///
    /// This method handles serialization internally and sends the encoded bytes
    /// over the underlying transport.
    pub fn send_message(&mut self, message: &Message) -> Result<(), TransportError<T::Error>> {
        let data = message.serialize()
            .map_err(|_| TransportError::SerializationFailed)?;

        self.transport.send_blocking(&data)
            .map_err(TransportError::TransportError)
    }

    /// Receive a message (automatically deserializes)
    ///
    /// Returns Ok(Some(message)) if a message was received and successfully decoded,
    /// Ok(None) if no data is available, or Err if there was a transport or deserialization error.
    pub fn receive_message(&mut self) -> Result<Option<Message>, TransportError<T::Error>> {
        match self.transport.receive_blocking() {
            Ok(Some(data)) => {
                // Copy data to our buffer (needed because transport may reuse its buffer)
                let len = data.len().min(self.rx_buffer.len());
                self.rx_buffer[..len].copy_from_slice(&data[..len]);

                // Deserialize
                Message::deserialize(&self.rx_buffer[..len])
                    .map(Some)
                    .map_err(|_| TransportError::DeserializationFailed)
            }
            Ok(None) => Ok(None),
            Err(e) => Err(TransportError::TransportError(e)),
        }
    }

    /// Check if the transport is ready
    pub fn is_ready(&self) -> bool {
        self.transport.is_ready()
    }

    /// Get a mutable reference to the underlying transport
    pub fn transport_mut(&mut self) -> &mut T {
        &mut self.transport
    }

    /// Get a reference to the underlying transport
    pub fn transport(&self) -> &T {
        &self.transport
    }
}

/// Transport layer errors
#[cfg(feature = "joint_api")]
#[derive(Debug)]
pub enum TransportError<E: core::fmt::Debug> {
    /// Failed to serialize message
    SerializationFailed,
    /// Failed to deserialize message
    DeserializationFailed,
    /// Underlying transport error
    TransportError(E),
}

#[cfg(feature = "joint_api")]
impl<E: core::fmt::Debug> From<TransportError<E>> for ProtocolError {
    fn from(e: TransportError<E>) -> Self {
        match e {
            TransportError::SerializationFailed => ProtocolError::SerializationError(
                #[cfg(feature = "arm_api")]
                "Transport serialization failed".to_string(),
                #[cfg(not(feature = "arm_api"))]
                alloc::string::String::new()
            ),
            TransportError::DeserializationFailed => ProtocolError::DeserializationError(
                #[cfg(feature = "arm_api")]
                "Transport deserialization failed".to_string(),
                #[cfg(not(feature = "arm_api"))]
                alloc::string::String::new()
            ),
            TransportError::TransportError(_) => ProtocolError::IoError(0),
        }
    }
}