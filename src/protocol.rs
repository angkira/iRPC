#![cfg_attr(not(feature = "std"), no_std)]

use serde::{Serialize, Deserialize};

pub type DeviceId = u16;
pub type MessageId = u32;

#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LifecycleState {
    Unconfigured,
    Inactive,
    Active,
    Error,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct SetTargetPayload {
    pub target_angle: f32,
    pub velocity_limit: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct EncoderTelemetry {
    pub position: f32,
    pub velocity: f32,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Payload {
    // Arm -> Joint Commands
    SetTarget(SetTargetPayload),
    Configure,
    Activate,
    Deactivate,
    Reset,

    // Joint -> Arm Telemetry & Status
    Encoder(EncoderTelemetry),
    JointStatus { state: LifecycleState, error_code: u16 },

    // Bidirectional Management
    Ack(MessageId),
    Nack { id: MessageId, error: u16 },
    ArmReady,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Header {
    pub source_id: DeviceId,
    pub target_id: DeviceId,
    pub msg_id: MessageId,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Message {
  pub header: Header,
  pub payload: Payload,
}

/// Basic error type for compatibility with existing modules
#[derive(Debug, Clone)]
pub enum ProtocolError {
    /// Invalid message format
    InvalidMessage,
    /// Unsupported protocol version
    UnsupportedVersion,
    /// Communication timeout
    Timeout,
    /// General IO error
    IoError(MessageId),
}