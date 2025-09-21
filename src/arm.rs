//! ARM API module for std host environments
//! 
//! This module provides functionality for standard host environments
//! with access to std library features, async runtime, and logging.

use crate::protocol::{Message, ProtocolError, DeviceId, MessageId, Payload, Header, LifecycleState, SetTargetPayload};

#[cfg(feature = "arm_api")]
use tokio::sync::{mpsc, RwLock};

#[cfg(feature = "arm_api")]
use tracing::{info, debug, warn, error};

#[cfg(feature = "arm_api")]
use std::collections::HashMap;

#[cfg(feature = "arm_api")]
use std::sync::atomic::{AtomicU32, Ordering};

#[cfg(feature = "arm_api")]
use std::sync::Arc;

/// Asynchronous communication manager for ARM systems
#[cfg(feature = "arm_api")]
pub struct CommunicationManager {
    message_id_counter: AtomicU32,
    pending_responses: Arc<RwLock<HashMap<MessageId, tokio::sync::oneshot::Sender<Message>>>>,
    outbound_tx: mpsc::UnboundedSender<Message>,
    inbound_rx: Arc<RwLock<mpsc::UnboundedReceiver<Message>>>,
}

#[cfg(feature = "arm_api")]
impl CommunicationManager {
    /// Create a new communication manager
    pub fn new() -> Self {
        let (outbound_tx, _outbound_rx) = mpsc::unbounded_channel();
        let (_inbound_tx, inbound_rx) = mpsc::unbounded_channel();
        
        Self {
            message_id_counter: AtomicU32::new(1),
            pending_responses: Arc::new(RwLock::new(HashMap::new())),
            outbound_tx,
            inbound_rx: Arc::new(RwLock::new(inbound_rx)),
        }
    }
    
    /// Generate a unique message ID
    fn next_message_id(&self) -> MessageId {
        self.message_id_counter.fetch_add(1, Ordering::SeqCst)
    }
    
    /// Send a message and wait for response
    pub async fn send_and_wait(&self, target_id: DeviceId, payload: Payload) -> Result<Message, ProtocolError> {
        let msg_id = self.next_message_id();
        let (tx, rx) = tokio::sync::oneshot::channel();
        
        // Register pending response
        {
            let mut pending = self.pending_responses.write().await;
            pending.insert(msg_id, tx);
        }
        
        let message = Message {
            header: Header {
                source_id: 0x0001, // ARM controller ID
                target_id,
                msg_id,
            },
            payload,
        };
        
        // Send message
        if let Err(_) = self.outbound_tx.send(message) {
            // Remove the pending response entry on send failure
            let mut pending = self.pending_responses.write().await;
            pending.remove(&msg_id);
            return Err(ProtocolError::IoError(msg_id));
        }
        
        // Wait for response with timeout
        match tokio::time::timeout(std::time::Duration::from_secs(5), rx).await {
            Ok(Ok(msg)) => Ok(msg),
            Ok(Err(_)) => {
                // Remove the pending response entry on oneshot receive error
                let mut pending = self.pending_responses.write().await;
                pending.remove(&msg_id);
                Err(ProtocolError::IoError(msg_id))
            }
            Err(_) => {
                // Remove the pending response entry on timeout
                let mut pending = self.pending_responses.write().await;
                pending.remove(&msg_id);
                Err(ProtocolError::Timeout)
            }
        }
    
    /// Send a message without waiting for response
    pub async fn send_fire_and_forget(&self, target_id: DeviceId, payload: Payload) -> Result<(), ProtocolError> {
        let msg_id = self.next_message_id();
        
        let message = Message {
            header: Header {
                source_id: 0x0001, // ARM controller ID
                target_id,
                msg_id,
            },
            payload,
        };
        
        self.outbound_tx.send(message)
            .map_err(|_| ProtocolError::IoError(msg_id))
    }
    
    /// Process incoming message (would typically be called by background task)
    pub async fn process_incoming(&self, message: Message) {
        let msg_id = message.header.msg_id;
        
        // Check if this is a response to a pending request
        let mut pending = self.pending_responses.write().await;
        if let Some(tx) = pending.remove(&msg_id) {
            if let Err(_) = tx.send(message) {
                warn!("Failed to deliver response for message {}", msg_id);
            }
        } else {
            // Handle unsolicited message (telemetry, status updates, etc.)
            debug!("Received unsolicited message: {:?}", message);
        }
    }
}

/// High-level interface for interacting with a single joint
#[cfg(feature = "arm_api")]
pub struct JointProxy {
    joint_id: DeviceId,
    comm_manager: Arc<CommunicationManager>,
    current_state: Arc<RwLock<LifecycleState>>,
}

#[cfg(feature = "arm_api")]
impl JointProxy {
    /// Create a new joint proxy
    pub fn new(joint_id: DeviceId, comm_manager: Arc<CommunicationManager>) -> Self {
        Self {
            joint_id,
            comm_manager,
            current_state: Arc::new(RwLock::new(LifecycleState::Unconfigured)),
        }
    }
    
