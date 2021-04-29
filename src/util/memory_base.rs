use std::num::Wrapping;

pub struct MemoryBase<T> {
	base: Wrapping<T>,
}

impl<T> MemoryBase<T>
where
	T: Copy,
	Wrapping<T>: std::ops::Sub<Output = Wrapping<T>> + std::ops::Add<Output = Wrapping<T>>,
{
	pub fn new(documented: T, leaked: T) -> Self {
		Self {
			base: Wrapping(leaked) - Wrapping(documented),
		}
	}

	pub fn documented_to_leaked(&self, documented: T) -> T { (Wrapping(documented) + self.base).0 }

	pub fn leaked_to_documented(&self, leaked: T) -> T { (Wrapping(leaked) - self.base).0 }
}
