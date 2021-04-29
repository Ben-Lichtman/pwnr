mod debruijn;

pub fn cyclic() -> impl Iterator<Item = char> {
	debruijn::debruijn(4, 26).map(|x| (x + b'A') as char)
}

pub fn lookup(needle: &str) -> usize {
	let needle = needle
		.as_bytes()
		.iter()
		.map(|x| x - b'A')
		.collect::<Vec<_>>();
	debruijn::lookup(4, 26, &needle)
}
