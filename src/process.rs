use std::{
	io::{stdin, stdout, BufRead, BufReader, BufWriter, Read, Write},
	iter::once,
	net::{TcpStream, ToSocketAddrs},
	path::Path,
	process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio},
	thread::spawn,
};

use crate::error::{Error, Result};

const BUFFER_SIZE: usize = 1024;

pub trait Process: BufRead + Write + Sized {
	fn interactive(self) -> Option<ExitStatus>;

	fn read_lines(&mut self, buf: &mut String, lines: usize) -> Result<()> {
		(0..lines)
			.map(|_| self.read_line(buf).map(|_| ()))
			.collect::<std::io::Result<Vec<_>>>()?;
		Ok(())
	}

	fn read_until<OU: From<u8>, O: Extend<OU>, P: AsRef<[u8]>>(
		&mut self,
		buf: &mut O,
		pattern: P,
		try_not_consume: bool,
	) -> Result<()> {
		let pattern = pattern.as_ref();
		let mut pattern_progress = 0;

		// Outer loop - over buffers
		loop {
			let internal_buffer = self.fill_buf()?;

			let pattern_remaining = &pattern[pattern_progress..];

			if pattern_remaining.is_empty() {
				if !try_not_consume {
					self.consume(pattern_progress);
				}
				break;
			}

			if internal_buffer.is_empty() {
				return Err(Error::PatternNotFound);
			}

			let bytes_to_compare = internal_buffer.len().min(pattern_remaining.len());

			let trunc_pattern = &pattern_remaining[..bytes_to_compare];
			let trunc_buffer = &internal_buffer[..bytes_to_compare];

			if trunc_buffer != trunc_pattern {
				pattern_progress = 0;
				buf.extend(once(trunc_buffer[0].into()));
				self.consume(1);
				continue;
			}

			pattern_progress += bytes_to_compare;
			if !try_not_consume {
				buf.extend(trunc_buffer.iter().copied().map(|x| x.into()));
			}
		}

		Ok(())
	}
}

pub struct LocalProcess {
	child: Child,
	id: u32,
	stdin: BufWriter<ChildStdin>,
	stdout: BufReader<ChildStdout>,
}

impl LocalProcess {
	pub fn new(path: impl AsRef<Path>) -> Result<Self> {
		let mut child = Command::new(path.as_ref())
			.stdin(Stdio::piped())
			.stdout(Stdio::piped())
			.spawn()?;

		let id = child.id();

		let (stdin, stdout) = match (child.stdin.take(), child.stdout.take()) {
			(Some(stdin), Some(stdout)) => (BufWriter::new(stdin), BufReader::new(stdout)),
			_ => panic!("Could not capture child IO"),
		};

		Ok(Self {
			child,
			id,
			stdin,
			stdout,
		})
	}
}

impl Read for LocalProcess {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> { self.stdout.read(buf) }
}

impl BufRead for LocalProcess {
	fn fill_buf(&mut self) -> std::io::Result<&[u8]> { self.stdout.fill_buf() }

	fn consume(&mut self, amt: usize) { self.stdout.consume(amt) }
}

impl Write for LocalProcess {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> { self.stdin.write(buf) }

	fn flush(&mut self) -> std::io::Result<()> { self.stdin.flush() }
}

impl Process for LocalProcess {
	fn interactive(self) -> Option<ExitStatus> {
		let LocalProcess {
			stdin: mut remote_stdin,
			stdout: mut remote_stdout,
			mut child,
			..
		} = self;

		let mut stdin = stdin();
		let mut stdout = stdout();

		let remote_to_local = spawn(move || {
			let mut buffer = [0u8; BUFFER_SIZE];

			while let Ok(n_bytes) = remote_stdout.read(&mut buffer) {
				if n_bytes == 0 {
					break;
				}
				stdout
					.write_all(&buffer[..n_bytes])
					.expect("could not write to stdout");
				if let Err(_) = stdout.flush() {
					break;
				}
			}
		});
		let local_to_remote = spawn(move || {
			let mut buffer = [0u8; BUFFER_SIZE];

			while let Ok(n_bytes) = stdin.read(&mut buffer) {
				remote_stdin
					.write_all(&buffer[..n_bytes])
					.expect("could not write to stdout");
				if let Err(_) = remote_stdin.flush() {
					break;
				}
			}
		});

		remote_to_local
			.join()
			.expect("One of the threads could not be joined");
		local_to_remote
			.join()
			.expect("One of the threads could not be joined");

		let exit = child.wait().expect("Child process could not be run");
		Some(exit)
	}
}

pub struct RemoteProcess {
	stdin: BufWriter<TcpStream>,
	stdout: BufReader<TcpStream>,
}

impl RemoteProcess {
	pub fn new(addr: impl ToSocketAddrs) -> Result<Self> {
		for addr in addr.to_socket_addrs()? {
			let stream = match TcpStream::connect(addr) {
				Ok(s) => s,
				Err(_) => continue,
			};
			let stream_clone = stream.try_clone()?;
			let stdin = BufWriter::new(stream);
			let stdout = BufReader::new(stream_clone);
			return Ok(Self { stdin, stdout });
		}
		Err(Error::CouldNotConnect)
	}
}

impl Read for RemoteProcess {
	fn read(&mut self, buf: &mut [u8]) -> std::io::Result<usize> { self.stdout.read(buf) }
}

impl BufRead for RemoteProcess {
	fn fill_buf(&mut self) -> std::io::Result<&[u8]> { self.stdout.fill_buf() }

	fn consume(&mut self, amt: usize) { self.stdout.consume(amt) }
}

impl Write for RemoteProcess {
	fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> { self.stdin.write(buf) }

	fn flush(&mut self) -> std::io::Result<()> { self.stdin.flush() }
}

impl Process for RemoteProcess {
	fn interactive(self) -> Option<ExitStatus> {
		let RemoteProcess {
			stdin: mut remote_stdin,
			stdout: mut remote_stdout,
		} = self;

		let mut stdin = stdin();
		let mut stdout = stdout();

		let remote_to_local = spawn(move || {
			let mut buffer = [0u8; BUFFER_SIZE];

			while let Ok(n_bytes) = remote_stdout.read(&mut buffer) {
				if n_bytes == 0 {
					break;
				}
				stdout
					.write_all(&buffer[..n_bytes])
					.expect("could not write to stdout");
				if let Err(_) = stdout.flush() {
					break;
				}
			}
		});
		let local_to_remote = spawn(move || {
			let mut buffer = [0u8; BUFFER_SIZE];

			while let Ok(n_bytes) = stdin.read(&mut buffer) {
				remote_stdin
					.write_all(&buffer[..n_bytes])
					.expect("could not write to stdout");
				if let Err(_) = remote_stdin.flush() {
					break;
				}
			}
		});

		remote_to_local
			.join()
			.expect("One of the threads could not be joined");
		local_to_remote
			.join()
			.expect("One of the threads could not be joined");

		None
	}
}
