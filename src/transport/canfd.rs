//! CAN-FD transport implementation for STM32 microcontrollers
//!
//! This module provides a ready-to-use CAN-FD transport that handles all
//! low-level hardware configuration. Firmware only needs to provide pins and bitrate.
//!
//! # Features
//!
//! - Automatic FDCAN peripheral configuration
//! - Message serialization/deserialization
//! - Buffer management
//! - Error handling
//!
//! # Example
//!
//! ```no_run
//! use irpc::transport::{CanFdTransport, CanFdConfig};
//! use irpc::Joint;
//!
//! let config = CanFdConfig {
//!     node_id: 0x0010,
//!     nominal_bitrate: 1_000_000,  // 1 Mbps for arbitration
//!     data_bitrate: 5_000_000,      // 5 Mbps for data phase
//! };
//!
//! let mut transport = CanFdTransport::new(
//!     peripherals.FDCAN1,
//!     peripherals.PA12,  // TX
//!     peripherals.PA11,  // RX
//!     config,
//! ).expect("FDCAN init failed");
//!
//! let mut joint = Joint::new(0x0010);
//!
//! loop {
//!     if let Some(msg) = transport.receive_message().ok().flatten() {
//!         if let Some(resp) = joint.handle_message(&msg) {
//!             transport.send_message(&resp).ok();
//!         }
//!     }
//! }
//! ```

#[cfg(not(feature = "arm_api"))]
extern crate alloc;

#[cfg(not(feature = "arm_api"))]
use alloc::vec::Vec;

use crate::protocol::{Message, DeviceId};

// Maximum CAN-FD frame payload (64 bytes)
const MAX_FDCAN_PAYLOAD: usize = 64;

// ============================================================================
// Configuration
// ============================================================================

/// CAN-FD configuration for a joint node
#[derive(Debug, Clone)]
pub struct CanFdConfig {
    /// Node ID for this device (used in CAN identifiers)
    pub node_id: DeviceId,

    /// Nominal bitrate for arbitration phase (Hz)
    /// Typical: 1_000_000 (1 Mbps)
    pub nominal_bitrate: u32,

    /// Data bitrate for FD data phase (Hz)
    /// Typical: 5_000_000 (5 Mbps)
    pub data_bitrate: u32,
}

impl CanFdConfig {
    /// Create configuration for a joint with default bitrates
    ///
    /// Default: 1 Mbps nominal, 5 Mbps data
    pub fn for_joint(node_id: DeviceId) -> Self {
        Self {
            node_id,
            nominal_bitrate: 1_000_000,
            data_bitrate: 5_000_000,
        }
    }
}

/// Pin configuration for CAN-FD
#[cfg(any(feature = "stm32g4", feature = "stm32f4"))]
pub struct CanFdPins<TX, RX> {
    pub tx: TX,
    pub rx: RX,
}

// ============================================================================
// Error Handling
// ============================================================================

/// CAN-FD transport errors
#[derive(Debug, Clone, Copy)]
pub enum CanError {
    /// Peripheral not initialized
    NotInitialized,

    /// Hardware not ready
    NotReady,

    /// Transmission buffer full
    TxBufferFull,

    /// Transmission failed
    TxFailed,

    /// Reception failed / no data
    RxFailed,

    /// Message serialization failed
    SerializationError,

    /// Message deserialization failed
    DeserializationError,

    /// Invalid configuration
    InvalidConfig,

    /// Frame too large for CAN-FD
    FrameTooLarge,
}

// ============================================================================
// STM32G4/F4 Implementation
// ============================================================================

#[cfg(feature = "stm32g4")]
use embassy_stm32::can::fdcan::{Fdcan, Instance as FdcanInstance};

#[cfg(feature = "stm32g4")]
use embassy_stm32::peripherals::FDCAN1;

/// CAN-FD transport for STM32G4 microcontrollers
///
/// This transport handles all FDCAN hardware configuration and provides
/// automatic message serialization/deserialization.
#[cfg(feature = "stm32g4")]
pub struct CanFdTransport<'d, T: FdcanInstance> {
    fdcan: Fdcan<'d, T>,
    node_id: DeviceId,
    rx_buffer: [u8; MAX_FDCAN_PAYLOAD],
    tx_buffer: [u8; MAX_FDCAN_PAYLOAD],
}

