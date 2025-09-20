#![cfg_attr(not(feature = "std"), no_std)]

use crate::protocol::{DeviceId, LifecycleState, Message, Payload, Header};

/// Represents a single joint on the embedded device, driven by a state machine.
pub struct Joint {
    id: DeviceId,
    state: LifecycleState,
}

impl Joint {
    /// Creates a new Joint in the Unconfigured state.
    pub fn new(id: DeviceId) -> Self {
        Self {
            id,
            state: LifecycleState::Unconfigured,
        }
    }
    
    /// Returns the current lifecycle state of the Joint.
    pub fn state(&self) -> LifecycleState {
        self.state
    }

    /// The core state machine logic. Processes an incoming message and returns a response.
    /// This function is the heart of the firmware's command processing.
    pub fn handle_message(&mut self, msg: &Message) -> Option<Message> {
        // Check if the message is targeted to this joint
        if msg.header.target_id != self.id {
            return None;
        }

        let response_payload = match &msg.payload {
            Payload::Configure => {
                match self.state {
                    LifecycleState::Unconfigured => {
                        self.state = LifecycleState::Inactive;
                        Some(Payload::Ack(msg.header.msg_id))
                    }
                    _ => Some(Payload::Nack { 
                        id: msg.header.msg_id, 
                        error: 1 // Invalid state for configure
                    })
                }
            }
            Payload::Activate => {
                match self.state {
                    LifecycleState::Inactive => {
                        self.state = LifecycleState::Active;
                        Some(Payload::Ack(msg.header.msg_id))
                    }
                    _ => Some(Payload::Nack { 
                        id: msg.header.msg_id, 
                        error: 2 // Invalid state for activate
                    })
                }
            }
            Payload::Deactivate => {
                match self.state {
                    LifecycleState::Active => {
                        self.state = LifecycleState::Inactive;
                        Some(Payload::Ack(msg.header.msg_id))
                    }
                    _ => Some(Payload::Nack { 
                        id: msg.header.msg_id, 
                        error: 3 // Invalid state for deactivate
                    })
                }
            }
            Payload::Reset => {
                self.state = LifecycleState::Unconfigured;
                Some(Payload::Ack(msg.header.msg_id))
            }
            Payload::SetTarget(_target) => {
                match self.state {
                    LifecycleState::Active => {
                        // In a real implementation, this would set the target angle and velocity
                        Some(Payload::Ack(msg.header.msg_id))
                    }
                    _ => Some(Payload::Nack { 
                        id: msg.header.msg_id, 
                        error: 4 // Invalid state for set target
                    })
                }
            }
            _ => {
                // Unknown or unhandled command
                Some(Payload::Nack { 
                    id: msg.header.msg_id, 
                    error: 255 // Unknown command
                })
            }
        };

        // Create response message if we have a payload to send
        response_payload.map(|payload| Message {
            header: Header {
                source_id: self.id,
                target_id: msg.header.source_id,
                msg_id: msg.header.msg_id, // Echo back the message ID for correlation
            },
            payload,
        })
    }
}