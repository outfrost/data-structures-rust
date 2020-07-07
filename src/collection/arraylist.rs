use std::alloc::{self, Layout};
use std::fmt::{self, Debug};
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

	pub fn capacity(&self) -> usize {
		self.buf_extents * EXTENT_LEN
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

impl<T: PartialEq> PartialEq for ArrayList<T> {
	fn eq(&self, other: &Self) -> bool {
		let mut equal = self.len() == other.len();
		let mut i = 0;
		let len = self.len();
		while equal && i < len {
			if self[i] != other[i] {
				equal = false;
			}
			i += 1;
		}
		equal
	}
}

impl<T: Debug> Debug for ArrayList<T> {
	fn fmt(&self, fmt: &mut fmt::Formatter<'_>) -> fmt::Result {
		fmt.debug_struct(&("ArrayList<".to_owned() + std::any::type_name::<T>() + ">"))
			.field("len", &self.len)
			.field("buf_extents", &self.buf_extents)
			.finish()?;

		fmt.write_str(" ")?;

		let mut dbg = fmt.debug_list();
		for i in 0..self.len() {
			dbg.entry(&self[i]);
		}
		dbg.finish()
	}
}

#[cfg(test)]
mod tests {
	use super::*;

	#[test]
	fn i32_new() {	
		let a = ArrayList::<i32>::new();
		assert_eq!(a.len(), 0);
	}

	#[test]
	fn i32_with_capacity() {
		let a = ArrayList::<i32>::with_capacity(34);
		assert_eq!(a.len(), 0);
		assert_eq!(a.capacity(), 48);
	}

	#[test]
	fn i32_from_slice() {
		let a = ArrayList::from(&[] as &[i32]);
		assert_eq!(a, ArrayList::new());
	}

	#[test]
	fn i32_index() {
		let a = ArrayList::from(&[4, 2, 0, 69] as &[i32]);
		assert_eq!(a[3], 69);
	}

	#[test]
	fn i32_index_mut() {
		let mut a = ArrayList::from(&[4, 2, 0, 69] as &[i32]);
		a[2] = -1;
		assert_eq!(a, ArrayList::from(&[4, 2, -1, 69] as &[i32]));
	}

	#[test]
	fn i32_add() {
		let mut a = ArrayList::from(&[4, 2, 0, 69] as &[i32]);
		assert_eq!(a.buf_extents, 1);
		a.add(-5);
		assert_eq!(a, ArrayList::from(&[4, 2, 0, 69, -5] as &[i32]));
		assert_eq!(a.buf_extents, 1);
	}

	#[test]
	fn i32_add_realloc() {
		let mut a = ArrayList::from(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15] as &[i32]);
		assert_eq!(a.buf_extents, 1);
		a.add(-5);
		assert_eq!(a, ArrayList::from(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, -5] as &[i32]));
		assert_eq!(a.buf_extents, 2);
	}
}
