use anyhow::Result;

use goblin::{mach::Mach, Object};

use checksec as cs;

use memmap::Mmap;

use std::{
	collections::HashMap,
	fs::{read, File},
};

pub fn get_symbols(file_name: &str) -> Result<HashMap<String, u64>> {
	let object = read(&file_name)?;
	let mut mapping = HashMap::new();

	match Object::parse(&object)? {
		Object::Archive(_) => {
			unimplemented!();
		}
		Object::Elf(e) => {
			let dynstrtab = e.dynstrtab;
			let strtab = e.strtab;

			for sym in &e.dynsyms {
				if sym.st_name == 0 || sym.st_value == 0 {
					continue;
				}
				let x = mapping.insert(dynstrtab[sym.st_name].to_string(), sym.st_value);
				if let Some(x) = x {
					println!("Duplicate symbol: {}", x);
				}
			}

			for sym in &e.syms {
				if sym.st_name == 0 || sym.st_value == 0 {
					continue;
				}
				let x = mapping.insert(strtab[sym.st_name].to_string(), sym.st_value);
				if let Some(x) = x {
					println!("Duplicate symbol: {}", x);
				}
			}
		}
		Object::Mach(m) => match m {
			Mach::Fat(_) => {
				unimplemented!();
			}
			Mach::Binary(_) => {
				unimplemented!();
			}
		},
		Object::PE(_) => {
			unimplemented!();
		}
		Object::Unknown(_) => panic!("Unknown binary type"),
	}
	Ok(mapping)
}

pub fn checksec(file_name: &str) -> Result<()> {
	let object = read(&file_name)?;

	match Object::parse(&object)? {
		Object::Archive(_) => {
			unimplemented!();
		}
		Object::Elf(e) => {
			let sec = cs::elf::ElfCheckSecResults::parse(&e);
			println!("{:#?}", sec);
		}
		Object::Mach(m) => match m {
			Mach::Fat(_) => {
				unimplemented!();
			}
			Mach::Binary(macho) => {
				let sec = cs::macho::MachOCheckSecResults::parse(&macho);
				println!("{:#?}", sec);
			}
		},
		Object::PE(p) => {
			let file = File::open(&file_name)?;
			let buf = unsafe { Mmap::map(&file)? };

			let sec = cs::pe::PECheckSecResults::parse(&p, &buf);
			println!("{:#?}", sec);
		}
		Object::Unknown(_) => panic!("Unknown binary type"),
	}
	Ok(())
}
