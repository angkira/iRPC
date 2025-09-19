# iRPC - Robotic Node Interaction Protocol

iRPC is a Rust embedded library designed for robotic node interaction protocol targeting ARM Cortex-M processors (thumbv7em-none-eabihf). This is a `no_std` library that can run on bare metal embedded systems.

Always reference these instructions first and fallback to search or bash commands only when you encounter unexpected information that does not match the info here.

## Working Effectively

### Prerequisites and Setup
- Rust stable toolchain is pinned via `rust-toolchain.toml`
- Embedded ARM target `thumbv7em-none-eabihf` is required for cross-compilation
- Install the target: `rustup target add thumbv7em-none-eabihf`

### Build and Test Commands
Execute these commands in the repository root:

#### Basic Development Workflow
- `cargo check` -- Fast syntax and type checking. Takes ~3 seconds. NEVER CANCEL.
- `cargo clippy` -- Linting analysis. Takes ~1 second. NEVER CANCEL.
- `cargo fmt` -- Code formatting. Takes ~0.4 seconds.
- `cargo test` -- Unit tests (host target only with std). Takes ~5 seconds. NEVER CANCEL.
- `cargo doc` -- Generate documentation. Takes ~2 seconds. NEVER CANCEL.

#### Embedded Target Builds
- `cargo build --target thumbv7em-none-eabihf` -- Debug build for embedded target. Takes ~0.1 seconds.
- `cargo build --target thumbv7em-none-eabihf --release` -- Optimized embedded build. Takes ~0.1 seconds.
- `cargo clippy --target thumbv7em-none-eabihf` -- Linting for embedded target. Takes ~2 seconds. NEVER CANCEL.

#### Clean Builds
- `cargo clean` -- Remove all build artifacts
- Full clean embedded build: `cargo clean && cargo build --target thumbv7em-none-eabihf --release` -- Takes ~0.1 seconds total

### Key Configuration Files
- `Cargo.toml` -- Main project configuration with embedded-specific profile settings
- `rust-toolchain.toml` -- Pins Rust stable toolchain
- `.cargo/config.toml` -- Embedded target configuration (thumbv7em-none-eabihf commented out by default for tests)
- `src/lib.rs` -- Main library with conditional `no_std` support
- `src/config.rs` -- Protocol constants and device addressing

### Build Targets and Features
- **Default (host)**: Builds with `std` for development and testing
- **Embedded**: Uses `no_std` when building for `thumbv7em-none-eabihf` target
- **Features**: `std` feature available for host development

## Validation Scenarios

### Always Run These Validation Steps After Changes
1. **Format and lint check**: `cargo fmt && cargo clippy`
2. **Host development validation**: `cargo test` -- Ensures library works with std
3. **Embedded target validation**: `cargo build --target thumbv7em-none-eabihf --release` -- Ensures no_std compatibility
4. **Documentation check**: `cargo doc` -- Validates public API documentation

### Complete Validation Workflow
Run this complete sequence before committing changes:
```bash
cargo fmt
cargo clippy
cargo test
cargo build --target thumbv7em-none-eabihf --release
cargo doc
```
Total time: ~11 seconds. NEVER CANCEL any of these steps.

### Manual Testing Scenarios
After making changes to the protocol or device addressing:
1. Verify constants in `src/config.rs` are correctly typed
2. Test library initialization with `cargo test` to ensure `init()` function works
3. Build for embedded target to verify `no_std` compatibility
4. Check generated documentation for API correctness

## Development Environment

### Essential Tools Available
- Rust stable toolchain (rustc 1.89.0)
- Cargo package manager
- Embedded ARM target support (thumbv7em-none-eabihf)
- Standard development tools (clippy, rustfmt, rustdoc)

### File Structure
```
.
├── .cargo/config.toml          # Embedded build configuration
├── .github/                    # GitHub configuration and workflows
├── .gitignore                  # Ignore build artifacts and editor files
├── Cargo.toml                  # Package configuration with embedded profiles
├── LICENSE                     # GPL-3.0 license
├── README.md                   # Project description
├── rust-toolchain.toml         # Rust toolchain pinning
└── src/
    ├── lib.rs                  # Main library with conditional no_std
    └── config.rs               # Protocol constants and addressing
```

### Key Architecture Notes
- **no_std compatibility**: Library is designed for embedded systems without std
- **Conditional compilation**: Uses `#![cfg_attr(not(feature = "std"), no_std)]` for flexibility
- **ARM Cortex-M target**: Optimized for `thumbv7em-none-eabihf` (Cortex-M4/M7 with hardware float)
- **Device addressing**: Constants defined for broadcast, ARM device, and joint addressing
- **Communication parameters**: Configurable timeouts and retry logic

### Build Profiles
- **Dev profile**: `panic = "abort"` for embedded compatibility
- **Release profile**: Optimized for size with LTO enabled, `opt-level = "s"`

### Common Development Tasks

#### Adding New Protocol Features
1. Add constants to `src/config.rs` for new device types or parameters
2. Implement functionality in `src/lib.rs` with proper error handling
3. Always maintain `no_std` compatibility
4. Add tests that can run with the `std` feature
5. Run full validation workflow

#### Debugging Build Issues
- For embedded build errors: Check `.cargo/config.toml` target configuration
- For test failures: Ensure tests are running with std support (without embedded target)
- For dependency issues: Verify `no_std` compatibility of all dependencies

#### Performance Considerations
- Release builds are optimized for code size (`opt-level = "s"`)
- LTO is enabled for minimal binary size on embedded targets
- Single codegen unit for maximum optimization

### Repository Status
This is an embedded Rust library in active development for robotic node interaction protocol. The build system is validated and working correctly for both host development and embedded targets.

Always build and test your changes with both the host target (for tests) and the embedded target (for deployment compatibility).