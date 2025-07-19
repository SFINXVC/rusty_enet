pub mod protocol;
pub mod packet;

pub const VERSION_MAJOR: u8 = 1;
pub const VERSION_MINOR: u8 = 3;
pub const VERSION_PATCH: u8 = 18;

pub fn version_create() -> u32 {
    (VERSION_MAJOR as u32) << 16 | (VERSION_MINOR as u32) << 8 | VERSION_PATCH as u32
}

pub fn get_version(version: u32) -> (u8, u8, u8) {
    (((version >> 16) & 0xff) as u8, ((version >> 8) & 0xff) as u8, (version & 0xff) as u8)
}