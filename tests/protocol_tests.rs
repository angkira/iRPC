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
