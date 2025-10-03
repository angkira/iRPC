//! Example of using iRPC on embedded firmware (no_std)
//!
//! This example demonstrates how to use the TransportLayer and Joint
//! on an embedded microcontroller with a CAN bus.
//!
//! Note: This is a pseudo-code example for documentation purposes.
//! In a real embedded project, you would compile this separately with
//! the proper target and linker script.

#[cfg(feature = "joint_api")]
use irpc::{Joint, TransportLayer, EmbeddedTransport, TransportError};

// ============================================================================
// Mock CAN transport for demonstration
// ============================================================================

#[cfg(feature = "joint_api")]
#[derive(Debug)]
struct CanError;

#[cfg(feature = "joint_api")]
struct MockCanBus {
    rx_buffer: [u8; 64],
    has_data: bool,
}

#[cfg(feature = "joint_api")]
impl MockCanBus {
    fn new() -> Self {
        Self {
            rx_buffer: [0u8; 64],
            has_data: false,
        }
    }
}

#[cfg(feature = "joint_api")]
impl EmbeddedTransport for MockCanBus {
    type Error = CanError;

    fn send_blocking(&mut self, data: &[u8]) -> Result<(), Self::Error> {
        // In real implementation: send data over CAN-FD
        // can_fd_transmit(data);
        Ok(())
    }

    fn receive_blocking(&mut self) -> Result<Option<&[u8]>, Self::Error> {
        // In real implementation: check CAN FIFO
        // if let Some(frame) = can_fd_receive() {
        //     let len = frame.len();
        //     self.rx_buffer[..len].copy_from_slice(frame.data());
        //     Ok(Some(&self.rx_buffer[..len]))
        // } else {
        //     Ok(None)
        // }

        if self.has_data {
            self.has_data = false;
            Ok(Some(&self.rx_buffer[..8]))
        } else {
            Ok(None)
        }
    }

    fn is_ready(&self) -> bool {
        true
    }
}

// ============================================================================
// Embedded firmware main loop (pseudo-code for documentation)
// ============================================================================

#[cfg(feature = "joint_api")]
fn embedded_main_loop() -> ! {
    // Initialize hardware (CAN, timers, etc.)
    // ...

    // Create the CAN transport
    let can_bus = MockCanBus::new();

    // Wrap it in TransportLayer for automatic serialization
    let mut transport = TransportLayer::new(can_bus);

    // Create the joint state machine
    let mut joint = Joint::new(0x0010); // Joint ID = 0x0010

    // Main loop
    loop {
        // Process incoming messages and automatically send responses
        match joint.process_transport(&mut transport) {
            Ok(true) => {
                // Message was processed successfully
            }
            Ok(false) => {
                // No message available
            }
            Err(_e) => {
                // Handle transport error
                // In real firmware: log error, increment counter, etc.
            }
        }

        // Other firmware tasks...
        // - Read encoder position
        // - Update motor control
        // - Send telemetry
        // - etc.
    }
}

// ============================================================================
// Alternative: Manual control over send/receive
// ============================================================================

#[cfg(feature = "joint_api")]
fn manual_control_example() {
    let can_bus = MockCanBus::new();
    let mut transport = TransportLayer::new(can_bus);
    let mut joint = Joint::new(0x0010);

    loop {
        // Receive and handle (but don't auto-send response)
        if let Ok(Some(response)) = joint.receive_and_handle(&mut transport) {
            // Custom logic before sending response
            // For example: log the response, add telemetry, etc.

            // Send response manually
            let _ = transport.send_message(&response);
        }

        // You can also send unsolicited messages
        // let telemetry = Message { ... };
        // transport.send_message(&telemetry)?;
    }
}

// ============================================================================
// Main function for example compilation
// ============================================================================

#[cfg(feature = "joint_api")]
fn main() {
    println!("iRPC Embedded Joint Example");
    println!("===========================");
    println!();
    println!("This example demonstrates how to use iRPC on embedded firmware.");
    println!();
    println!("In a real embedded application, you would:");
    println!("1. Initialize your CAN/SPI/UART transport");
    println!("2. Implement the EmbeddedTransport trait for your hardware");
    println!("3. Create a TransportLayer wrapping your transport");
    println!("4. Create a Joint with your device ID");
    println!("5. Call joint.process_transport() in your main loop");
    println!();
    println!("The TransportLayer automatically handles:");
    println!("- Message serialization (encoding to bytes)");
    println!("- Message deserialization (decoding from bytes)");
    println!("- Buffer management");
    println!();
    println!("Example usage:");
    println!("  let mut transport = TransportLayer::new(my_can_bus);");
    println!("  let mut joint = Joint::new(0x0010);");
    println!("  ");
    println!("  loop {{");
    println!("      joint.process_transport(&mut transport)?;");
    println!("  }}");
}

#[cfg(not(feature = "joint_api"))]
fn main() {
    println!("This example requires the 'joint_api' feature to be enabled.");
    println!("Run with: cargo run --example embedded_joint --features joint_api");
}
