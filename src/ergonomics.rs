use anyhow::Result;

use std::process::ExitStatus;

use crate::comms::Process as InnerProcess;

pub struct Process {
	inner: InnerProcess,
}

impl Process {
	pub fn new(path: &str, remote: bool) -> Result<Self> {
		let process = Process {
			inner: InnerProcess::new(path, remote)?,
		};
		Ok(process)
	}

	pub fn get_id(&self) -> Option<u32> { self.inner.get_id() }

	pub fn write(&mut self, buf: &[u8]) { self.inner.write(buf).unwrap() }

	pub fn write_str(&mut self, str: impl AsRef<str>) {
		self.inner.write_str(str.as_ref()).unwrap()
	}

	pub fn write_line(&mut self, str: impl AsRef<str>) {
		self.inner.write_line(str.as_ref()).unwrap()
	}

	pub fn read_exact(&mut self, bytes: usize) -> Vec<u8> {
		let mut buf = vec![0u8; bytes];
		self.inner.read_exact(&mut buf).unwrap();
		buf
	}

	pub fn read_lines(&mut self, lines: usize) -> String {
		let mut str = String::new();
		self.inner.read_lines(lines, &mut str).unwrap();
		str
	}

	pub fn read_until(&mut self, byte: u8, inclusive: bool) -> Vec<u8> {
		let mut buf = Vec::new();
		self.inner.read_until(byte, inclusive, &mut buf).unwrap();
		buf
	}

	pub fn read_to_end(&mut self) -> Vec<u8> {
		let mut buf = Vec::new();
		self.inner.read_to_end(&mut buf).unwrap();
		buf
	}

	pub fn read_to_string(&mut self) -> String {
		let mut str = String::new();
		self.inner.read_to_string(&mut str).unwrap();
		str
	}

	pub fn interactive(self) -> Result<Option<ExitStatus>> { self.inner.interactive() }
}
