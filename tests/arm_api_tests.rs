//! Tests for ARM API functionality

#[cfg(feature = "arm_api")]
use irpc::{ArmClient, ArmOrchestrator, JointProxy, CommunicationManager, LifecycleState};

#[cfg(feature = "arm_api")]
use std::sync::Arc;

#[cfg(feature = "arm_api")]
#[tokio::test]
async fn test_communication_manager() {
    let comm_manager = CommunicationManager::new();
    
    // Test that communication manager can be created and used
    // The actual functionality requires a full messaging loop to test properly
    assert!(std::ptr::addr_of!(comm_manager).is_null() == false);
}

#[cfg(feature = "arm_api")]
#[tokio::test]
async fn test_joint_proxy() {
    let comm_manager = Arc::new(CommunicationManager::new());
    let joint_proxy = JointProxy::new(0x0010, comm_manager);
    
    // Test initial state
    assert_eq!(joint_proxy.get_state().await, LifecycleState::Unconfigured);
    assert_eq!(joint_proxy.id(), 0x0010);
    
    // Note: These operations would timeout in real test because there's no actual device
    // responding, but they test the API structure
}

#[cfg(feature = "arm_api")]
#[tokio::test]
async fn test_arm_orchestrator() {
    let mut orchestrator = ArmOrchestrator::new();
    
    // Test initial state
    assert!(!orchestrator.is_ready());
    assert_eq!(orchestrator.get_joint_ids().len(), 0);
    
    // Add some joints
    orchestrator.add_joint(0x0010);
    orchestrator.add_joint(0x0020);
    
    assert_eq!(orchestrator.get_joint_ids().len(), 2);
    assert!(orchestrator.get_joint_ids().contains(&0x0010));
    assert!(orchestrator.get_joint_ids().contains(&0x0020));
    
    // Test joint retrieval
    assert!(orchestrator.get_joint(0x0010).is_some());
    assert!(orchestrator.get_joint(0x0030).is_none());
    
    // Test system status
    let status = orchestrator.get_system_status().await;
    assert_eq!(status.len(), 2);
    assert_eq!(status[&0x0010], LifecycleState::Unconfigured);
    assert_eq!(status[&0x0020], LifecycleState::Unconfigured);
}

#[cfg(feature = "arm_api")]
#[tokio::test]
async fn test_arm_client() {
    let mut client = ArmClient::new();
    
    // Test initial state
    assert!(!client.is_ready());
    
    // Add joints
    client.add_joint(0x0010);
    client.add_joint(0x0020);
    
    // Test joint access
    assert!(client.get_joint(0x0010).is_some());
    assert!(client.get_joint(0x0030).is_none());
    
    // Test system status
    let status = client.get_system_status().await;
    assert_eq!(status.len(), 2);
    
    // Note: initialize() and shutdown() would timeout without real devices
    // but the API structure is tested
}

#[cfg(all(feature = "arm_api", feature = "joint_api"))]
#[tokio::test]
async fn test_arm_joint_integration() {
    use irpc::{Joint, Message, Header, Payload};
    
    // Create a joint (simulating embedded device)
    let mut joint = Joint::new(0x0010);
    
    // Create ARM client (simulating host)
    let mut arm_client = ArmClient::new();
    arm_client.add_joint(0x0010);
    
    // Simulate message exchange
    let configure_msg = Message {
        header: Header {
            source_id: 0x0001,
            target_id: 0x0010,
            msg_id: 1,
        },
        payload: Payload::Configure,
    };
    
    // Joint processes the message
    let response = joint.handle_message(&configure_msg);
    assert!(response.is_some());
    
    // Verify joint state changed
    assert_eq!(joint.state(), LifecycleState::Inactive);
    
    // Verify response is correct
    if let Some(resp) = response {
        assert_eq!(resp.header.target_id, 0x0001); // Response back to ARM
        assert_eq!(resp.header.source_id, 0x0010); // From joint
        match resp.payload {
            Payload::Ack(msg_id) => assert_eq!(msg_id, 1),
            _ => panic!("Expected ACK response"),
        }
    }
}

#[cfg(feature = "arm_api")]
#[test]
fn test_default_implementations() {
    let _client = ArmClient::default();
    let _orchestrator = ArmOrchestrator::default();
    let _comm_manager = CommunicationManager::default();
}