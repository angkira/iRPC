#[cfg(test)]
mod calibration_tests {
    use irpc::protocol::*;

    #[test]
    fn test_calibration_request_serialization() {
        let request = CalibrationRequest {
            phases: 0b11111,
            max_current: 8.0,
            max_velocity: 5.0,
            max_position_range: 3.14,
            phase_timeout: 60.0,
            return_home: true,
        };

        let msg = Message {
            header: Header {
                source_id: 0x0000,
                target_id: 0x0010,
                msg_id: 42,
            },
            payload: Payload::StartCalibration(request),
        };

        // Serialize
        let bytes = msg.serialize().expect("Serialization failed");

        // Deserialize
        let decoded = Message::deserialize(&bytes).expect("Deserialization failed");

        // Verify
        match decoded.payload {
            Payload::StartCalibration(req) => {
                assert_eq!(req.phases, 0b11111);
                assert_eq!(req.max_current, 8.0);
                assert!(req.return_home);
            }
            _ => panic!("Wrong payload type"),
        }
    }

    #[test]
    fn test_calibration_status_roundtrip() {
        let status = CalibrationStatus {
            phase: CalibrationPhase::FrictionTest,
            progress: 0.65,
            time_remaining: 12.5,
            current_position: 1.2,
            current_velocity: 2.5,
            current_iq: 3.0,
        };

        let msg = Message {
            header: Header {
                source_id: 0x0010,
                target_id: 0x0000,
                msg_id: 100,
            },
            payload: Payload::CalibrationStatus(status),
        };

        let bytes = msg.serialize().unwrap();
        let decoded = Message::deserialize(&bytes).unwrap();

        match decoded.payload {
            Payload::CalibrationStatus(s) => {
                assert_eq!(s.phase, CalibrationPhase::FrictionTest);
                assert!((s.progress - 0.65).abs() < 0.01);
            }
            _ => panic!("Wrong payload"),
        }
    }

    #[test]
    fn test_calibration_result_complete() {
        let result = CalibrationResult {
            success: true,
            parameters: MotorParameters {
                inertia_J: 0.001,
                torque_constant_kt: 0.15,
                damping_b: 0.0005,
                friction_coulomb: 0.02,
                friction_stribeck: 0.01,
                friction_vstribeck: 0.1,
                friction_viscous: 0.001,
            },
            confidence: CalibrationConfidence {
                overall: 0.92,
                inertia: 0.95,
                friction: 0.88,
                torque_constant: 0.94,
                validation_rms: 0.015,
            },
            total_time: 62.5,
            error_code: 0,
        };

        let msg = Message {
            header: Header {
                source_id: 0x0010,
                target_id: 0x0000,
                msg_id: 200,
            },
            payload: Payload::CalibrationResult(result),
        };

        let bytes = msg.serialize().unwrap();
        assert!(bytes.len() < Message::max_size());

        let decoded = Message::deserialize(&bytes).unwrap();
        match decoded.payload {
            Payload::CalibrationResult(r) => {
                assert!(r.success);
                assert!((r.parameters.inertia_J - 0.001).abs() < 1e-6);
                assert!((r.confidence.overall - 0.92).abs() < 0.01);
            }
            _ => panic!("Wrong payload"),
        }
    }

    #[test]
    fn test_default_calibration_request() {
        let default = CalibrationRequest::default();
        assert_eq!(default.phases, 0b11111);
        assert_eq!(default.max_current, 8.0);
        assert_eq!(default.max_velocity, 5.0);
        assert!(default.return_home);
    }

    #[test]
    fn test_calibration_phase_values() {
        assert_eq!(CalibrationPhase::Idle as u8, 0);
        assert_eq!(CalibrationPhase::InertiaTest as u8, 1);
        assert_eq!(CalibrationPhase::Complete as u8, 6);
        assert_eq!(CalibrationPhase::Failed as u8, 7);
    }

    #[test]
    fn test_lifecycle_state_calibrating() {
        let state = LifecycleState::Calibrating;
        assert_eq!(state as u8, 3);
    }

    #[test]
    fn test_stop_calibration_roundtrip() {
        let msg = Message {
            header: Header {
                source_id: 0x0000,
                target_id: 0x0010,
                msg_id: 50,
            },
            payload: Payload::StopCalibration,
        };

        let bytes = msg.serialize().unwrap();
        let decoded = Message::deserialize(&bytes).unwrap();

        match decoded.payload {
            Payload::StopCalibration => (),
            _ => panic!("Wrong payload type"),
        }
    }
}

#[cfg(test)]
mod power_monitoring_tests {
    use irpc::protocol::*;

