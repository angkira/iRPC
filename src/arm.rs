//! ARM API module for std host environments
//! 
//! This module provides functionality for standard host environments
//! with access to std library features, async runtime, and logging.

use crate::protocol::{Message, ProtocolError};

#[cfg(feature = "arm_api")]
use tokio::sync::mpsc;

#[cfg(feature = "arm_api")]
use tracing::{info, debug};

/// ARM-specific client for host environments
#[cfg(feature = "arm_api")]
pub struct ArmClient {
    sender: mpsc::UnboundedSender<Message>,
    receiver: mpsc::UnboundedReceiver<Message>,
}

#[cfg(feature = "arm_api")]
impl ArmClient {
    /// Create a new ARM client
    pub fn new() -> Self {
        let (sender, receiver) = mpsc::unbounded_channel();
        info!("ARM client initialized");
        Self { sender, receiver }
    }
    
    /// Send a message asynchronously
    pub async fn send_async(&self, message: Message) -> Result<(), ProtocolError> {
        debug!("Sending message: {:?}", message);
        self.sender.send(message)
            .map_err(|_| ProtocolError::IoError("Channel closed".to_string()))
    }
    
    /// Receive a message asynchronously
    pub async fn receive_async(&mut self) -> Result<Option<Message>, ProtocolError> {
        Ok(self.receiver.recv().await)
    }
}

#[cfg(feature = "arm_api")]
impl Default for ArmClient {
    fn default() -> Self {
        Self::new()
    }
}