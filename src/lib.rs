#![cfg_attr(not(feature = "std"), no_std)]

pub mod config;

pub use config::*;

#[cfg(not(feature = "std"))]
use core::result::Result;

/// Main library functionality for iRPC (Robotic node interaction protocol)
pub fn init() -> Result<(), &'static str> {
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_init() {
        assert!(init().is_ok());
    }
}