    /// Get the current state of the joint
    pub async fn get_state(&self) -> LifecycleState {
        *self.current_state.read().await
    }
    
    /// Configure the joint (transition from Unconfigured to Inactive)
    pub async fn configure(&self) -> Result<(), ProtocolError> {
        let response = self.comm_manager.send_and_wait(self.joint_id, Payload::Configure).await?;
        
        match response.payload {
            Payload::Ack(_) => {
                let mut state = self.current_state.write().await;
                *state = LifecycleState::Inactive;
                info!("Joint {} configured successfully", self.joint_id);
                Ok(())
            }
            Payload::Nack { id, error } => {
                error!("Joint {} configure failed: error {}", self.joint_id, error);
                Err(ProtocolError::IoError(id))
            }
            _ => Err(ProtocolError::InvalidMessage)
        }
    }
    
    /// Activate the joint (transition from Inactive to Active)
    pub async fn activate(&self) -> Result<(), ProtocolError> {
        let response = self.comm_manager.send_and_wait(self.joint_id, Payload::Activate).await?;
        
        match response.payload {
            Payload::Ack(_) => {
                let mut state = self.current_state.write().await;
                *state = LifecycleState::Active;
                info!("Joint {} activated successfully", self.joint_id);
                Ok(())
            }
            Payload::Nack { id, error } => {
                error!("Joint {} activate failed: error {}", self.joint_id, error);
                Err(ProtocolError::IoError(id))
            }
            _ => Err(ProtocolError::InvalidMessage)
        }
    }
    
    /// Deactivate the joint (transition from Active to Inactive)
    pub async fn deactivate(&self) -> Result<(), ProtocolError> {
        let response = self.comm_manager.send_and_wait(self.joint_id, Payload::Deactivate).await?;
        
        match response.payload {
            Payload::Ack(_) => {
                let mut state = self.current_state.write().await;
                *state = LifecycleState::Inactive;
                info!("Joint {} deactivated successfully", self.joint_id);
                Ok(())
            }
            Payload::Nack { id, error } => {
                error!("Joint {} deactivate failed: error {}", self.joint_id, error);
                Err(ProtocolError::IoError(id))
            }
            _ => Err(ProtocolError::InvalidMessage)
        }
    }
    
    /// Reset the joint (transition to Unconfigured from any state)
    pub async fn reset(&self) -> Result<(), ProtocolError> {
        let response = self.comm_manager.send_and_wait(self.joint_id, Payload::Reset).await?;
        
        match response.payload {
            Payload::Ack(_) => {
                let mut state = self.current_state.write().await;
                *state = LifecycleState::Unconfigured;
                info!("Joint {} reset successfully", self.joint_id);
                Ok(())
            }
            Payload::Nack { id, error } => {
                error!("Joint {} reset failed: error {}", self.joint_id, error);
                Err(ProtocolError::IoError(id))
            }
            _ => Err(ProtocolError::InvalidMessage)
        }
    }
    
