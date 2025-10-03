use crate::protocol::{DeviceId, LifecycleState, Message, Payload, Header};

/// Represents a single joint on the embedded device, driven by a state machine.
///
/// This is the firmware-side implementation that processes incoming commands
/// and enforces lifecycle state transitions. Designed for `no_std` embedded use.
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

    /// Get the joint ID
    pub fn id(&self) -> DeviceId {
        self.id
    }

    /// Create a Joint with CAN-FD transport (STM32G4 only)
    ///
    /// This is a convenience constructor that creates both the Joint state machine
    /// and the CAN-FD transport, fully configured and ready to use.
    ///
    /// # Example
    ///
    /// ```no_run
    /// use irpc::{Joint, transport::CanFdConfig};
    /// use embassy_stm32::{bind_interrupts, can, peripherals};
    ///
    /// // Define interrupt handlers
    /// bind_interrupts!(struct Irqs {
    ///     FDCAN1_IT0 => can::IT0InterruptHandler<peripherals::FDCAN1>;
    ///     FDCAN1_IT1 => can::IT1InterruptHandler<peripherals::FDCAN1>;
    /// });
    ///
    /// let config = CanFdConfig::for_joint(0x0010);
    ///
    /// let (mut joint, mut transport) = Joint::with_canfd(
    ///     0x0010,
    ///     peripherals.FDCAN1,
    ///     peripherals.PA11,  // RX
    ///     peripherals.PA12,  // TX
    ///     Irqs,
    ///     config,
    /// ).expect("CAN-FD init");
    ///
    /// loop {
    ///     if let Some(msg) = transport.receive_message()? {
    ///         if let Some(resp) = joint.handle_message(&msg) {
    ///             transport.send_message(&resp)?;
    ///         }
    ///     }
    /// }
    /// ```
    #[cfg(feature = "stm32g4")]
    pub fn with_canfd<'d, T, TX, RX, I>(
        device_id: DeviceId,
        fdcan: embassy_stm32::Peri<'d, T>,
        rx_pin: embassy_stm32::Peri<'d, RX>,
        tx_pin: embassy_stm32::Peri<'d, TX>,
        irqs: I,
        config: crate::transport::CanFdConfig,
    ) -> Result<(Self, crate::transport::CanFdTransport<'d>), crate::transport::CanError>
    where
        T: embassy_stm32::can::Instance,
        TX: embassy_stm32::can::TxPin<T>,
        RX: embassy_stm32::can::RxPin<T>,
        I: embassy_stm32::interrupt::typelevel::Binding<T::IT0Interrupt, embassy_stm32::can::IT0InterruptHandler<T>>
            + embassy_stm32::interrupt::typelevel::Binding<T::IT1Interrupt, embassy_stm32::can::IT1InterruptHandler<T>>
            + 'd,
    {
        let joint = Self::new(device_id);
        let transport = crate::transport::CanFdTransport::new(fdcan, rx_pin, tx_pin, irqs, config)?;

        Ok((joint, transport))
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

// ============================================================================
// Transport integration helpers (joint_api only)
// ============================================================================

#[cfg(feature = "joint_api")]
use crate::bus::{TransportLayer, EmbeddedTransport, TransportError};

#[cfg(feature = "joint_api")]
impl Joint {
    /// Process incoming messages from transport and send responses automatically
    ///
    /// This is a convenience method that combines receive, handle, and send operations.
    /// It polls the transport, processes any received message through the state machine,
    /// and automatically sends the response if one is generated.
    ///
    /// # Example
    /// ```no_run
    /// use irpc::{Joint, TransportLayer};
    ///
    /// let mut joint = Joint::new(0x0010);
    /// let mut transport = TransportLayer::new(my_can_bus);
    ///
    /// loop {
    ///     // Process one message (if available) and send response
    ///     if let Err(e) = joint.process_transport(&mut transport) {
    ///         // Handle transport error
    ///     }
    /// }
    /// ```
    pub fn process_transport<T: EmbeddedTransport>(
        &mut self,
        transport: &mut TransportLayer<T>,
    ) -> Result<bool, TransportError<T::Error>> {
        // Try to receive a message
        if let Some(msg) = transport.receive_message()? {
            // Process it through the state machine
            if let Some(response) = self.handle_message(&msg) {
                // Send the response
                transport.send_message(&response)?;
                return Ok(true); // Message was processed
            }
        }
        Ok(false) // No message or not for us
    }

    /// Convenience method: receive and handle message (without auto-response)
    ///
    /// This allows you to control when/how responses are sent.
    pub fn receive_and_handle<T: EmbeddedTransport>(
        &mut self,
        transport: &mut TransportLayer<T>,
    ) -> Result<Option<Message>, TransportError<T::Error>> {
        if let Some(msg) = transport.receive_message()? {
            Ok(self.handle_message(&msg))
        } else {
            Ok(None)
        }
    }
}