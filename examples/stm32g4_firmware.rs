//! Example firmware for STM32G4 using iRPC with built-in CAN-FD transport
//!
//! This example demonstrates how simple firmware code becomes when iRPC
//! handles all the hardware configuration and message serialization.
//!
//! # Hardware Setup
//!
//! - STM32G431CB or similar
//! - CAN-FD transceiver connected to PA12 (TX) and PA11 (RX)
//! - Joint ID: 0x0010
//!
//! # Key Differences from Old Approach
//!
//! **OLD (firmware implements transport):**
//! ```ignore
//! // Firmware had to:
//! // 1. Implement EmbeddedTransport trait
//! // 2. Configure FDCAN registers manually
//! // 3. Handle TX/RX FIFOs
//! // 4. Manage buffers
//! impl EmbeddedTransport for CanDriver {
//!     fn send_blocking(&mut self, data: &[u8]) -> Result<(), Self::Error> {
//!         // 50+ lines of PAC register access...
//!     }
//! }
//! ```
//!
//! **NEW (iRPC provides transport):**
//! ```ignore
//! // Firmware only provides configuration:
//! let config = CanFdConfig::for_joint(0x0010);
//! let (joint, transport) = Joint::with_canfd(p.FDCAN1, p.PA12, p.PA11, config)?;
//! // Done! Hardware is configured, ready to use.
//! ```

#![no_std]
#![no_main]

#[cfg(feature = "stm32g4")]
use {
    defmt_rtt as _,
    panic_probe as _,
    embassy_executor::Spawner,
    embassy_stm32::{self as _, bind_interrupts, can::fdcan, peripherals, Config},
    embassy_time::Timer,
    irpc::{Joint, transport::{CanFdConfig, CanFdTransport}},
};

#[cfg(feature = "stm32g4")]
bind_interrupts!(struct Irqs {
    FDCAN1_IT0 => fdcan::IT0InterruptHandler<peripherals::FDCAN1>;
    FDCAN1_IT1 => fdcan::IT1InterruptHandler<peripherals::FDCAN1>;
});

#[cfg(feature = "stm32g4")]
#[embassy_executor::main]
async fn main(_spawner: Spawner) {
    // 1. Initialize embassy and clocks
    let mut config = Config::default();
    {
        use embassy_stm32::rcc::*;
        config.rcc.hse = Some(Hse {
            freq: embassy_stm32::time::Hertz(8_000_000),
            mode: HseMode::Oscillator,
        });
        config.rcc.pll = Some(Pll {
            source: PllSource::HSE,
            prediv: PllPreDiv::DIV2,
            mul: PllMul::MUL85,
            divp: None,
            divq: None,
            divr: Some(PllRDiv::DIV2), // 170 MHz sysclk
        });
        config.rcc.sys = Sysclk::PLL1_R;
    }

    let p = embassy_stm32::init(config);

    defmt::info!("🚀 iRPC STM32G4 Firmware Starting...");

    // 2. Configure CAN-FD (declarative configuration)
    let config = CanFdConfig {
        node_id: 0x0010,
        nominal_bitrate: 1_000_000,  // 1 Mbps
        data_bitrate: 5_000_000,     // 5 Mbps
    };

    // 3. Create Joint + Transport in one call
    //    iRPC handles ALL hardware configuration internally!
    let (mut joint, mut transport) = Joint::with_canfd(
        0x0010,
        p.FDCAN1,
        p.PA12,  // TX
        p.PA11,  // RX
        config,
    ).expect("❌ CAN-FD initialization failed");

    defmt::info!("✅ Joint 0x{:04X} ready with CAN-FD transport", joint.id());

    // 4. Main control loop (EXTREMELY SIMPLE)
    loop {
        // Check for incoming messages
        if let Ok(Some(msg)) = transport.receive_message() {
            defmt::debug!("📨 RX: {:?}", msg.payload);

            // Process through state machine
            if let Some(response) = joint.handle_message(&msg) {
                defmt::debug!("📤 TX: {:?}", response.payload);

                // Send response
                if let Err(e) = transport.send_message(&response) {
                    defmt::error!("❌ TX failed: {:?}", e);
                }
            }
        }

        // Other firmware tasks can go here:
        // - Read encoder
        // - Update motor control
        // - Send telemetry
        // - etc.

        Timer::after_millis(1).await;
    }
}

// ============================================================================
// Comparison: Lines of Code
// ============================================================================
//
// OLD approach (firmware implements transport):
// - can.rs driver: ~200 lines (FDCAN registers, buffers, etc.)
// - main.rs: ~50 lines (create driver, implement trait, etc.)
// - TOTAL: ~250 lines of low-level code
//
// NEW approach (iRPC provides transport):
// - main.rs: ~30 lines (config + loop)
// - TOTAL: ~30 lines
//
// 🎯 Result: 8x less code in firmware!
//           Hardware complexity moved to reusable library.

// Main function when stm32g4 feature is not enabled
#[cfg(not(feature = "stm32g4"))]
fn main() {
    // This example is for embedded only, compile with --target and --features stm32g4
}

#[cfg(all(not(feature = "stm32g4"), not(test)))]
#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    loop {}
}