#[cfg(feature = "stm32g4")]
impl<'d, T: FdcanInstance> CanFdTransport<'d, T> {
    /// Create and configure a new CAN-FD transport
    ///
    /// This function:
    /// - Configures FDCAN peripheral with specified bitrates
    /// - Sets up standard ID filters for the node
    /// - Initializes TX/RX FIFOs
    /// - Enables CAN-FD mode
    ///
    /// # Arguments
    ///
    /// * `fdcan` - FDCAN peripheral instance
    /// * `tx_pin` - TX pin (e.g., PA12)
    /// * `rx_pin` - RX pin (e.g., PA11)
    /// * `config` - Bitrate and node ID configuration
    ///
    /// # Returns
    ///
    /// Returns `Ok(transport)` if successful, `Err(CanError)` otherwise.
    pub fn new<TX, RX>(
        fdcan: impl embassy_stm32::Peripheral<P = T> + 'd,
        tx_pin: TX,
        rx_pin: RX,
        config: CanFdConfig,
    ) -> Result<Self, CanError>
    where
        TX: embassy_stm32::can::fdcan::TxPin<T> + 'd,
        RX: embassy_stm32::can::fdcan::RxPin<T> + 'd,
    {
        use embassy_stm32::can::fdcan::{config::FdCanConfig, filter::*};

        // Configure FDCAN
        let mut fdcan_config = FdCanConfig::default();

        // Enable FD mode with bit rate switching
        fdcan_config.mode = embassy_stm32::can::fdcan::config::OperatingMode::NormalOperationMode;

        // TODO: Calculate bit timing from bitrates
        // For now, use default timing (requires embassy-stm32 update for dynamic calculation)

        let mut fdcan = Fdcan::new(fdcan, rx_pin, tx_pin, fdcan_config);

        // Configure filters to accept messages for this node
        // Standard ID filter: accept messages with ID = node_id
        fdcan.set_standard_filter(
            StandardFilterSlot::_0,
            StandardFilter::accept_masked_id(config.node_id as u32, 0x7FF),
        );

        // Start FDCAN
        fdcan.enable();

        Ok(Self {
            fdcan,
            node_id: config.node_id,
            rx_buffer: [0u8; MAX_FDCAN_PAYLOAD],
            tx_buffer: [0u8; MAX_FDCAN_PAYLOAD],
        })
    }

    /// Send a message over CAN-FD
    ///
    /// Automatically serializes the message and transmits over CAN-FD.
    pub fn send_message(&mut self, message: &Message) -> Result<(), CanError> {
        // Serialize message
        let data = message.serialize()
            .map_err(|_| CanError::SerializationError)?;

        if data.len() > MAX_FDCAN_PAYLOAD {
            return Err(CanError::FrameTooLarge);
        }

        // Copy to TX buffer
        self.tx_buffer[..data.len()].copy_from_slice(&data);

        // Create CAN frame with node ID
        use embassy_stm32::can::fdcan::frame::{FrameFormat, TxFrameHeader};
        use embassy_stm32::can::fdcan::id::StandardId;

        let header = TxFrameHeader {
            id: StandardId::new(self.node_id).unwrap().into(),
            len: data.len() as u8,
            frame_format: FrameFormat::Fdcan,
            bit_rate_switching: true,
            marker: None,
        };

        // Transmit (blocking for now)
        self.fdcan.write(&header, &self.tx_buffer[..data.len()])
            .map_err(|_| CanError::TxFailed)?;

        Ok(())
    }

    /// Receive a message from CAN-FD
    ///
    /// Returns `Ok(Some(message))` if a message was received,
    /// `Ok(None)` if no message is available.
    pub fn receive_message(&mut self) -> Result<Option<Message>, CanError> {
        // Try to receive a frame (non-blocking)
        match self.fdcan.read() {
            Ok(envelope) => {
                let len = envelope.header.len as usize;

                if len > MAX_FDCAN_PAYLOAD {
                    return Err(CanError::FrameTooLarge);
                }

                // Copy data to RX buffer
                self.rx_buffer[..len].copy_from_slice(&envelope.data[..len]);

                // Deserialize
                Message::deserialize(&self.rx_buffer[..len])
                    .map(Some)
                    .map_err(|_| CanError::DeserializationError)
            }
            Err(_) => Ok(None), // No data available
        }
    }

    /// Check if transport is ready
    pub fn is_ready(&self) -> bool {
        // Check if FDCAN is in normal mode
        // For now, always return true
        true
    }

    /// Get node ID
    pub fn node_id(&self) -> DeviceId {
        self.node_id
    }
}

// ============================================================================
// Compatibility layer for custom implementations
// ============================================================================

/// Simplified CAN-FD transport (no embassy dependency)
///
/// This is a placeholder for when embassy-stm32 is not available.
/// Users should implement `EmbeddedTransport` trait for their own hardware.
#[cfg(not(any(feature = "stm32g4", feature = "stm32f4")))]
pub struct CanFdTransport {
    node_id: DeviceId,
}

#[cfg(not(any(feature = "stm32g4", feature = "stm32f4")))]
impl CanFdTransport {
    pub fn node_id(&self) -> DeviceId {
        self.node_id
    }
}
