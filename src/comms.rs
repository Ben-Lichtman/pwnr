use anyhow::Result;

use std::process::{ExitStatus, Stdio};

use tokio::prelude::*;

use tokio::io::{stdin, stdout, BufReader, BufStream, BufWriter};
use tokio::join;
use tokio::net::TcpStream;
use tokio::process::{Child, ChildStdin, ChildStdout, Command};

const BUFFER_CAPACITY: usize = 1024;

pub struct LocalProcess {
	child: Child,
	id: u32,
	stdin: BufWriter<ChildStdin>,
	stdout: BufReader<ChildStdout>,
}

impl LocalProcess {
	async fn write(&mut self, buf: &[u8]) -> Result<()> {
		self.stdin.write_all(buf).await?;
		self.stdin.flush().await?;
		Ok(())
	}

	async fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
		self.stdout.read_exact(buf).await?;
		Ok(())
	}

	async fn read_lines(&mut self, lines: usize, buf: &mut String) -> Result<()> {
		for _ in 0..lines {
			self.stdout.read_line(buf).await?;
		}
		Ok(())
	}

	async fn read_until(&mut self, byte: u8, inclusive: bool, buf: &mut Vec<u8>) -> Result<()> {
		self.stdout.read_until(byte, buf).await?;
		if !inclusive {
			buf.pop();
		}
		Ok(())
	}

	async fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<()> {
		self.stdout.read_to_end(buf).await?;
		Ok(())
	}

	async fn read_to_string(&mut self, str: &mut String) -> Result<()> {
		self.stdout.read_to_string(str).await?;
		Ok(())
	}

	async fn interactive(self) -> Result<Option<ExitStatus>> {
		let (child, mut proc_stdin, mut proc_stdout) = (
			self.child,
			self.stdin.into_inner(),
			self.stdout.into_inner(),
		);

		let stdin_end = async move {
			let mut buf = [0u8; BUFFER_CAPACITY];
			let mut stdin = stdin();
			loop {
				let num_bytes = match stdin.read(&mut buf).await {
					Ok(n) => n,
					Err(_) => break,
				};
				if let Err(_) = proc_stdin.write(&mut buf[..num_bytes]).await {
					break;
				}
			}
		};

		let stdout_end = async move {
			let mut buf = [0u8; BUFFER_CAPACITY];
			let mut stdout = stdout();
			loop {
				let num_bytes = match proc_stdout.read(&mut buf).await {
					Ok(n) => n,
					Err(_) => break,
				};
				if let Err(_) = stdout.write(&mut buf[..num_bytes]).await {
					break;
				}
			}
		};

		tokio::spawn(stdin_end);
		tokio::spawn(stdout_end);

		let result = child.await?;
		Ok(Some(result))
	}
}

pub struct RemoteProcess {
	stream: BufStream<TcpStream>,
}

impl RemoteProcess {
	async fn write(&mut self, buf: &[u8]) -> Result<()> {
		self.stream.write_all(buf).await?;
		self.stream.flush().await?;
		Ok(())
	}

	async fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
		self.stream.read_exact(buf).await?;
		Ok(())
	}

	async fn read_lines(&mut self, lines: usize, buf: &mut String) -> Result<()> {
		for _ in 0..lines {
			self.stream.read_line(buf).await?;
		}
		Ok(())
	}

	async fn read_until(&mut self, byte: u8, inclusive: bool, buf: &mut Vec<u8>) -> Result<()> {
		self.stream.read_until(byte, buf).await?;
		if !inclusive {
			buf.pop();
		}
		Ok(())
	}

	async fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<()> {
		self.stream.read_to_end(buf).await?;
		Ok(())
	}

	async fn read_to_string(&mut self, str: &mut String) -> Result<()> {
		self.stream.read_to_string(str).await?;
		Ok(())
	}

	pub async fn interactive(self) -> Result<Option<ExitStatus>> {
		let mut tcp = self.stream.into_inner();
		let (mut proc_stdout, mut proc_stdin) = tcp.split();

		let stdin_end = async move {
			let mut buf = [0u8; BUFFER_CAPACITY];
			let mut stdin = stdin();
			loop {
				let num_bytes = match stdin.read(&mut buf).await {
					Ok(n) => n,
					Err(_) => break,
				};
				if let Err(_) = proc_stdin.write(&mut buf[..num_bytes]).await {
					break;
				}
			}
		};

		let stdout_end = async move {
			let mut buf = [0u8; BUFFER_CAPACITY];
			let mut stdout = stdout();
			loop {
				let num_bytes = match proc_stdout.read(&mut buf).await {
					Ok(n) => n,
					Err(_) => break,
				};
				if let Err(_) = stdout.write(&mut buf[..num_bytes]).await {
					break;
				}
			}
		};

		join!(stdin_end, stdout_end);

		Ok(None)
	}
}

pub enum Process {
	Local(LocalProcess),
	Remote(RemoteProcess),
}

impl Process {
	pub async fn new(location: &str, remote: bool) -> Result<Self> {
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
				let stream = TcpStream::connect(location).await?;

				let process = Process::Remote(RemoteProcess {
					stream: BufStream::new(stream),
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

	pub async fn write(&mut self, buf: &[u8]) -> Result<()> {
		match self {
			Process::Local(p) => p.write(buf).await?,
			Process::Remote(p) => p.write(buf).await?,
		}
		Ok(())
	}

	pub async fn write_str(&mut self, str: &str) -> Result<()> {
		self.write(str.as_bytes()).await?;
		Ok(())
	}

	pub async fn write_line(&mut self, str: &str) -> Result<()> {
		self.write(str.as_bytes()).await?;
		self.write(&[b'\n']).await?;
		Ok(())
	}

	pub async fn read_exact(&mut self, buf: &mut [u8]) -> Result<()> {
		match self {
			Process::Local(p) => p.read_exact(buf).await,
			Process::Remote(p) => p.read_exact(buf).await,
		}
	}

	pub async fn read_lines(&mut self, lines: usize, buf: &mut String) -> Result<()> {
		match self {
			Process::Local(p) => p.read_lines(lines, buf).await,
			Process::Remote(p) => p.read_lines(lines, buf).await,
		}
	}

	pub async fn read_until(&mut self, byte: u8, inclusive: bool, buf: &mut Vec<u8>) -> Result<()> {
		match self {
			Process::Local(p) => p.read_until(byte, inclusive, buf).await,
			Process::Remote(p) => p.read_until(byte, inclusive, buf).await,
		}
	}

	pub async fn read_to_end(&mut self, buf: &mut Vec<u8>) -> Result<()> {
		match self {
			Process::Local(p) => p.read_to_end(buf).await,
			Process::Remote(p) => p.read_to_end(buf).await,
		}
	}

	pub async fn read_to_string(&mut self, str: &mut String) -> Result<()> {
		match self {
			Process::Local(p) => p.read_to_string(str).await,
			Process::Remote(p) => p.read_to_string(str).await,
		}
	}

	pub async fn interactive(self) -> Result<Option<ExitStatus>> {
		match self {
			Process::Local(p) => p.interactive().await,
			Process::Remote(p) => p.interactive().await,
		}
	}
}
