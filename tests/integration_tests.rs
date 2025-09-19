//! Integration tests for iRPC library features

use irpc::{Message, ProtocolError, PROTOCOL_VERSION};

#[test]
fn test_protocol_version() {
    assert_eq!(PROTOCOL_VERSION, "1.0.0");
}

#[test]
fn test_message_creation() {
    let request = Message::Request { 
        id: 1, 
        payload: vec![1, 2, 3] 
    };
    
    let response = Message::Response { 
        id: 1, 
        result: Ok(vec![4, 5, 6]) 
    };
    
    let notification = Message::Notification { 
        payload: vec![7, 8, 9] 
    };
    
    // Test that messages can be created and cloned
    let _request_clone = request.clone();
    let _response_clone = response.clone();
    let _notification_clone = notification.clone();
}

#[test]
fn test_protocol_error() {
    let errors = vec![
        ProtocolError::InvalidMessage,
        ProtocolError::UnsupportedVersion,
        ProtocolError::Timeout,
        ProtocolError::IoError("test error".to_string()),
    ];
    
    for error in errors {
        let _error_clone = error.clone();
    }
}

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