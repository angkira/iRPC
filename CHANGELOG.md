# Changelog

All notable changes to iRPC will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

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
