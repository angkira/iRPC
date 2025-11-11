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
/// - Active → Calibrating (via StartCalibration)
/// - Calibrating → Active (via calibration completion)
/// - Any → Unconfigured (via Reset)
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum LifecycleState {
    /// Joint is not configured and cannot accept commands
    Unconfigured = 0,
    /// Joint is configured but not ready for motion
    Inactive = 1,
    /// Joint is active and can execute motion commands
    Active = 2,
    /// Joint is performing automatic calibration
    Calibrating = 3,
    /// Joint is in error state
    Error = 4,
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

/// Stall detection status
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum StallStatus {
    /// Normal operation
    Normal = 0,
    /// Warning: high load, might stall
    Warning = 1,
    /// Stalled: motor cannot move
    Stalled = 2,
}

/// Configure adaptive control features (v2.0 - Phase 3)
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct ConfigureAdaptivePayload {
    /// Enable coolStep (adaptive current reduction)
    pub coolstep_enable: bool,
    /// Minimum current percentage for coolStep (0.0-1.0)
    pub coolstep_min_current: f32,
    /// Load threshold for current reduction start (%)
    pub coolstep_threshold: f32,
    
    /// Enable dcStep (load-adaptive velocity derating)
    pub dcstep_enable: bool,
    /// Load threshold to start velocity derating (%)
    pub dcstep_threshold: f32,
    /// Maximum velocity derating factor (0.0-1.0)
    pub dcstep_max_derating: f32,
    
    /// Enable stallGuard (sensorless stall detection)
    pub stallguard_enable: bool,
    /// Current threshold for stall detection (A)
    pub stallguard_current_threshold: f32,
    /// Velocity threshold for stall detection (deg/s)
    pub stallguard_velocity_threshold: f32,
}

/// Adaptive control status telemetry (v2.0 - Phase 3)
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct AdaptiveStatusPayload {
    /// Estimated load percentage (0-100%)
    pub load_percent: f32,
    
    /// coolStep current scaling factor (0.0-1.0)
    pub current_scale: f32,
    /// coolStep enabled
    pub coolstep_enabled: bool,
    /// Power savings percentage from coolStep (0-100%)
    pub power_savings_percent: f32,
    /// Total energy saved (Watt-hours)
    pub energy_saved_wh: f32,
    
    /// dcStep velocity scaling factor (0.0-1.0)
    pub velocity_scale: f32,
    /// dcStep enabled
    pub dcstep_enabled: bool,
    /// dcStep derating active
    pub dcstep_derating: bool,
    
    /// stallGuard status
    pub stall_status: StallStatus,
    /// stallGuard enabled
    pub stallguard_enabled: bool,
    /// Stall detection confidence (0-100%)
    pub stall_confidence: f32,
}

/// Calibration request configuration
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq)]
pub struct CalibrationRequest {
    /// Phases to run (bitmask: bit 0 = Inertia, bit 1 = Friction, bit 2 = TorqueConstant, bit 3 = Damping, bit 4 = Validation)
    pub phases: u8,
    /// Maximum test current (A)
    pub max_current: f32,
    /// Maximum test velocity (rad/s)
    pub max_velocity: f32,
    /// Maximum position excursion from start (rad)
    pub max_position_range: f32,
    /// Safety timeout per phase (seconds)
    pub phase_timeout: f32,
    /// Return to home after completion
    pub return_home: bool,
}

impl Default for CalibrationRequest {
    fn default() -> Self {
        Self {
            phases: 0b11111,  // All phases
            max_current: 8.0,
            max_velocity: 5.0,
            max_position_range: 3.14,  // ±180°
            phase_timeout: 60.0,
            return_home: true,
        }
    }
}

/// Calibration phase identifiers
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum CalibrationPhase {
    Idle = 0,
    InertiaTest = 1,
    FrictionTest = 2,
    TorqueConstantVerification = 3,
    DampingTest = 4,
    Validation = 5,
    Complete = 6,
    Failed = 7,
}

/// Calibration status update (sent periodically during calibration)
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct CalibrationStatus {
    /// Current calibration phase
    pub phase: CalibrationPhase,
    /// Progress within current phase (0.0 - 1.0)
    pub progress: f32,
    /// Estimated time remaining (seconds)
    pub time_remaining: f32,
    /// Current position (rad)
    pub current_position: f32,
    /// Current velocity (rad/s)
    pub current_velocity: f32,
    /// Current test current (A)
    pub current_iq: f32,
}

