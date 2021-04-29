use thiserror::Error;

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Debug, Error)]
pub enum Error {
	#[error("Could not connect to a remote socket")]
	CouldNotConnect,
	#[error("Input closed before pattern could be found")]
	PatternNotFound,
	#[error("Goblin")]
	Goblin(#[from] goblin::error::Error),
	#[error("IO")]
	IO(#[from] std::io::Error),
	#[error("Int parsing")]
	ParseInt(#[from] std::num::ParseIntError),
	#[error("UTF8")]
	UTF8(#[from] std::str::Utf8Error),
}
