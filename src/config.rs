// Shared constants for the iRPC protocol and application logic.

// --- Device Addressing ---
pub const BROADCAST_ADDRESS: u16 = 0x0000;
pub const ARM_DEVICE_ID: u16 = 0x0001;
pub const JOINT_ID_OFFSET: u16 = 0x0010;

// --- Communication Parameters ---
pub const REQUEST_TIMEOUT_MS: u64 = 100;
pub const MAX_RETRIES: u32 = 3;

// --- Entity Type Identifiers ---
pub const ENTITY_TYPE_JOINT_CLN17: u16 = 0x1001;