    #[test]
    fn test_power_metrics_serialization() {
        let metrics = PowerMetrics {
            vbus_mv: 24000,
            ia_ma: 1500,
            ib_ma: -1200,
            i_rms_ma: 1350.5,
            power_mw: 32400,
            mcu_temp_c: 45.5,
            throttle_factor: 1.0,
            energy_mwh: 1500,
            charge_mah: 850,
            active_time_ms: 120000,
            faults: FaultCounters {
                overcurrent_events: 2,
                overvoltage_events: 0,
                undervoltage_events: 1,
                overtemp_events: 0,
                driver_fault_events: 0,
                emergency_stops: 1,
            },
        };

        let msg = Message {
            header: Header {
                source_id: 0x0010,
                target_id: 0x0000,
                msg_id: 1000,
            },
            payload: Payload::PowerMetrics(metrics),
        };

        // Serialize
        let bytes = msg.serialize().expect("Serialization failed");

        // Verify size is within CAN-FD limits
        assert!(bytes.len() < Message::max_size(), "Message too large: {} bytes", bytes.len());
        assert!(bytes.len() < 128, "PowerMetrics exceeds CAN-FD frame limit");

        // Deserialize
        let decoded = Message::deserialize(&bytes).expect("Deserialization failed");

        // Verify
        match decoded.payload {
            Payload::PowerMetrics(m) => {
                assert_eq!(m.vbus_mv, 24000);
                assert_eq!(m.ia_ma, 1500);
                assert_eq!(m.ib_ma, -1200);
                assert!((m.i_rms_ma - 1350.5).abs() < 0.1);
                assert_eq!(m.power_mw, 32400);
                assert!((m.mcu_temp_c - 45.5).abs() < 0.1);
                assert_eq!(m.faults.overcurrent_events, 2);
                assert_eq!(m.faults.emergency_stops, 1);
            }
            _ => panic!("Wrong payload type"),
        }
    }

    #[test]
    fn test_emergency_stop_roundtrip() {
        let emergency = EmergencyStop {
            reason: EmergencyReason::PeakOvercurrent,
            vbus_mv: 25000,
            current_ma: 2800,
            temp_c: 72.5,
            timestamp_ms: 5000,
        };

        let msg = Message {
            header: Header {
                source_id: 0x0010,
                target_id: 0x0000,
                msg_id: 2000,
            },
            payload: Payload::EmergencyStop(emergency),
        };

        let bytes = msg.serialize().unwrap();

        // Verify small size for high-priority transmission
        assert!(bytes.len() < 64, "EmergencyStop should fit in single CAN frame");

        let decoded = Message::deserialize(&bytes).unwrap();

        match decoded.payload {
            Payload::EmergencyStop(e) => {
                assert_eq!(e.reason, EmergencyReason::PeakOvercurrent);
                assert_eq!(e.vbus_mv, 25000);
                assert_eq!(e.current_ma, 2800);
                assert!((e.temp_c - 72.5).abs() < 0.1);
                assert_eq!(e.timestamp_ms, 5000);
            }
            _ => panic!("Wrong payload type"),
        }
    }

    #[test]
    fn test_power_config_default_values() {
        let default = PowerConfig::default();

        assert_eq!(default.vbus_overvoltage_mv, 50000);
        assert_eq!(default.vbus_undervoltage_mv, 8000);
        assert_eq!(default.max_rms_current_ma, 1750);
        assert_eq!(default.max_peak_current_ma, 2500);
        assert_eq!(default.temp_throttle_start_c, 70);
        assert_eq!(default.temp_shutdown_c, 85);
        assert_eq!(default.telemetry_rate_hz, 10);
    }

    #[test]
    fn test_power_config_roundtrip() {
        let config = PowerConfig {
            vbus_overvoltage_mv: 48000,
            vbus_undervoltage_mv: 10000,
            max_rms_current_ma: 2000,
            max_peak_current_ma: 3000,
            temp_throttle_start_c: 65,
            temp_shutdown_c: 80,
            telemetry_rate_hz: 50,
        };

        let msg = Message {
            header: Header {
                source_id: 0x0000,
                target_id: 0x0010,
                msg_id: 3000,
            },
            payload: Payload::ConfigurePower(config),
        };

        let bytes = msg.serialize().unwrap();
        let decoded = Message::deserialize(&bytes).unwrap();

        match decoded.payload {
            Payload::ConfigurePower(c) => {
                assert_eq!(c.vbus_overvoltage_mv, 48000);
                assert_eq!(c.max_rms_current_ma, 2000);
                assert_eq!(c.telemetry_rate_hz, 50);
            }
            _ => panic!("Wrong payload type"),
        }
    }

