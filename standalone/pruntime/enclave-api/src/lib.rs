#![no_std]

pub mod actions {
    pub const ACTION_TEST: u8 = 0;
    pub const ACTION_INIT_RUNTIME: u8 = 1;
    pub const ACTION_GET_INFO: u8 = 2;
    pub const ACTION_DUMP_STATES: u8 = 3;
    pub const ACTION_LOAD_STATES: u8 = 4;
    pub const ACTION_SYNC_HEADER: u8 = 5;
    pub const ACTION_QUERY: u8 = 6;
    pub const ACTION_DISPATCH_BLOCK: u8 = 7;
    // Reserved: 8, 9
    pub const ACTION_GET_RUNTIME_INFO: u8 = 10;
    pub const ACTION_SET: u8 = 21;
    pub const ACTION_GET: u8 = 22;
    pub const ACTION_GET_EGRESS_MESSAGES: u8 = 23;
    pub const ACTION_TEST_INK: u8 = 100;
}