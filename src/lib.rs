#![no_std]

pub mod config;

pub fn add(left: u64, right: u64) -> u64 {
    left + right
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn it_works() {
        let result = add(2, 2);
        assert_eq!(result, 4);
    }

    #[test]
    fn test_config_constants() {
        // Test that our config constants are accessible
        assert_eq!(config::BROADCAST_ADDRESS, 0x0000);
        assert_eq!(config::ARM_DEVICE_ID, 0x0001);
        assert_eq!(config::JOINT_ID_OFFSET, 0x0010);
        assert_eq!(config::REQUEST_TIMEOUT_MS, 100);
        assert_eq!(config::MAX_RETRIES, 3);
        assert_eq!(config::ENTITY_TYPE_JOINT_CLN17, 0x1001);
    }
}