    #[test]
    fn test_fault_history_serialization() {
        let mut records = [FaultRecord {
            fault_type: 0,
            timestamp_sec: 0,
            vbus_mv: 0,
            current_ma: 0,
            temp_c: 0,
        }; 10];

        // Add some sample fault records
        records[0] = FaultRecord {
            fault_type: EmergencyReason::Overvoltage as u8,
            timestamp_sec: 100,
            vbus_mv: 51000,
            current_ma: 1500,
            temp_c: 55,
        };

        records[1] = FaultRecord {
            fault_type: EmergencyReason::PeakOvercurrent as u8,
            timestamp_sec: 250,
            vbus_mv: 24000,
            current_ma: 2700,
            temp_c: 68,
        };

        let history = FaultHistory {
            records,
            total_faults: 12,
            valid_count: 2,
        };

        let msg = Message {
            header: Header {
                source_id: 0x0010,
                target_id: 0x0000,
                msg_id: 4000,
            },
            payload: Payload::FaultHistory(history),
        };

        let bytes = msg.serialize().unwrap();

        // Verify within size limits
        assert!(bytes.len() < Message::max_size());

        let decoded = Message::deserialize(&bytes).unwrap();

        match decoded.payload {
            Payload::FaultHistory(h) => {
                assert_eq!(h.total_faults, 12);
                assert_eq!(h.valid_count, 2);
                assert_eq!(h.records[0].fault_type, EmergencyReason::Overvoltage as u8);
                assert_eq!(h.records[0].vbus_mv, 51000);
                assert_eq!(h.records[1].fault_type, EmergencyReason::PeakOvercurrent as u8);
            }
            _ => panic!("Wrong payload type"),
        }
    }

    #[test]
    fn test_emergency_reason_enum_values() {
        assert_eq!(EmergencyReason::Overvoltage as u8, 0);
        assert_eq!(EmergencyReason::Undervoltage as u8, 1);
        assert_eq!(EmergencyReason::PeakOvercurrent as u8, 2);
        assert_eq!(EmergencyReason::RmsOvercurrent as u8, 3);
        assert_eq!(EmergencyReason::Overtemperature as u8, 4);
        assert_eq!(EmergencyReason::DriverFault as u8, 5);
        assert_eq!(EmergencyReason::WatchdogReset as u8, 6);
        assert_eq!(EmergencyReason::ManualStop as u8, 7);
    }

    #[test]
    fn test_fault_counters_default() {
        let counters = FaultCounters::default();

        assert_eq!(counters.overcurrent_events, 0);
        assert_eq!(counters.overvoltage_events, 0);
        assert_eq!(counters.undervoltage_events, 0);
        assert_eq!(counters.overtemp_events, 0);
        assert_eq!(counters.driver_fault_events, 0);
        assert_eq!(counters.emergency_stops, 0);
    }

    #[test]
    fn test_request_power_config_roundtrip() {
        let msg = Message {
            header: Header {
                source_id: 0x0000,
                target_id: 0x0010,
                msg_id: 5000,
            },
            payload: Payload::RequestPowerConfig,
        };

        let bytes = msg.serialize().unwrap();
        let decoded = Message::deserialize(&bytes).unwrap();

        match decoded.payload {
            Payload::RequestPowerConfig => (),
            _ => panic!("Wrong payload type"),
        }
    }

    #[test]
    fn test_power_config_response_roundtrip() {
        let config = PowerConfig::default();

        let msg = Message {
            header: Header {
                source_id: 0x0010,
                target_id: 0x0000,
                msg_id: 5001,
            },
            payload: Payload::PowerConfigResponse(config),
        };

        let bytes = msg.serialize().unwrap();
        let decoded = Message::deserialize(&bytes).unwrap();

        match decoded.payload {
            Payload::PowerConfigResponse(c) => {
                assert_eq!(c.telemetry_rate_hz, 10);
            }
            _ => panic!("Wrong payload type"),
        }
    }

    #[test]
    fn test_request_fault_history_roundtrip() {
        let msg = Message {
            header: Header {
                source_id: 0x0000,
                target_id: 0x0010,
                msg_id: 6000,
            },
            payload: Payload::RequestFaultHistory,
        };

        let bytes = msg.serialize().unwrap();
        let decoded = Message::deserialize(&bytes).unwrap();

        match decoded.payload {
            Payload::RequestFaultHistory => (),
            _ => panic!("Wrong payload type"),
        }
    }
}
