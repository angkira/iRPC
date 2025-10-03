//! iRPC - Robotic node interaction protocol
//! 
//! This library provides a feature-gated implementation of the iRPC protocol
//! for robotic systems, supporting both std host environments and no_std
//! embedded environments.

#![cfg_attr(not(feature = "arm_api"), no_std)]

// When using no_std, we need alloc for Vec and String
#[cfg(not(feature = "arm_api"))]
extern crate alloc;

// Core modules available in all configurations
pub mod config;
pub mod protocol;
pub mod bus;

// Feature-gated modules
#[cfg(feature = "arm_api")]
pub mod arm;

#[cfg(feature = "joint_api")]
pub mod joint;

// Concrete transport implementations (joint_api only)
#[cfg(feature = "joint_api")]
pub mod transport;

// Re-export commonly used types
pub use config::*;
pub use protocol::*;

// Re-export bus types based on features
#[cfg(feature = "arm_api")]
pub use bus::{CommunicationAdapter, DeviceInfo};

#[cfg(feature = "joint_api")]
pub use bus::{EmbeddedTransport, TransportLayer, TransportError, DeviceInfo};

#[cfg(feature = "arm_api")]
pub use arm::*;

#[cfg(feature = "joint_api")]
pub use joint::*;