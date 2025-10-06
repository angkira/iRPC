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

/// Target position and velocity for joint motion (v1.0)
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct SetTargetPayload {
    /// Target angle in degrees
    pub target_angle: f32,
    /// Maximum velocity limit in degrees/second
    pub velocity_limit: f32,
}

/// Enhanced target with motion profiling (v2.0)
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct SetTargetPayloadV2 {
    /// Target angle in degrees
    pub target_angle: f32,
    /// Maximum velocity in degrees/second
    pub max_velocity: f32,
    /// Target velocity at end point (for fly-by waypoints) in degrees/second
    pub target_velocity: f32,
    /// Maximum acceleration in degrees/second²
    pub max_acceleration: f32,
    /// Maximum deceleration in degrees/second²
    pub max_deceleration: f32,
    /// Maximum jerk (optional, for S-curve) in degrees/second³
    /// Use 0.0 or negative value to disable jerk limiting
    pub max_jerk: f32,
    /// Motion profile type to use
    pub profile: MotionProfile,
    /// Maximum current limit (optional, use 0.0 to disable) in amperes
    pub max_current: f32,
    /// Maximum temperature limit (optional, use 0.0 to disable) in celsius
    pub max_temperature: f32,
}

/// Motion profile type for trajectory generation
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum MotionProfile {
    /// Trapezoidal velocity profile - constant acceleration/deceleration
    Trapezoidal = 0,
    /// S-curve velocity profile - jerk-limited for smooth motion
    SCurve = 1,
    /// Adaptive profile - adjusts to load conditions (future)
    Adaptive = 2,
}

/// Encoder telemetry data from a joint (v1.0 - basic)
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct EncoderTelemetry {
    /// Current position in degrees
    pub position: f32,
    /// Current velocity in degrees/second
    pub velocity: f32,
}

/// Comprehensive telemetry stream (v2.0)
///
/// Size: 64 bytes (struct) + ~10 bytes (postcard) = ~74 bytes
/// Fits in CAN-FD frame (64 bytes data payload)
///
/// At 1 kHz streaming:
/// - Bandwidth: 74 bytes * 8 * 1000 = 592 kbps
/// - CAN-FD usage: 592 / 5000 = 11.8% (plenty of headroom)
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct TelemetryStream {
    /// Timestamp in microseconds since boot
    pub timestamp_us: u64,
    
    // Motion state
    /// Current position in degrees
    pub position: f32,
    /// Current velocity in degrees/second
    pub velocity: f32,
    /// Current acceleration in degrees/second² (calculated)
    pub acceleration: f32,
    
    // FOC state (Clarke-Park transformed currents/voltages)
    /// D-axis current in amperes
    pub current_d: f32,
    /// Q-axis current in amperes (torque-producing)
    pub current_q: f32,
    /// D-axis voltage in volts
    pub voltage_d: f32,
    /// Q-axis voltage in volts
    pub voltage_q: f32,
    
    // Derived metrics
    /// Estimated torque in Newton-meters
    pub torque_estimate: f32,
    /// Electrical power in watts
    pub power: f32,
    /// Load percentage (0-100%)
    pub load_percent: f32,
    
    // Performance metrics
    /// FOC loop execution time in microseconds
    pub foc_loop_time_us: u16,
    /// Motor/driver temperature in Celsius
    pub temperature_c: f32,
    
    // Status flags
    /// Warning flags bitmap
    pub warnings: u16,
    /// Is trajectory currently active?
    pub trajectory_active: bool,
}

/// Telemetry streaming mode
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum TelemetryMode {
    /// Send telemetry only on explicit request
    OnDemand = 0,
    /// Send periodically at configured rate
    Periodic = 1,
    /// Continuous streaming at maximum rate (1 kHz)
    Streaming = 2,
    /// Send only when values change significantly
    OnChange = 3,
    /// Adapt rate based on motion activity
    Adaptive = 4,
}

/// Configure telemetry streaming
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ConfigureTelemetryPayload {
    /// Streaming mode
    pub mode: TelemetryMode,
    /// Update rate in Hz (for Periodic mode, 0 = use default)
    pub rate_hz: u16,
    /// Change threshold (for OnChange mode, 0.0 = use default)
    pub change_threshold: f32,
}

/// Message payload variants for the iRPC protocol
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum Payload {
    // Arm → Joint Commands (v1.0)
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

    // Arm → Joint Commands (v2.0)
    /// Set target with motion profiling (enhanced version)
    SetTargetV2(SetTargetPayloadV2),

    // Joint → Arm Telemetry & Status (v1.0)
    /// Encoder position and velocity data (basic)
    Encoder(EncoderTelemetry),
    /// Joint status update with state and error code
    JointStatus { state: LifecycleState, error_code: u16 },
    
    // Joint → Arm Telemetry & Status (v2.0)
    /// Comprehensive telemetry stream
    TelemetryStream(TelemetryStream),
    
    // Telemetry Configuration (v2.0)
    /// Configure telemetry streaming mode
    ConfigureTelemetry(ConfigureTelemetryPayload),
    /// Request immediate telemetry (for OnDemand mode)
    RequestTelemetry,

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