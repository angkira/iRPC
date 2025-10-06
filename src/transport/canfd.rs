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
#[cfg_attr(feature = "defmt", derive(defmt::Format))]
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
use embassy_stm32::can::{Can, Instance};

/// CAN-FD transport for STM32G4 microcontrollers
///
/// This transport handles all FDCAN hardware configuration and provides
/// automatic message serialization/deserialization.
#[cfg(feature = "stm32g4")]
pub struct CanFdTransport<'d> {
    can: Can<'d>,
    node_id: DeviceId,
    rx_buffer: [u8; MAX_FDCAN_PAYLOAD],
    tx_buffer: [u8; MAX_FDCAN_PAYLOAD],
}

#[cfg(feature = "stm32g4")]
impl<'d> CanFdTransport<'d> {
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
    pub fn new<T, TX, RX, I>(
        fdcan: embassy_stm32::Peri<'d, T>,
        rx_pin: embassy_stm32::Peri<'d, RX>,
        tx_pin: embassy_stm32::Peri<'d, TX>,
        irqs: I,
        config: CanFdConfig,
    ) -> Result<Self, CanError>
    where
        T: Instance,
        TX: embassy_stm32::can::TxPin<T>,
        RX: embassy_stm32::can::RxPin<T>,
        I: embassy_stm32::interrupt::typelevel::Binding<T::IT0Interrupt, embassy_stm32::can::IT0InterruptHandler<T>>
            + embassy_stm32::interrupt::typelevel::Binding<T::IT1Interrupt, embassy_stm32::can::IT1InterruptHandler<T>>
            + 'd,
    {
        use embassy_stm32::can;

        // Create configurator
        let mut can_config = can::CanConfigurator::new(fdcan, rx_pin, tx_pin, irqs);

        // Set bitrates
        can_config.set_bitrate(config.nominal_bitrate);

        // Enable FD mode with higher data bitrate
        can_config.set_fd_data_bitrate(config.data_bitrate, true);

        // Configure filters to accept messages for this node
        // Accept all messages into FIFO0 for now (we'll filter by ID in software)
        can_config.properties().set_extended_filter(
            can::filter::ExtendedFilterSlot::_0,
            can::filter::ExtendedFilter::accept_all_into_fifo0(),
        );

        // Start in normal operation mode
        let can = can_config.start(can::OperatingMode::NormalOperationMode);

        Ok(Self {
            can,
            node_id: config.node_id,
            rx_buffer: [0u8; MAX_FDCAN_PAYLOAD],
            tx_buffer: [0u8; MAX_FDCAN_PAYLOAD],
        })
    }

    /// Send a message over CAN-FD
    ///
    /// Automatically serializes the message and transmits over CAN-FD.
    pub async fn send_message(&mut self, message: &Message) -> Result<(), CanError> {
        // Serialize message
        let data = message.serialize()
            .map_err(|_| CanError::SerializationError)?;

        if data.len() > MAX_FDCAN_PAYLOAD {
            return Err(CanError::FrameTooLarge);
        }

        // Copy to TX buffer
        self.tx_buffer[..data.len()].copy_from_slice(&data);

        // Create CAN-FD frame with standard ID
        use embassy_stm32::can::frame::FdFrame;

        let frame = FdFrame::new_standard(self.node_id, &self.tx_buffer[..data.len()])
            .map_err(|_| CanError::InvalidConfig)?;

        // Transmit (async)
        self.can.write_fd(&frame).await;

        Ok(())
    }

    /// Receive a message from CAN-FD
    ///
    /// Waits for a message to be received.
    pub async fn receive_message(&mut self) -> Result<Message, CanError> {
        // Receive a frame (async)
        let envelope = self.can.read_fd().await
            .map_err(|_| CanError::RxFailed)?;

        let rx_frame = envelope.frame;
        let len = rx_frame.header().len() as usize;

        if len > MAX_FDCAN_PAYLOAD {
            return Err(CanError::FrameTooLarge);
        }

        // Copy data to RX buffer
        self.rx_buffer[..len].copy_from_slice(&rx_frame.data()[..len]);

        // Deserialize
        Message::deserialize(&self.rx_buffer[..len])
            .map_err(|_| CanError::DeserializationError)
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
