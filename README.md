Of course. Here is a proper `README.md` file for your `iRPC` library, detailing the core idea, architecture, and API usage.

-----

# iRPC

[](https://www.google.com/search?q=https://crates.io/crates/irpc)
[](https://www.google.com/search?q=https://github.com/your-user/irpc/actions)
[](https://www.google.com/search?q=./LICENSE-MIT)

A specs-driven, asynchronous, and feature-gated RPC library for robotics communication in Rust.

-----

## \#\# The iRPC Idea 💡

Controlling a robotic arm with multiple joints requires a robust, real-time communication system. **iRPC** (industrial Remote Procedure Call) provides a high-level, gRPC-like API for this exact purpose. It's designed from the ground up to be resilient, transport-agnostic, and suitable for both high-level host controllers and resource-constrained embedded firmware.

The core philosophy is built on three pillars:

1.  **Lifecycle Management:** Joints aren't just "on" or "off." They follow a strict lifecycle (`Unconfigured` -\> `Inactive` -\> `Active`) managed by the central Arm, ensuring a safe, predictable, and recoverable system.
2.  **State-Gated Commands:** A Joint will only accept movement commands when it's in the `Active` state. This prevents unexpected behavior and enforces safety protocols at the firmware level.
3.  **Resilient Communication:** The host-side API handles message timeouts and retries automatically, abstracting away the unreliability of physical communication buses.

-----

## \#\# Features

  * **Asynchronous Host API:** Built on `tokio` for non-blocking, high-performance control.
  * **`no_std` Firmware Logic:** The `joint_api` is fully `no_std` compatible for use on embedded microcontrollers.
  * **Feature-Gated Design:** Compile only what you need by enabling the `arm_api` (for the host) or `joint_api` (for firmware).
  * **Transport Agnostic:** The `CommunicationAdapter` trait allows iRPC to run over any bus.
  * **Built-in CAN-FD Adapter:** Includes a ready-to-use `CanFdAdapter` for Linux systems using `socketcan`.
  * **Structured Error Handling:** Uses a proper `Error` enum with `thiserror` for clear and concise error management.
  * **Structured Logging:** Integrated `tracing` provides deep insight into the communication flow.

-----

## \#\# Architecture Overview

iRPC uses a clear, layered architecture that separates high-level application logic from the low-level bus communication.

```mermaid
graph TD
    subgraph Host Application (std)
        Arm[Arm Orchestrator] --> JointProxy[JointProxy];
        JointProxy --> BusManager[BusManager (Async Task)];
        BusManager --> Adapter[CommunicationAdapter Trait];
    end

    subgraph Physical Bus
        Adapter -->|Serialized Frame| CanBus[CAN-FD / SPI / etc.];
    end

    subgraph Firmware (no_std)
        CanBus -->|Serialized Frame| Joint[Joint State Machine];
    end

    style Arm fill:#cde4ff
    style Joint fill:#d5f0d5
```

  * **Arm Orchestrator:** Your top-level application object. It manages the startup sequence and dispatches high-level commands (e.g., `set_targets`).
  * **JointProxy:** A gRPC-like stub object that represents a remote `Joint`. Calling `proxy.set_target(...)` on the host transparently sends a command over the bus.
  * **BusManager:** A background `tokio` task that manages all bus I/O, including command retries, timeouts, and matching responses to requests.
  * **CommunicationAdapter:** A trait that decouples the `BusManager` from any specific hardware, allowing for mock testing and support for different buses.
  * **Joint:** The state machine running on the embedded firmware. It processes incoming commands according to its current lifecycle state.

-----

## \#\# Quickstart: API Usage

Here's how you would use the `arm_api` to initialize an arm with two joints over CAN-FD and command them to move.

```rust
use irpc::arm::Arm;
use irpc::bus::can_adapter::CanFdAdapter;
use std::sync::Arc;
use tracing_subscriber;

#[tokio::main]
async fn main() {
    // 1. Initialize logging
    tracing_subscriber::fmt::init();

    // 2. Create a hardware-specific communication adapter.
    // This connects to the "can0" interface on a Linux machine.
    let adapter = Arc::new(CanFdAdapter::new("can0").expect("Failed to open CAN socket"));

    // 3. Start up the Arm.
    // This single command handles the entire lifecycle:
    // - Spawns the communication manager
    // - Broadcasts the "ArmReady" signal
    // - Registers the 2 expected joints
    // - Configures and Activates each joint sequentially
    let arm = Arm::startup(adapter, 2).await;

    // 4. The arm is now fully operational. Send a high-level vector command.
    // This moves the first joint to 90.0 degrees and the second to -45.0 degrees.
    let angles = vec![90.0, -45.0];
    let velocities = vec![150.0, 100.0];

    if let Err(_) = arm.set_targets(angles, velocities).await {
        eprintln!("Failed to execute the movement command.");
    }
}
```

-----

## \#\# Setup

To use `iRPC` in your project, add it to your `Cargo.toml`.

#### For a Host Application (The Arm)

```toml
[dependencies]
irpc = { version = "0.1.0", features = ["arm_api"] }
tokio = { version = "1", features = ["full"] }
```

#### For Embedded Firmware (The Joint)

```toml
[dependencies]
# Use default-features = false to stay no_std compatible
irpc = { version = "0.1.0", default-features = false, features = ["joint_api"] }
```

-----

## \#\# Running Tests

The test suite is part of the `arm_api` and must be run on a host machine. Because the repository is pre-configured for embedded cross-compilation in `.cargo/config.toml`, you must specify a host target to run the tests.

```bash
# For Linux
cargo test --features arm_api --target x86_64-unknown-linux-gnu

# For macOS
cargo test --features arm_api --target x86_64-apple-darwin

# For Windows
cargo test --features arm_api --target x86_64-pc-windows-msvc
```