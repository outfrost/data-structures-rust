use std::alloc::{self, Layout};
use std::mem;
use std::ops::{Index, IndexMut};

const EXTENT_LEN: usize = 16;

pub struct ArrayList<T>{
	buf: *mut T,
	buf_extents: usize,
	len: usize,
}

impl<T> ArrayList<T> {
	pub fn new() -> ArrayList<T> {
		ArrayList {
			buf: unsafe { alloc::alloc(Self::layout()) as *mut T },
			buf_extents: 1,
			len: 0,
		}
	}

	pub fn with_capacity(cap: usize) -> ArrayList<T> {
		let mut buf_extents = cap / EXTENT_LEN;
		if cap % EXTENT_LEN > 0 {
			buf_extents += 1;
		}
		ArrayList {
			buf: unsafe { alloc::realloc(
				alloc::alloc(Self::layout()),
				Self::layout(),
				Self::layout().size() * buf_extents) as *mut T },
			buf_extents,
			len: 0,
		}
	}

	pub fn len(&self) -> usize {
		self.len
	}

	pub fn add(&mut self, item: T) {
		self.len += 1;
		if self.len > EXTENT_LEN * self.buf_extents {
			self.buf_extents += 1;
			self.buf = unsafe {
				alloc::realloc(
					self.buf as *mut u8,
					Self::layout(),
					Self::layout().size() * self.buf_extents) as *mut T };
		}
		let last_idx = self.len - 1;
		self[last_idx] = item;
	}

	fn layout() -> Layout {
		Layout::array::<T>(EXTENT_LEN).unwrap().pad_to_align()
	}
}

impl<T> Drop for ArrayList<T> {
	fn drop(&mut self) {
		unsafe { alloc::dealloc(self.buf as *mut u8, Self::layout()); }
	}
}

impl<T: Clone> From<&[T]> for ArrayList<T> {
	fn from(s: &[T]) -> ArrayList<T> {
		let mut arraylist = Self::with_capacity(s.len());
		for item in s.iter() {
			arraylist.add(item.clone());
		}
		arraylist
	}
}

impl<T> Index<usize> for ArrayList<T> {
	type Output = T;

	fn index(&self, index: usize) -> &Self::Output {
		unsafe {
			if index >= self.len {
				panic!("Index out of bounds");
			}
			&(*(self.buf.add(index)))
		}
	}
}

impl<T> IndexMut<usize> for ArrayList<T> {
	fn index_mut(&mut self, index: usize) -> &mut Self::Output {
		unsafe {
			if index >= self.len {				
				panic!("Index out of bounds");
			}
			&mut (*(self.buf.add(index)))
		}
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn i32_add() {
		let mut a = ArrayList::from(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15] as &[i32]);

		assert_eq!(a.len(), 16);
		for i in 0..16 {
			assert_eq!(a[i], i as i32);
		}

		a.add(-5);

		assert_eq!(a.len(), 17);
		for i in 0..16 {
			assert_eq!(a[i], i as i32);
		}
		assert_eq!(a[16], -5);
	}
}
