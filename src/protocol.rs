use serde::{Serialize, Deserialize};

#[cfg(not(feature = "arm_api"))]
extern crate alloc;

#[cfg(not(feature = "arm_api"))]
use alloc::{vec::Vec, string::String};

#[cfg(feature = "arm_api")]
use std::{vec::Vec, string::String};

/// Device identifier type
pub type DeviceId = u16;

/// Message identifier type for request/response correlation
pub type MessageId = u32;

/// Lifecycle state of a joint in the robotic system
///
/// State transitions follow a strict lifecycle:
/// - Unconfigured → Inactive (via Configure)
/// - Inactive → Active (via Activate)
/// - Active → Inactive (via Deactivate)
/// - Any → Unconfigured (via Reset)
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LifecycleState {
    /// Joint is not configured and cannot accept commands
    Unconfigured,
    /// Joint is configured but not ready for motion
    Inactive,
    /// Joint is active and can execute motion commands
    Active,
    /// Joint is in error state
    Error,
}

/// Target position and velocity for joint motion
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct SetTargetPayload {
    /// Target angle in degrees
    pub target_angle: f32,
    /// Maximum velocity limit in degrees/second
    pub velocity_limit: f32,
}

/// Encoder telemetry data from a joint
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct EncoderTelemetry {
    /// Current position in degrees
    pub position: f32,
    /// Current velocity in degrees/second
    pub velocity: f32,
}

/// Message payload variants for the iRPC protocol
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Payload {
    // Arm → Joint Commands
    /// Set target position and velocity (only valid in Active state)
    SetTarget(SetTargetPayload),
    /// Configure the joint (Unconfigured → Inactive)
    Configure,
    /// Activate the joint (Inactive → Active)
    Activate,
    /// Deactivate the joint (Active → Inactive)
    Deactivate,
    /// Reset the joint to Unconfigured state
    Reset,

    // Joint → Arm Telemetry & Status
    /// Encoder position and velocity data
    Encoder(EncoderTelemetry),
    /// Joint status update with state and error code
    JointStatus { state: LifecycleState, error_code: u16 },

    // Bidirectional Management
    /// Acknowledgment of successful command
    Ack(MessageId),
    /// Negative acknowledgment with error code
    Nack { id: MessageId, error: u16 },
    /// Arm ready broadcast signal
    ArmReady,
}

/// Message header containing routing and correlation information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    /// Source device ID
    pub source_id: DeviceId,
    /// Target device ID (use 0x0000 for broadcast)
    pub target_id: DeviceId,
    /// Message ID for request/response correlation
    pub msg_id: MessageId,
}

/// Complete iRPC message with header and payload
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
  pub header: Header,
  pub payload: Payload,
}

/// Protocol error types
#[derive(Debug, Clone)]
#[cfg_attr(feature = "arm_api", derive(thiserror::Error))]
pub enum ProtocolError {
    /// Invalid message format
    #[cfg_attr(feature = "arm_api", error("Invalid message format"))]
    InvalidMessage,

    /// Unsupported protocol version
    #[cfg_attr(feature = "arm_api", error("Unsupported protocol version"))]
    UnsupportedVersion,

    /// Communication timeout
    #[cfg_attr(feature = "arm_api", error("Communication timeout"))]
    Timeout,

    /// General IO error
    #[cfg_attr(feature = "arm_api", error("IO error for message {0}"))]
    IoError(MessageId),

    /// Serialization error
    #[cfg_attr(feature = "arm_api", error("Serialization failed: {0}"))]
    SerializationError(String),

    /// Deserialization error
    #[cfg_attr(feature = "arm_api", error("Deserialization failed: {0}"))]
    DeserializationError(String),

    /// Invalid lifecycle state transition
    #[cfg_attr(feature = "arm_api", error("Invalid state transition"))]
    InvalidStateTransition,

    /// Hardware error
    #[cfg_attr(feature = "arm_api", error("Hardware error: {0}"))]
    HardwareError(u16),
}

impl Message {
    /// Serialize message to bytes using postcard
    pub fn serialize(&self) -> Result<Vec<u8>, ProtocolError> {
        #[cfg(feature = "arm_api")]
        {
            postcard::to_stdvec(self).map_err(|e| {
                ProtocolError::SerializationError(e.to_string())
            })
        }

        #[cfg(not(feature = "arm_api"))]
        {
            postcard::to_allocvec(self).map_err(|_| {
                ProtocolError::SerializationError(String::new())
            })
        }
    }

    /// Deserialize message from bytes using postcard
    pub fn deserialize(bytes: &[u8]) -> Result<Self, ProtocolError> {
        #[cfg(feature = "arm_api")]
        {
            postcard::from_bytes(bytes).map_err(|e| {
                ProtocolError::DeserializationError(e.to_string())
            })
        }

        #[cfg(not(feature = "arm_api"))]
        {
            postcard::from_bytes(bytes).map_err(|_| {
                ProtocolError::DeserializationError(String::new())
            })
        }
    }

    /// Get the maximum serialized size estimate (for buffer allocation)
    pub const fn max_size() -> usize {
        // Header (2 + 2 + 4 = 8 bytes) + Payload (worst case ~20 bytes) + overhead
        128
    }
}