/// Identified motor parameters
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct MotorParameters {
    /// Rotor inertia (kg·m²)
    pub inertia_J: f32,
    /// Torque constant (Nm/A)
    pub torque_constant_kt: f32,
    /// Viscous damping (Nm·s/rad)
    pub damping_b: f32,
    /// Coulomb friction (Nm)
    pub friction_coulomb: f32,
    /// Stribeck friction amplitude (Nm)
    pub friction_stribeck: f32,
    /// Stribeck velocity (rad/s)
    pub friction_vstribeck: f32,
    /// Viscous friction coefficient (Nm·s/rad)
    pub friction_viscous: f32,
}

/// Calibration confidence metrics
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct CalibrationConfidence {
    /// Overall confidence (0.0 - 1.0)
    pub overall: f32,
    /// Inertia confidence (based on measurement variance)
    pub inertia: f32,
    /// Friction model fit quality (R² score)
    pub friction: f32,
    /// Torque constant confidence
    pub torque_constant: f32,
    /// Validation tracking RMS error (rad)
    pub validation_rms: f32,
}

/// Calibration result
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct CalibrationResult {
    /// Calibration success flag
    pub success: bool,
    /// Identified motor parameters
    pub parameters: MotorParameters,
    /// Confidence metrics
    pub confidence: CalibrationConfidence,
    /// Total calibration time (seconds)
    pub total_time: f32,
    /// Error code (0 = success, non-zero = error)
    pub error_code: u16,
}

/// Fault event counters for power monitoring
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
pub struct FaultCounters {
    /// Number of overcurrent events detected
    pub overcurrent_events: u16,
    /// Number of overvoltage events detected
    pub overvoltage_events: u16,
    /// Number of undervoltage events detected
    pub undervoltage_events: u16,
    /// Number of overtemperature events detected
    pub overtemp_events: u16,
    /// Number of motor driver fault events
    pub driver_fault_events: u16,
    /// Number of emergency stop events
    pub emergency_stops: u16,
}

impl Default for FaultCounters {
    fn default() -> Self {
        Self {
            overcurrent_events: 0,
            overvoltage_events: 0,
            undervoltage_events: 0,
            overtemp_events: 0,
            driver_fault_events: 0,
            emergency_stops: 0,
        }
    }
}

/// Comprehensive power monitoring telemetry (v2.2)
///
/// Size: ~56 bytes (struct) + ~8 bytes (postcard) = ~64 bytes
/// Fits in CAN-FD frame (64 bytes data payload)
///
/// At 10 Hz streaming (default):
/// - Bandwidth: 64 bytes * 8 * 10 = 5.12 kbps
/// - CAN-FD usage: 5.12 / 5000 = 0.1% (minimal overhead)
///
/// At 100 Hz streaming (maximum):
/// - Bandwidth: 64 bytes * 8 * 100 = 51.2 kbps
/// - CAN-FD usage: 51.2 / 5000 = 1.0% (acceptable)
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct PowerMetrics {
    /// Supply bus voltage (millivolts)
    pub vbus_mv: u32,
    /// Phase A current (milliamps, signed)
    pub ia_ma: i32,
    /// Phase B current (milliamps, signed)
    pub ib_ma: i32,
    /// RMS current (milliamps)
    pub i_rms_ma: f32,
    /// Instantaneous electrical power (milliwatts)
    pub power_mw: u32,
    /// MCU die temperature (degrees Celsius)
    pub mcu_temp_c: f32,
    /// Thermal throttle factor (0.0 to 1.0, where 1.0 = no throttling)
    pub throttle_factor: f32,
    /// Accumulated energy consumption (milliwatt-hours)
    pub energy_mwh: u32,
    /// Accumulated charge (milliamp-hours)
    pub charge_mah: u32,
    /// Total active time (milliseconds)
    pub active_time_ms: u32,
    /// Fault event counters
    pub faults: FaultCounters,
}

/// Emergency stop reason codes
#[derive(Serialize, Deserialize, Debug, Clone, Copy, PartialEq, Eq)]
#[repr(u8)]
pub enum EmergencyReason {
    /// Supply voltage exceeded maximum limit
    Overvoltage = 0,
    /// Supply voltage below minimum limit
    Undervoltage = 1,
    /// Instantaneous current exceeded peak limit
    PeakOvercurrent = 2,
    /// RMS current exceeded continuous limit
    RmsOvercurrent = 3,
    /// Temperature exceeded thermal shutdown limit
    Overtemperature = 4,
    /// Motor driver hardware fault detected
    DriverFault = 5,
    /// Watchdog timer reset occurred
    WatchdogReset = 6,
    /// Manual emergency stop triggered
    ManualStop = 7,
}