    /// Set target position and velocity (only works when joint is Active)
    pub async fn set_target(&self, target_angle: f32, velocity_limit: f32) -> Result<(), ProtocolError> {
        let payload = Payload::SetTarget(SetTargetPayload {
            target_angle,
            velocity_limit,
        });
        
        let response = self.comm_manager.send_and_wait(self.joint_id, payload).await?;
        
        match response.payload {
            Payload::Ack(_) => {
                debug!("Joint {} target set: angle={}, velocity={}", 
                       self.joint_id, target_angle, velocity_limit);
                Ok(())
            }
            Payload::Nack { id, error } => {
                error!("Joint {} set target failed: error {}", self.joint_id, error);
                Err(ProtocolError::IoError(id))
            }
            _ => Err(ProtocolError::InvalidMessage)
        }
    }
    
    /// Get the joint ID
    pub fn id(&self) -> DeviceId {
        self.joint_id
    }
}
/// ARM orchestrator that coordinates multiple joints and manages the system lifecycle
#[cfg(feature = "arm_api")]
pub struct ArmOrchestrator {
    comm_manager: Arc<CommunicationManager>,
    joints: HashMap<DeviceId, JointProxy>,
    is_ready: bool,
}

#[cfg(feature = "arm_api")]
impl ArmOrchestrator {
    /// Create a new ARM orchestrator
    pub fn new() -> Self {
        Self {
            comm_manager: Arc::new(CommunicationManager::new()),
            joints: HashMap::new(),
            is_ready: false,
        }
    }
    
    /// Add a joint to the orchestrator
    pub fn add_joint(&mut self, joint_id: DeviceId) {
        let joint_proxy = JointProxy::new(joint_id, Arc::clone(&self.comm_manager));
        self.joints.insert(joint_id, joint_proxy);
        info!("Added joint {} to orchestrator", joint_id);
    }
    
    /// Get a reference to a joint proxy
    pub fn get_joint(&self, joint_id: DeviceId) -> Option<&JointProxy> {
        self.joints.get(&joint_id)
    }
    
    /// Configure all joints in the system
    pub async fn configure_all(&mut self) -> Result<(), ProtocolError> {
        info!("Configuring all joints in the system");
        
        for (joint_id, joint) in &self.joints {
            match joint.configure().await {
                Ok(_) => info!("Joint {} configured successfully", joint_id),
                Err(e) => {
                    error!("Failed to configure joint {}: {:?}", joint_id, e);
                    return Err(e);
                }
            }
        }
        
        info!("All joints configured successfully");
        Ok(())
    }
    
    /// Activate all joints in the system
    pub async fn activate_all(&mut self) -> Result<(), ProtocolError> {
        info!("Activating all joints in the system");
        
        for (joint_id, joint) in &self.joints {
            match joint.activate().await {
                Ok(_) => info!("Joint {} activated successfully", joint_id),
                Err(e) => {
                    error!("Failed to activate joint {}: {:?}", joint_id, e);
                    return Err(e);
                }
            }
        }
        
        self.is_ready = true;
        info!("ARM system is now ready - all joints activated");
        Ok(())
    }
    
    /// Deactivate all joints in the system
    pub async fn deactivate_all(&mut self) -> Result<(), ProtocolError> {
        info!("Deactivating all joints in the system");
        
        for (joint_id, joint) in &self.joints {
            match joint.deactivate().await {
                Ok(_) => info!("Joint {} deactivated successfully", joint_id),
                Err(e) => {
                    error!("Failed to deactivate joint {}: {:?}", joint_id, e);
                    // Continue with other joints even if one fails
                }
            }
        }
        
        self.is_ready = false;
        info!("All joints deactivated");
        Ok(())
    }
    
