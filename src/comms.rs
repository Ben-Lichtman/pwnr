use anyhow::Result;

use std::io::{stdin, stdout, BufRead, BufReader, BufWriter, Read, Write};
use std::net::TcpStream;
use std::process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio};
use std::thread::spawn;

const BUFFER_CAPACITY: usize = 1024;

pub struct LocalProcess {
	child: Child,
	id: u32,
	stdin: BufWriter<ChildStdin>,
	stdout: BufReader<ChildStdout>,
}

impl LocalProcess {
	fn write(&mut self, buf: &[u8]) -> Result<()> {
		self.stdin.write_all(buf)?;
		self.stdin.flush()?;
		Ok(())
	}

	fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
		self.stdout.read_exact(buf)?;
		Ok(())
	}

	fn read_lines(&mut self, lines: usize, buf: &mut String) -> Result<()> {
		for _ in 0..lines {
			self.stdout.read_line(buf)?;
		}
		Ok(())
	}

	fn read_until(&mut self, byte: u8, inclusive: bool, buf: &mut Vec<u8>) -> Result<()> {
		self.stdout.read_until(byte, buf)?;
		if !inclusive {
			buf.pop();
		}
		Ok(())
	}

	fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<()> {
		self.stdout.read_to_end(buf)?;
		Ok(())
	}

	fn read_to_string(&mut self, str: &mut String) -> Result<()> {
		self.stdout.read_to_string(str)?;
		Ok(())
	}

	fn interactive(self) -> Result<Option<ExitStatus>> {
		let LocalProcess {
			mut child,
			id: _,
			stdin: mut proc_stdin,
			stdout: mut proc_stdout,
		} = self;

		let stdin_end = move || {
			let mut buf = [0u8; BUFFER_CAPACITY];
			let mut stdin = stdin();
			loop {
				let num_bytes = match stdin.read(&mut buf) {
					Ok(0) => break,
					Ok(n) => n,
					Err(_) => break,
				};
				if let Err(_) = proc_stdin.write(&mut buf[..num_bytes]) {
					break;
				}
				if let Err(_) = proc_stdin.flush() {
					break;
				}
			}
		};

		let stdout_end = move || {
			let mut buf = [0u8; BUFFER_CAPACITY];
			let mut stdout = stdout();
			loop {
				let num_bytes = match proc_stdout.read(&mut buf) {
					Ok(0) => break,
					Ok(n) => n,
					Err(_) => break,
				};
				if let Err(_) = stdout.write(&mut buf[..num_bytes]) {
					break;
				}
				if let Err(_) = stdout.flush() {
					break;
				}
			}
		};

		let stdin_end = spawn(stdin_end);
		let stdout_end = spawn(stdout_end);
		stdin_end.join().unwrap();
		stdout_end.join().unwrap();

		let result = child.wait()?;
		Ok(Some(result))
	}
}

pub struct RemoteProcess {
	stdin: BufWriter<TcpStream>,
	stdout: BufReader<TcpStream>,
}

impl RemoteProcess {
	fn write(&mut self, buf: &[u8]) -> Result<()> {
		self.stdin.write_all(buf)?;
		self.stdin.flush()?;
		Ok(())
	}

	fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
		self.stdout.read_exact(buf)?;
		Ok(())
	}

	fn read_lines(&mut self, lines: usize, buf: &mut String) -> Result<()> {
		for _ in 0..lines {
			self.stdout.read_line(buf)?;
		}
		Ok(())
	}

	fn read_until(&mut self, byte: u8, inclusive: bool, buf: &mut Vec<u8>) -> Result<()> {
		self.stdout.read_until(byte, buf)?;
		if !inclusive {
			buf.pop();
		}
		Ok(())
	}

	fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<()> {
		self.stdout.read_to_end(buf)?;
		Ok(())
	}

	fn read_to_string(&mut self, str: &mut String) -> Result<()> {
		self.stdout.read_to_string(str)?;
		Ok(())
	}

	pub fn interactive(self) -> Result<Option<ExitStatus>> {
		let RemoteProcess {
			stdin: mut remote_stdin,
			stdout: mut remote_stdout,
		} = self;

		let stdin_end = move || {
			let mut buf = [0u8; BUFFER_CAPACITY];
			let mut stdin = stdin();
			loop {
				let num_bytes = match stdin.read(&mut buf) {
					Ok(n) => n,
					Err(_) => break,
				};
				if let Err(_) = remote_stdin.write(&mut buf[..num_bytes]) {
					break;
				}
				if let Err(_) = remote_stdin.flush() {
					break;
				}
			}
		};

		let stdout_end = move || {
			let mut buf = [0u8; BUFFER_CAPACITY];
			let mut stdout = stdout();
			loop {
				let num_bytes = match remote_stdout.read(&mut buf) {
					Ok(n) => n,
					Err(_) => break,
				};
				if let Err(_) = stdout.write(&mut buf[..num_bytes]) {
					break;
				}
				if let Err(_) = stdout.flush() {
					break;
				}
			}
		};

		let stdin_end = spawn(stdin_end);
		let stdout_end = spawn(stdout_end);
		stdin_end.join().unwrap();
		stdout_end.join().unwrap();

		Ok(None)
	}
}

pub enum Process {
	Local(LocalProcess),
	Remote(RemoteProcess),
}

impl Process {
	pub fn new(location: &str, remote: bool) -> Result<Self> {
		match remote {
			false => {
				let mut child = Command::new(location)
					.stdin(Stdio::piped())
					.stdout(Stdio::piped())
					.spawn()?;
				let id = child.id();
				let (stdin, stdout) = match (child.stdin.take(), child.stdout.take()) {
					(Some(stdin), Some(stdout)) => (stdin, stdout),
					_ => panic!("Could not capture child IO"),
				};

				let process = Process::Local(LocalProcess {
					child,
					id,
					stdin: BufWriter::new(stdin),
					stdout: BufReader::new(stdout),
				});
				Ok(process)
			}
			true => {
				let stream = TcpStream::connect(location)?;

				let process = Process::Remote(RemoteProcess {
					stdin: BufWriter::new(stream.try_clone()?),
					stdout: BufReader::new(stream.try_clone()?),
				});
				Ok(process)
			}
		}
	}

	pub fn get_id(&self) -> Option<u32> {
		match self {
			Process::Local(p) => Some(p.id),
			Process::Remote(_) => None,
		}
	}

	pub fn write(&mut self, buf: &[u8]) -> Result<()> {
		match self {
			Process::Local(p) => p.write(buf)?,
			Process::Remote(p) => p.write(buf)?,
		}
		Ok(())
	}

	pub fn write_str(&mut self, str: &str) -> Result<()> {
		self.write(str.as_bytes())?;
		Ok(())
	}

	pub fn write_line(&mut self, str: &str) -> Result<()> {
		self.write(str.as_bytes())?;
		self.write(&[b'\n'])?;
		Ok(())
	}

	pub fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
		match self {
			Process::Local(p) => p.read_exact(buf),
			Process::Remote(p) => p.read_exact(buf),
		}
	}

	pub fn read_lines(&mut self, lines: usize, buf: &mut String) -> Result<()> {
		match self {
			Process::Local(p) => p.read_lines(lines, buf),
			Process::Remote(p) => p.read_lines(lines, buf),
		}
	}

	pub fn read_until(&mut self, byte: u8, inclusive: bool, buf: &mut Vec<u8>) -> Result<()> {
		match self {
			Process::Local(p) => p.read_until(byte, inclusive, buf),
			Process::Remote(p) => p.read_until(byte, inclusive, buf),
		}
	}

	pub fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<()> {
		match self {
			Process::Local(p) => p.read_to_end(buf),
			Process::Remote(p) => p.read_to_end(buf),
		}
	}

	pub fn read_to_string(&mut self, str: &mut String) -> Result<()> {
		match self {
			Process::Local(p) => p.read_to_string(str),
			Process::Remote(p) => p.read_to_string(str),
		}
	}

	pub fn interactive(self) -> Result<Option<ExitStatus>> {
		match self {
			Process::Local(p) => p.interactive(),
			Process::Remote(p) => p.interactive(),
		}
	}

	pub fn is_remote(&self) -> bool {
		match self {
			Process::Local(_) => false,
			Process::Remote(_) => true,
		}
	}
}
