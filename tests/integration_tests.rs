//! Integration tests for iRPC library features

use irpc::{Message, Header, Payload, SetTargetPayload, EncoderTelemetry, LifecycleState};

#[test]
fn test_message_creation() {
    let header = Header {
        source_id: 0x0001,
        target_id: 0x0010,
        msg_id: 42,
    };
    
    let set_target = Message {
        header: header.clone(),
        payload: Payload::SetTarget(SetTargetPayload {
            target_angle: 90.0,
            velocity_limit: 10.0,
        }),
    };
    
    let encoder_telemetry = Message {
        header: header.clone(),
        payload: Payload::Encoder(EncoderTelemetry {
            position: 45.0,
            velocity: 5.0,
        }),
    };
    
    let joint_status = Message {
        header,
        payload: Payload::JointStatus {
            state: LifecycleState::Active,
            error_code: 0,
        },
    };
    
    // Test that messages can be created and cloned
    let _set_target_clone = set_target.clone();
    let _encoder_clone = encoder_telemetry.clone();
    let _status_clone = joint_status.clone();
}

#[test]
fn test_lifecycle_states() {
    let states = vec![
        LifecycleState::Unconfigured,
        LifecycleState::Inactive,
        LifecycleState::Active,
        LifecycleState::Error,
    ];
    
    for state in states {
        let _state_clone = state;
        // Test that states implement required traits
        assert!(format!("{:?}", state).len() > 0);
    }
}

#[test]
fn test_payload_variants() {
    let payloads = vec![
        Payload::Configure,
        Payload::Activate,
        Payload::Deactivate,
        Payload::Reset,
        Payload::Ack(123),
        Payload::Nack { id: 456, error: 1 },
        Payload::ArmReady,
    ];
    
    for payload in payloads {
        let _payload_clone = payload.clone();
    }
}

#[cfg(feature = "joint_api")]
#[test]
fn test_joint_state_machine() {
    use irpc::Joint;
    
    let mut joint = Joint::new(0x0010);
    
    // Test initial state
    assert_eq!(joint.state(), LifecycleState::Unconfigured);
    
    // Test Configure command
    let configure_msg = Message {
        header: Header {
            source_id: 0x0001,
            target_id: 0x0010,
            msg_id: 1,
        },
        payload: Payload::Configure,
    };
    
    let response = joint.handle_message(&configure_msg);
    assert!(response.is_some());
    assert_eq!(joint.state(), LifecycleState::Inactive);
    
    // Test Activate command
    let activate_msg = Message {
        header: Header {
            source_id: 0x0001,
            target_id: 0x0010,
            msg_id: 2,
        },
        payload: Payload::Activate,
    };
    
    let response = joint.handle_message(&activate_msg);
    assert!(response.is_some());
    assert_eq!(joint.state(), LifecycleState::Active);
    
    // Test that we can handle messages when active
    let set_target_msg = Message {
        header: Header {
            source_id: 0x0001,
            target_id: 0x0010,
            msg_id: 3,
        },
        payload: Payload::SetTarget(irpc::SetTargetPayload {
            target_angle: 90.0,
            velocity_limit: 10.0,
        }),
    };
    
    let response = joint.handle_message(&set_target_msg);
    assert!(response.is_some());
    
    // Test Deactivate command
    let deactivate_msg = Message {
        header: Header {
            source_id: 0x0001,
            target_id: 0x0010,
            msg_id: 4,
        },
        payload: Payload::Deactivate,
    };
    
    let response = joint.handle_message(&deactivate_msg);
    assert!(response.is_some());
    assert_eq!(joint.state(), LifecycleState::Inactive);
    
    // Test Reset command
    let reset_msg = Message {
        header: Header {
            source_id: 0x0001,
            target_id: 0x0010,
            msg_id: 5,
        },
        payload: Payload::Reset,
    };
    
    let response = joint.handle_message(&reset_msg);
    assert!(response.is_some());
    assert_eq!(joint.state(), LifecycleState::Unconfigured);
}

#[cfg(feature = "joint_api")]
#[test]
fn test_joint_invalid_state_transitions() {
    use irpc::Joint;
    
    let mut joint = Joint::new(0x0010);
    
    // Try to activate without configuring first
    let activate_msg = Message {
        header: Header {
            source_id: 0x0001,
            target_id: 0x0010,
            msg_id: 1,
        },
        payload: Payload::Activate,
    };
    
    let response = joint.handle_message(&activate_msg);
    assert!(response.is_some());
    
    // Should remain in Unconfigured state
    assert_eq!(joint.state(), LifecycleState::Unconfigured);
    
    // Check that response is a NACK
    if let Some(resp) = response {
        match resp.payload {
            Payload::Nack { id, error } => {
                assert_eq!(id, 1);
                assert_eq!(error, 2); // Invalid state for activate
            }
            _ => panic!("Expected NACK response"),
        }
    }
}

#[cfg(feature = "joint_api")]
#[test]
fn test_joint_message_targeting() {
    use irpc::Joint;
    
    let mut joint = Joint::new(0x0010);
    
    // Message targeted to different joint should be ignored
    let msg_wrong_target = Message {
        header: Header {
            source_id: 0x0001,
            target_id: 0x0020, // Different target
            msg_id: 1,
        },
        payload: Payload::Configure,
    };
    
    let response = joint.handle_message(&msg_wrong_target);
    assert!(response.is_none());
    
    // State should remain unchanged
    assert_eq!(joint.state(), LifecycleState::Unconfigured);
}

/*
#[cfg(feature = "arm_api")]
#[tokio::test]
async fn test_arm_client() {
    use irpc::ArmClient;
    
    let client = ArmClient::new();
    let message = Message::Notification { 
        payload: vec![1, 2, 3] 
    };
    
    // Test async send
    let result = client.send_async(message).await;
    assert!(result.is_ok());
}

#[cfg(feature = "joint_api")]
#[test]
fn test_joint_client() {
    use irpc::JointClient;
    
    let mut client = JointClient::new();
    assert!(!client.is_connected());
    
    // Test connection
    let result = client.connect();
    assert!(result.is_ok());
    assert!(client.is_connected());
    
    // Test send without payload (placeholder implementation)
    let message = Message::Notification { 
        payload: vec![1, 2, 3] 
    };
    let result = client.send(message);
    assert!(result.is_ok());
    
    // Test receive (should return None in placeholder implementation)
    let result = client.try_receive();
    assert!(result.is_ok());
    assert!(result.unwrap().is_none());
    
    // Test disconnect
    client.disconnect();
    assert!(!client.is_connected());
}
*/