use anyhow::Result;

use std::io::stdin;
use std::num::Wrapping;

mod debruijn;

pub struct MemoryBase<T> {
	base: Wrapping<T>,
}

impl<T> MemoryBase<T>
where
	T: Copy,
	Wrapping<T>: std::ops::Sub<Output = Wrapping<T>> + std::ops::Add<Output = Wrapping<T>>,
{
	pub fn new(documented: T, leaked: T) -> Self {
		Self {
			base: Wrapping(leaked) - Wrapping(documented),
		}
	}

	pub fn documented_to_leaked(&self, documented: T) -> T { (Wrapping(documented) + self.base).0 }

	pub fn leaked_to_documented(&self, leaked: T) -> T { (Wrapping(leaked) - self.base).0 }
}

pub fn pause() {
	println!("Paused, waiting on newline");
	let mut pause = String::new();
	stdin().read_line(&mut pause).unwrap();
}

pub fn buf_to_str(buf: &[u8]) -> Result<String> {
	let parsed = std::str::from_utf8(buf)?.to_string();
	Ok(parsed)
}

pub fn p32(addr: u32) -> [u8; 4] { addr.to_le_bytes() }

pub fn u32(bytes: [u8; 4]) -> u32 { u32::from_le_bytes(bytes) }

pub fn p64(addr: u64) -> [u8; 8] { addr.to_le_bytes() }

pub fn u64(bytes: [u8; 8]) -> u64 { u64::from_le_bytes(bytes) }

pub fn u32_from_bytes(buf: &[u8]) -> Result<u32> {
	let parsed = std::str::from_utf8(buf)?;
	let value = u32::from_str_radix(parsed, 16)?;
	Ok(value)
}

pub fn u64_from_bytes(buf: &[u8]) -> Result<u64> {
	let parsed = std::str::from_utf8(buf)?;
	let value = u64::from_str_radix(parsed, 16)?;
	Ok(value)
}

pub fn cyclic(size: usize) -> String {
	debruijn::debruijn(4, 26)
		.take(size)
		.map(|x| (x + b'A') as char)
		.collect()
}

pub fn lookup(needle: &str) -> usize {
	let needle = needle
		.as_bytes()
		.iter()
		.map(|x| x - b'A')
		.collect::<Vec<_>>();
	debruijn::lookup(4, 26, &needle)
}