/// Emergency stop event notification (v2.2)
///
/// High-priority safety notification sent immediately when fault occurs.
/// Transmission latency target: <10ms from detection to CAN bus.
///
/// Size: ~20 bytes (fits in single CAN frame)
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct EmergencyStop {
    /// Reason for emergency stop
    pub reason: EmergencyReason,
    /// Supply voltage at fault time (millivolts)
    pub vbus_mv: u32,
    /// Current at fault time (milliamps, signed)
    pub current_ma: i32,
    /// Temperature at fault time (degrees Celsius)
    pub temp_c: f32,
    /// Timestamp since boot (milliseconds)
    pub timestamp_ms: u32,
}

/// Power monitoring configuration (v2.2)
///
/// Bidirectional configuration for runtime adjustment of power limits
/// and telemetry settings.
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct PowerConfig {
    /// Overvoltage shutdown threshold (millivolts)
    pub vbus_overvoltage_mv: u32,
    /// Undervoltage shutdown threshold (millivolts)
    pub vbus_undervoltage_mv: u32,
    /// Maximum continuous RMS current (milliamps)
    pub max_rms_current_ma: u16,
    /// Maximum peak current (milliamps)
    pub max_peak_current_ma: u16,
    /// Temperature threshold to start thermal throttling (degrees Celsius)
    pub temp_throttle_start_c: u8,
    /// Temperature threshold for thermal shutdown (degrees Celsius)
    pub temp_shutdown_c: u8,
    /// Power telemetry update rate (Hz, 1-100)
    pub telemetry_rate_hz: u8,
}

impl Default for PowerConfig {
    fn default() -> Self {
        Self {
            vbus_overvoltage_mv: 50000,     // 50V
            vbus_undervoltage_mv: 8000,      // 8V
            max_rms_current_ma: 1750,        // 1.75A
            max_peak_current_ma: 2500,       // 2.5A
            temp_throttle_start_c: 70,       // 70°C
            temp_shutdown_c: 85,             // 85°C
            telemetry_rate_hz: 10,           // 10 Hz default
        }
    }
}

/// Individual fault event record for diagnostic history
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct FaultRecord {
    /// Fault type (EmergencyReason as u8)
    pub fault_type: u8,
    /// Timestamp of fault event (seconds since boot)
    pub timestamp_sec: u32,
    /// Supply voltage at fault time (millivolts)
    pub vbus_mv: u16,
    /// Current at fault time (milliamps, unsigned)
    pub current_ma: u16,
    /// Temperature at fault time (degrees Celsius, signed)
    pub temp_c: i8,
}

/// Fault history diagnostic data (v2.2)
///
/// On-demand query/response for fault diagnosis and troubleshooting.
/// Contains circular buffer of last N faults with environmental data.
///
/// Size: ~100 bytes (10 records * ~10 bytes each)
#[derive(Serialize, Deserialize, Debug, Clone, Copy)]
pub struct FaultHistory {
    /// Last 10 fault records (circular buffer, oldest overwritten)
    pub records: [FaultRecord; 10],
    /// Total lifetime fault count (all time)
    pub total_faults: u32,
    /// Number of valid records in the array (0-10)
    pub valid_count: u8,
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

    // Adaptive Control Configuration & Status (v2.0 - Phase 3)
    /// Configure adaptive control features (coolStep, dcStep, stallGuard)
    ConfigureAdaptive(ConfigureAdaptivePayload),
    /// Request immediate adaptive status
    RequestAdaptiveStatus,
    /// Adaptive control status telemetry
    AdaptiveStatus(AdaptiveStatusPayload),

    // Motor Calibration (v2.1) - Phase 6
    /// Start automatic motor parameter calibration
    StartCalibration(CalibrationRequest),
    /// Stop/abort ongoing calibration
    StopCalibration,
    /// Calibration status update (Joint → Arm, sent every 100ms during calibration)
    CalibrationStatus(CalibrationStatus),
    /// Calibration final result (Joint → Arm, sent once at end)
    CalibrationResult(CalibrationResult),

    // Power Monitoring (v2.2) - Phase 7
    /// Power metrics telemetry stream (Joint → Arm, periodic broadcast)
    PowerMetrics(PowerMetrics),
    /// Emergency stop event notification (Joint → Arm, high-priority)
    EmergencyStop(EmergencyStop),
    /// Configure power monitoring parameters (Arm → Joint)
    ConfigurePower(PowerConfig),
    /// Request current power configuration (Arm → Joint)
    RequestPowerConfig,
    /// Power configuration response (Joint → Arm)
    PowerConfigResponse(PowerConfig),
    /// Request fault history (Arm → Joint)
    RequestFaultHistory,
    /// Fault history response (Joint → Arm)
    FaultHistory(FaultHistory),

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