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
pub mod protocol;
pub mod bus;

// Feature-gated modules
#[cfg(feature = "arm_api")]
pub mod arm;

#[cfg(feature = "joint_api")]
pub mod joint;

// Re-export commonly used types
pub use protocol::*;
pub use bus::*;

#[cfg(feature = "arm_api")]
pub use arm::*;

#[cfg(feature = "joint_api")]
pub use joint::*;