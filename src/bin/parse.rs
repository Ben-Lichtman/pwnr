use anyhow::Result;

use std::env::args;

use ctftools::prelude::*;

fn main() -> Result<()> {
	let file_name = args().nth(1).unwrap();
	let m = get_symbols(&file_name)?;
	println!("{:#?}", m);

	Ok(())
}
