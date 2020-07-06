use std::alloc::{self, Layout};
use std::mem;
use std::ops::{Index, IndexMut};

const EXTENT_LEN: usize = 16;

pub struct ArrayList<T>{
	buf: *mut T,
	buf_layout: Layout,
	buf_extents: usize,
	len: usize,
}

impl<T> ArrayList<T> {
	pub fn new() -> ArrayList<T> {
		let buf_layout = Self::layout();
		ArrayList {
			buf: unsafe { alloc::alloc(buf_layout) as *mut T },
			buf_layout,
			buf_extents: 1,
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
					self.buf_layout,
					self.buf_layout.size() * self.buf_extents) as *mut T };
		}
		let last_idx = self.len - 1;
		self[last_idx] = item;
	}

	fn layout() -> Layout {
		Layout::array::<T>(EXTENT_LEN).unwrap().pad_to_align()
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
		let buf_layout = ArrayList::<i32>::layout();
		let mut a = ArrayList::<i32> {
			buf: unsafe { alloc::alloc(buf_layout) as *mut i32 },
			buf_layout,
			buf_extents: 1,
			len: EXTENT_LEN,
		};

		for i in 0..16 {
			a[i] = i as i32;
		}

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
