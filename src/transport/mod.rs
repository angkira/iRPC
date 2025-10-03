//! Concrete transport implementations for embedded systems
//!
//! This module provides ready-to-use transport implementations for common
//! embedded communication buses. Firmware only needs to provide configuration,
//! and iRPC handles all low-level hardware interaction.
//!
//! # Available Transports
//!
//! - **CAN-FD** - `CanFdTransport` (requires `stm32g4` or `stm32f4` feature)
//! - **SPI** - Coming soon
//! - **UART** - Coming soon
//!
//! # Example
//!
//! ```no_run
//! use irpc::transport::{CanFdTransport, CanFdConfig};
//! use irpc::Joint;
//!
//! // iRPC handles all hardware configuration
//! let config = CanFdConfig {
//!     node_id: 0x0010,
//!     nominal_bitrate: 1_000_000,
//!     data_bitrate: 5_000_000,
//! };
//!
//! let transport = CanFdTransport::new(peripherals.FDCAN1, pins, config)?;
//! let mut joint = Joint::new(0x0010);
//!
//! loop {
//!     if let Some(msg) = transport.receive_message()? {
//!         if let Some(resp) = joint.handle_message(&msg) {
//!             transport.send_message(&resp)?;
//!         }
//!     }
//! }
//! ```

// CAN-FD transport for STM32 microcontrollers
#[cfg(any(feature = "stm32g4", feature = "stm32f4"))]
pub mod canfd;

#[cfg(any(feature = "stm32g4", feature = "stm32f4"))]
pub use canfd::{CanFdTransport, CanFdConfig, CanFdPins, CanError};

// Future transports
// #[cfg(feature = "spi")]
// pub mod spi;
//
// #[cfg(feature = "uart")]
// pub mod uart;
