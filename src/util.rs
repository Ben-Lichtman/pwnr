pub mod cyclic;
pub mod memory_base;

use std::io::stdin;

use crate::error::Result;

pub fn pause() {
	println!("Paused - waiting on newline");
	let mut pause = String::new();
	stdin().read_line(&mut pause).unwrap();
}

// Little-endian packing convenience functions
pub fn p8l(addr: u8) -> [u8; 1] { addr.to_le_bytes() }
pub fn u8l(bytes: [u8; 1]) -> u8 { u8::from_le_bytes(bytes) }
pub fn p16l(addr: u16) -> [u8; 2] { addr.to_le_bytes() }
pub fn u16l(bytes: [u8; 2]) -> u16 { u16::from_le_bytes(bytes) }
pub fn p32l(addr: u32) -> [u8; 4] { addr.to_le_bytes() }
pub fn u32l(bytes: [u8; 4]) -> u32 { u32::from_le_bytes(bytes) }
pub fn p64l(addr: u64) -> [u8; 8] { addr.to_le_bytes() }
pub fn u64l(bytes: [u8; 8]) -> u64 { u64::from_le_bytes(bytes) }

// Big-endian packing convenience functions
pub fn p8b(addr: u8) -> [u8; 1] { addr.to_be_bytes() }
pub fn u8b(bytes: [u8; 1]) -> u8 { u8::from_be_bytes(bytes) }
pub fn p16b(addr: u16) -> [u8; 2] { addr.to_be_bytes() }
pub fn u16b(bytes: [u8; 2]) -> u16 { u16::from_be_bytes(bytes) }
pub fn p32b(addr: u32) -> [u8; 4] { addr.to_be_bytes() }
pub fn u32b(bytes: [u8; 4]) -> u32 { u32::from_be_bytes(bytes) }
pub fn p64b(addr: u64) -> [u8; 8] { addr.to_be_bytes() }
pub fn u64b(bytes: [u8; 8]) -> u64 { u64::from_be_bytes(bytes) }

pub fn num_from_hex(buf: impl AsRef<[u8]>) -> Result<u64> {
	let parsed = std::str::from_utf8(buf.as_ref())?;
	let value = u64::from_str_radix(parsed, 16)?;
	Ok(value)
}

pub fn num_from_dec(buf: impl AsRef<[u8]>) -> Result<u64> {
	let parsed = std::str::from_utf8(buf.as_ref())?;
	let value = u64::from_str_radix(parsed, 10)?;
	Ok(value)
}
