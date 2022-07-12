use std::alloc::{self, Layout};
use std::fmt::{self, Debug};
use std::mem::MaybeUninit;
use std::ops::{Index, IndexMut};
use std::ptr;

const EXTENT_LEN: usize = 16;

pub struct ArrayList<T> {
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
			buf: unsafe {
				alloc::realloc(
					alloc::alloc(Self::layout()),
					Self::layout(),
					Self::layout().size() * buf_extents,
				) as *mut T
			},
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

	pub fn clear(&mut self) {
		for i in 0..self.len {
			unsafe {
				ptr::drop_in_place(self.buf.add(i));
			}
		}
		self.shrink(self.len);
	}

	pub fn insert(&mut self, index: usize, item: T) {
		if index > self.len {
			panic!("Index out of bounds");
		}
		self.grow(1);
		unsafe {
			ptr::copy(
				self.buf.add(index),
				self.buf.add(index + 1),
				self.len - index,
			);
		}
		self[index] = item;
	}

	pub fn push(&mut self, item: T) {
		self.grow(1);
		let last_idx = self.len - 1;
		self[last_idx] = item;
	}

	pub fn remove(&mut self, index: usize) -> T {
		if index >= self.len {
			panic!("Index out of bounds");
		}
		let item = unsafe {
			let mut space = MaybeUninit::<T>::uninit();
			ptr::copy_nonoverlapping(self.buf.add(index), space.as_mut_ptr(), 1);
			ptr::copy(
				self.buf.add(index + 1),
				self.buf.add(index),
				self.len - index - 1,
			);
			space.assume_init()
		};
		self.shrink(1);
		item
	}

	fn grow(&mut self, count: usize) {
		self.len += count;
		let extents = self.required_extents();
		if self.buf_extents < extents {
			self.realloc_extents(extents);
		}
	}

	fn shrink(&mut self, count: usize) {
		self.len -= count;
		let extents = self.required_extents();
		if self.buf_extents > extents {
			self.realloc_extents(extents);
		}
	}

	fn required_extents(&self) -> usize {
		let extents = self.len / EXTENT_LEN;
		if self.len % EXTENT_LEN > 0 {
			extents + 1
		} else {
			extents
		}
	}

	fn realloc_extents(&mut self, extents: usize) {
		self.buf_extents = extents;
		self.buf = unsafe {
			alloc::realloc(
				self.buf as *mut u8,
				Self::layout(),
				Self::layout().size() * self.buf_extents,
			) as *mut T
		};
	}

	fn layout() -> Layout {
		Layout::array::<T>(EXTENT_LEN).unwrap().pad_to_align()
	}
}

impl<T> Drop for ArrayList<T> {
	fn drop(&mut self) {
		unsafe {
			alloc::dealloc(self.buf as *mut u8, Self::layout());
		}
	}
}

impl<T: Clone> From<&[T]> for ArrayList<T> {
	fn from(s: &[T]) -> ArrayList<T> {
		let mut arraylist = Self::with_capacity(s.len());
		for item in s.iter() {
			arraylist.push(item.clone());
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
	fn i32_clear() {
		let mut a =
			ArrayList::from(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16] as &[i32]);
		assert_eq!(a.buf_extents, 2);
		a.clear();
		assert_eq!(a, ArrayList::new());
		assert_eq!(a.buf_extents, 0);
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
	fn i32_insert() {
		let mut a = ArrayList::from(&[4, 2, 0, 69] as &[i32]);
		assert_eq!(a.buf_extents, 1);
		a.insert(3, 1337);
		assert_eq!(a, ArrayList::from(&[4, 2, 0, 1337, 69] as &[i32]));
		assert_eq!(a.buf_extents, 1);
	}

	#[test]
	fn i32_insert_reallocate() {
		let mut a =
			ArrayList::from(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15] as &[i32]);
		assert_eq!(a.buf_extents, 1);
		a.insert(3, -99);
		assert_eq!(
			a,
			ArrayList::from(&[0, 1, 2, -99, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15] as &[i32])
		);
		assert_eq!(a.buf_extents, 2);
	}

	#[test]
	fn i32_push() {
		let mut a = ArrayList::from(&[4, 2, 0, 69] as &[i32]);
		assert_eq!(a.buf_extents, 1);
		a.push(-5);
		assert_eq!(a, ArrayList::from(&[4, 2, 0, 69, -5] as &[i32]));
		assert_eq!(a.buf_extents, 1);
	}

	#[test]
	fn i32_push_realloc() {
		let mut a =
			ArrayList::from(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15] as &[i32]);
		assert_eq!(a.buf_extents, 1);
		a.push(-5);
		assert_eq!(
			a,
			ArrayList::from(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, -5] as &[i32])
		);
		assert_eq!(a.buf_extents, 2);
	}

	#[test]
	fn i32_remove() {
		let mut a = ArrayList::from(&[4, 2, 0, 69] as &[i32]);
		assert_eq!(a.buf_extents, 1);
		assert_eq!(a.remove(1), 2);
		assert_eq!(a, ArrayList::from(&[4, 0, 69] as &[i32]));
		assert_eq!(a.buf_extents, 1);
	}

	#[test]
	fn i32_remove_realloc() {
		let mut a =
			ArrayList::from(&[0, 1, 2, 3, 4, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16] as &[i32]);
		assert_eq!(a.buf_extents, 2);
		assert_eq!(a.remove(4), 4);
		assert_eq!(
			a,
			ArrayList::from(&[0, 1, 2, 3, 5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 15, 16] as &[i32])
		);
		assert_eq!(a.buf_extents, 1);
	}
}
