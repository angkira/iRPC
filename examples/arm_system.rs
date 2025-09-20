//! Example demonstrating the complete iRPC ARM system
//! 
//! This example shows how to use the ARM API to control multiple joints
//! in a robotic arm system.

#[cfg(feature = "arm_api")]
use irpc::ArmClient;

#[cfg(feature = "arm_api")]
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();
    
    // Create ARM client
    let mut arm_client = ArmClient::new();
    
    // Add joints to the system
    println!("Adding joints to ARM system...");
    arm_client.add_joint(0x0010); // Joint 1
    arm_client.add_joint(0x0020); // Joint 2
    arm_client.add_joint(0x0030); // Joint 3
    
    // Check system status
    let status = arm_client.get_system_status().await;
    println!("Initial system status:");
    for (joint_id, state) in status {
        println!("  Joint 0x{:04X}: {:?}", joint_id, state);
    }
    
    // Demonstrate joint control (would work with real hardware)
    if let Some(joint1) = arm_client.get_joint(0x0010) {
        println!("Joint 1 ID: 0x{:04X}", joint1.id());
        println!("Joint 1 state: {:?}", joint1.get_state().await);
        
        // In a real system, these operations would communicate with actual hardware:
        // joint1.configure().await?;
        // joint1.activate().await?;
        // joint1.set_target(45.0, 10.0).await?;
    }
    
    println!("ARM system ready for operation!");
    println!("Note: To see full functionality, connect real joint hardware.");
    
    Ok(())
}

#[cfg(not(feature = "arm_api"))]
fn main() {
    println!("This example requires the 'arm_api' feature to be enabled.");
    println!("Run with: cargo run --example arm_system --features arm_api");
}