    /// Emergency stop - reset all joints immediately
    pub async fn emergency_stop(&mut self) -> Result<(), ProtocolError> {
        warn!("Emergency stop initiated - resetting all joints");
        
        for (joint_id, joint) in &self.joints {
            match joint.reset().await {
                Ok(_) => info!("Joint {} reset successfully", joint_id),
                Err(e) => {
                    error!("Failed to reset joint {} during emergency stop: {:?}", joint_id, e);
                    // Continue with other joints even if one fails
                }
            }
        }
        
        self.is_ready = false;
        warn!("Emergency stop completed");
        Ok(())
    }
    
    /// Check if the ARM system is ready (all joints active)
    pub fn is_ready(&self) -> bool {
        self.is_ready
    }
    
    /// Get the list of joint IDs in the system
    pub fn get_joint_ids(&self) -> Vec<DeviceId> {
        self.joints.keys().copied().collect()
    }
    
    /// Get system status
    pub async fn get_system_status(&self) -> HashMap<DeviceId, LifecycleState> {
        let mut status = HashMap::new();
        
        for (joint_id, joint) in &self.joints {
            let state = joint.get_state().await;
            status.insert(*joint_id, state);
        }
        
        status
    }
    
    /// Process incoming message (should be called by background task)
    pub async fn process_incoming_message(&self, message: Message) {
        self.comm_manager.process_incoming(message).await;
    }
}

/// ARM-specific client for host environments (updated to use orchestrator)
#[cfg(feature = "arm_api")]
pub struct ArmClient {
    orchestrator: ArmOrchestrator,
}

#[cfg(feature = "arm_api")]
impl ArmClient {
    /// Create a new ARM client
    pub fn new() -> Self {
        info!("ARM client initialized");
        Self { 
            orchestrator: ArmOrchestrator::new(),
        }
    }
    
    /// Add a joint to the system
    pub fn add_joint(&mut self, joint_id: DeviceId) {
        self.orchestrator.add_joint(joint_id);
    }
    
    /// Initialize the ARM system (configure and activate all joints)
    pub async fn initialize(&mut self) -> Result<(), ProtocolError> {
        info!("Initializing ARM system");
        self.orchestrator.configure_all().await?;
        self.orchestrator.activate_all().await?;
        info!("ARM system initialization complete");
        Ok(())
    }
    
    /// Shutdown the ARM system
    pub async fn shutdown(&mut self) -> Result<(), ProtocolError> {
        info!("Shutting down ARM system");
        self.orchestrator.deactivate_all().await?;
        info!("ARM system shutdown complete");
        Ok(())
    }
    
    /// Get a joint proxy for direct control
    pub fn get_joint(&self, joint_id: DeviceId) -> Option<&JointProxy> {
        self.orchestrator.get_joint(joint_id)
    }
    
    /// Emergency stop the system
    pub async fn emergency_stop(&mut self) -> Result<(), ProtocolError> {
        self.orchestrator.emergency_stop().await
    }
    
    /// Check if the system is ready
    pub fn is_ready(&self) -> bool {
        self.orchestrator.is_ready()
    }
    
    /// Get system status
    pub async fn get_system_status(&self) -> HashMap<DeviceId, LifecycleState> {
        self.orchestrator.get_system_status().await
    }
    
    /// Send a message asynchronously (legacy method for compatibility)
    pub async fn send_async(&self, message: Message) -> Result<(), ProtocolError> {
        debug!("Sending message: {:?}", message);
        // Process through orchestrator for proper message handling
        self.orchestrator.process_incoming_message(message).await;
        Ok(())
    }
    
    /// Receive a message asynchronously (legacy method for compatibility)
    pub async fn receive_async(&mut self) -> Result<Option<Message>, ProtocolError> {
        // This is a placeholder - in a real implementation this would 
        // receive from the actual communication channel
        Ok(None)
    }
}

#[cfg(feature = "arm_api")]
impl Default for ArmClient {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "arm_api")]
impl Default for ArmOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(feature = "arm_api")]
impl Default for CommunicationManager {
    fn default() -> Self {
        Self::new()
    }
}