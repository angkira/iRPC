use crate::protocol::{Message, DeviceId};
use async_trait::async_trait;

// Import Vec and Box for async trait
#[cfg(feature = "arm_api")]
use std::vec::Vec;
#[cfg(feature = "arm_api")]
use std::boxed::Box;

#[cfg(not(feature = "arm_api"))]
extern crate alloc;
#[cfg(not(feature = "arm_api"))]
use alloc::vec::Vec;
#[cfg(not(feature = "arm_api"))]
use alloc::boxed::Box;

#[derive(Debug, Clone)]
pub struct DeviceInfo {
    pub id: DeviceId,
    pub entity_type: u16,
}

#[async_trait]
pub trait CommunicationAdapter: Send + Sync {
    type Error: core::fmt::Debug;
    
    async fn transmit(&self, message: &Message) -> Result<(), Self::Error>;
    async fn receive(&self) -> Result<Option<Message>, Self::Error>;
    async fn discover_devices(&self) -> Result<Vec<DeviceInfo>, Self::Error>;
    fn is_connected(&self) -> bool;
}