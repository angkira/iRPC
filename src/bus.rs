//! Communication bus abstraction for iRPC

use crate::protocol::{Message, ProtocolError};

/// Trait for communication bus implementations
pub trait Bus {
    /// Send a message over the bus
    fn send(&mut self, message: Message) -> Result<(), ProtocolError>;
    
    /// Receive a message from the bus
    fn receive(&mut self) -> Result<Option<Message>, ProtocolError>;
    
    /// Check if the bus is connected
    fn is_connected(&self) -> bool;
}

/// Configuration for bus implementations
#[derive(Debug, Clone)]
pub struct BusConfig {
    /// Maximum message size in bytes
    pub max_message_size: usize,
    /// Connection timeout in milliseconds
    pub timeout_ms: u32,
}