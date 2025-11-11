# Changelog

All notable changes to iRPC will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [2.2.0] - 2025-11-11

### Added
- **Power Monitoring and Telemetry** (Phase 7)
  - `PowerMetrics` telemetry message with comprehensive power monitoring data
    - Supply voltage, phase currents, RMS current, power consumption
    - MCU temperature and thermal throttle factor
    - Energy (mWh) and charge (mAh) accumulation
    - Active time tracking
    - Fault event counters
  - `EmergencyStop` high-priority event notification
    - 8 emergency stop reason codes (overvoltage, overcurrent, overtemperature, etc.)
    - Environmental snapshot at fault time (voltage, current, temperature)
    - <10ms latency target for safety-critical notifications
  - `PowerConfig` configuration message
    - Configurable voltage limits (overvoltage/undervoltage thresholds)
    - Current limits (RMS continuous and peak)
    - Thermal thresholds (throttle start and shutdown temperatures)
    - Telemetry rate adjustment (1-100 Hz)
  - `FaultHistory` diagnostic query/response
    - Circular buffer of last 10 fault events
    - Detailed fault records with timestamp and environmental data
    - Total lifetime fault counter
  - New payload structures:
    - `FaultCounters` - Event counter struct
    - `PowerMetrics` - Main telemetry payload (~56 bytes)
    - `EmergencyReason` - Fault type enum
    - `EmergencyStop` - Event notification payload (~20 bytes)
    - `PowerConfig` - Configuration payload with defaults (~16 bytes)
    - `FaultRecord` - Individual fault event record
    - `FaultHistory` - Fault history response (~100 bytes)
  - Host API methods in `JointProxy`:
    - `configure_power()` - Set power limits and telemetry rate
    - `get_power_config()` - Query current power configuration
    - `get_fault_history()` - Retrieve fault diagnostic data
    - `set_power_telemetry_rate()` - Convenience method for rate adjustment
  - Firmware handlers in `Joint`:
    - `ConfigurePower` - Accepts configuration in any state
    - `RequestPowerConfig` - Returns current configuration
    - `RequestFaultHistory` - Returns fault history buffer

### Changed
- `Payload` enum extended with 7 new power monitoring variants:
  - `PowerMetrics(PowerMetrics)` - Periodic telemetry stream
  - `EmergencyStop(EmergencyStop)` - High-priority event
  - `ConfigurePower(PowerConfig)` - Configuration command
  - `RequestPowerConfig` - Configuration query
  - `PowerConfigResponse(PowerConfig)` - Configuration response
  - `RequestFaultHistory` - Fault history query
  - `FaultHistory(FaultHistory)` - Fault history response

### Documentation
- Power monitoring bandwidth analysis in struct documentation
- Comprehensive API documentation for all new methods
- Integration examples in method comments

### Performance
- Power telemetry overhead at 10 Hz: 0.1% CAN-FD bandwidth
- Power telemetry overhead at 100 Hz: 1.0% CAN-FD bandwidth
- Emergency stop notification: <10ms latency (safety-critical)
- All payloads fit within CAN-FD frame limits (≤128 bytes)

## [2.1.0] - 2025-10-10

### Added
- **Motor Parameter Calibration** (Phase 6)
  - `StartCalibration` command with configurable test parameters
  - `StopCalibration` command for emergency abort
  - `CalibrationStatus` telemetry (10 Hz during calibration)
  - `CalibrationResult` with identified parameters and confidence metrics
  - New `Calibrating` lifecycle state
  - Automatic identification of:
    - Rotor inertia (J)
    - Torque constant (kt)
    - Viscous damping (b)
    - Stribeck friction model (τ_c, τ_s, v_s, b_f)
  - Safety monitoring (position, velocity, current, temperature limits)
  - Confidence scoring for parameter quality
  - 5 calibration phases: Inertia, Friction, TorqueConstant, Damping, Validation

### Changed
- `LifecycleState` enum now includes `Calibrating = 3`
- `LifecycleState` enum values now explicitly numbered (Error: 3 → 4)
- `Payload` enum extended with 4 new calibration variants

### Documentation
- Added `CHANGELOG.md`
- Added `examples/calibration_example.rs`
- Added `tests/protocol_tests.rs` with comprehensive calibration tests
- Updated README.md with calibration section
- Updated state transition diagram in `LifecycleState` documentation

### Breaking Changes
- `LifecycleState::Error` enum value shifted from 3 to 4
- Existing code using numeric state values must be updated
- Serialized messages containing `LifecycleState` are not backward compatible with v2.0.x

## [2.0.0] - Previous Release

### Added
- Enhanced target with motion profiling (SetTargetV2)
- Comprehensive telemetry streaming (TelemetryStream)
- Adaptive control features (coolStep, dcStep, stallGuard)
- Configurable telemetry modes
- Motion profile types (Trapezoidal, SCurve, Adaptive)

### Changed
- Protocol v2.0 specification

## [0.1.0] - Initial Release

### Added
- Basic iRPC protocol implementation
- Lifecycle state management
- SetTarget command
- Encoder telemetry
- CAN-FD transport support
- ARM and Joint APIs
