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