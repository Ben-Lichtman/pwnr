struct Lyndon {
	n: usize,
	k: u8,
	buf: Option<Vec<u8>>,
}

impl Lyndon {
	fn new(n: usize, k: u8) -> Self { Self { n, k, buf: None } }
}

impl Iterator for Lyndon {
	type Item = Vec<u8>;

	fn next(&mut self) -> Option<Self::Item> {
		match &mut self.buf {
			None => {
				// Initialise
				let mut v = Vec::with_capacity(self.n);
				v.push(0);
				self.buf = Some(v);
				Some(vec![0])
			}
			Some(buf) => {
				// Repeat until chosen length
				let len = buf.len();
				for i in len..self.n {
					buf.push(buf[i % len]);
				}

				// Remove last element if it is the greatest value
				while buf.len() != 0 {
					if buf[buf.len() - 1] != self.k - 1 {
						break;
					}
					buf.pop();
				}

				// Increment last element
				let len = buf.len();
				if len != 0 {
					buf[len - 1] += 1;
				}

				// Give result
				match len {
					0 => None,
					_ => Some(buf.clone()),
				}
			}
		}
	}
}

pub fn debruijn(n: usize, k: u8) -> impl Iterator<Item = u8> {
	Lyndon::new(n, k)
		.filter(move |x| n % x.len() == 0)
		.flatten()
}

pub fn lookup(n: usize, k: u8, needle: &[u8]) -> usize {
	let table = debruijn(n, k).collect::<Vec<_>>().repeat(2);
	let position = table
		.windows(needle.len())
		.position(|window| window == needle);
	position.unwrap()
}
