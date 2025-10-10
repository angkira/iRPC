//! Example: Motor Parameter Calibration
//!
//! This example demonstrates how to:
//! 1. Start motor calibration
//! 2. Monitor calibration progress
//! 3. Receive and display results
//!
//! Usage: cargo run --example calibration_example --features arm_api

use irpc::protocol::*;
use std::time::Duration;
use std::thread;

fn main() {
    println!("🔧 Motor Calibration Example\n");

    // Create calibration request
    let request = CalibrationRequest {
        phases: 0b11111,  // All phases
        max_current: 8.0,
        max_velocity: 5.0,
        max_position_range: 3.14,
        phase_timeout: 60.0,
        return_home: true,
    };

    println!("📋 Calibration Configuration:");
    println!("  Phases: 0b{:05b} (all enabled)", request.phases);
    println!("  Max current: {:.1} A", request.max_current);
    println!("  Max velocity: {:.1} rad/s", request.max_velocity);
    println!("  Position range: ±{:.1}°", request.max_position_range * 180.0 / 3.14159);
    println!("  Phase timeout: {:.0}s", request.phase_timeout);
    println!();

    // Create message
    let msg = Message {
        header: Header {
            source_id: 0x0000,  // Arm
            target_id: 0x0010,  // Joint
            msg_id: 1,
        },
        payload: Payload::StartCalibration(request),
    };

    // Serialize
    let bytes = msg.serialize().expect("Failed to serialize");
    println!("✅ Message serialized: {} bytes", bytes.len());
    println!("   CAN frames needed: {}", (bytes.len() + 7) / 8);
    println!();

    // Simulate status updates
    println!("📊 Simulating calibration progress:\n");

    let phases = vec![
        (CalibrationPhase::InertiaTest, "Inertia Test", 15.0),
        (CalibrationPhase::FrictionTest, "Friction Test", 25.0),
        (CalibrationPhase::TorqueConstantVerification, "Torque Constant Verification", 8.0),
        (CalibrationPhase::DampingTest, "Damping Test", 12.0),
        (CalibrationPhase::Validation, "Validation", 5.0),
    ];

    for (phase, name, duration) in phases {
        println!("Phase: {}", name);
        let steps = 20;
        for i in 0..=steps {
            let progress = i as f32 / steps as f32;
            let time_remaining = duration * (1.0 - progress);

            print!("\r  [");
            for j in 0..steps {
                if j < i {
                    print!("█");
                } else {
                    print!("░");
                }
            }
            print!("] {:.0}% (ETA: {:.1}s)  ", progress * 100.0, time_remaining);
            std::io::Write::flush(&mut std::io::stdout()).unwrap();

            thread::sleep(Duration::from_millis((duration * 1000.0 / steps as f32) as u64 / 10));
        }
        println!();
    }

    println!("\n✅ Calibration complete!\n");

    // Simulate result
    let result = CalibrationResult {
        success: true,
        parameters: MotorParameters {
            inertia_J: 0.001052,
            torque_constant_kt: 0.1487,
            damping_b: 0.000521,
            friction_coulomb: 0.0198,
            friction_stribeck: 0.0087,
            friction_vstribeck: 0.0953,
            friction_viscous: 0.001034,
        },
        confidence: CalibrationConfidence {
            overall: 0.91,
            inertia: 0.94,
            friction: 0.87,
            torque_constant: 0.93,
            validation_rms: 0.0172,
        },
        total_time: 63.2,
        error_code: 0,
    };

    println!("📊 Motor Parameters:");
    println!("  J  = {:.6} kg·m²", result.parameters.inertia_J);
    println!("  kt = {:.4} Nm/A", result.parameters.torque_constant_kt);
    println!("  b  = {:.6} Nm·s/rad", result.parameters.damping_b);
    println!();
    println!("📊 Friction Model:");
    println!("  τ_coulomb  = {:.4} Nm", result.parameters.friction_coulomb);
    println!("  τ_stribeck = {:.4} Nm", result.parameters.friction_stribeck);
    println!("  v_stribeck = {:.4} rad/s", result.parameters.friction_vstribeck);
    println!("  b_viscous  = {:.6} Nm·s/rad", result.parameters.friction_viscous);
    println!();
    println!("🎯 Confidence Metrics:");
    println!("  Overall:       {:.1}%", result.confidence.overall * 100.0);
    println!("  Inertia:       {:.1}%", result.confidence.inertia * 100.0);
    println!("  Friction:      {:.1}%", result.confidence.friction * 100.0);
    println!("  Torque const:  {:.1}%", result.confidence.torque_constant * 100.0);
    println!("  Validation RMS: {:.4} rad ({:.2}°)",
             result.confidence.validation_rms,
             result.confidence.validation_rms * 180.0 / 3.14159);
    println!();
    println!("⏱️  Total time: {:.1}s", result.total_time);
}
