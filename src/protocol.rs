//! Core protocol definitions and message types for iRPC

// Import std types when arm_api feature is enabled, otherwise use no_std alternatives
#[cfg(feature = "arm_api")]
use std::vec::Vec;
#[cfg(feature = "arm_api")]
use std::string::String;

#[cfg(not(feature = "arm_api"))]
extern crate alloc;
#[cfg(not(feature = "arm_api"))]
use alloc::vec::Vec;
#[cfg(not(feature = "arm_api"))]
use alloc::string::String;

/// Protocol version information
pub const PROTOCOL_VERSION: &str = "1.0.0";

/// Core message types for the iRPC protocol
#[derive(Debug, Clone)]
pub enum Message {
    /// Request message with ID and payload
    Request { id: u32, payload: Vec<u8> },
    /// Response message with ID and result
    Response { id: u32, result: Result<Vec<u8>, String> },
    /// Notification message (fire-and-forget)
    Notification { payload: Vec<u8> },
}

/// Error types for protocol operations
#[derive(Debug, Clone)]
pub enum ProtocolError {
    /// Invalid message format
    InvalidMessage,
    /// Unsupported protocol version
    UnsupportedVersion,
    /// Communication timeout
    Timeout,
    /// General IO error
    IoError(String),
}