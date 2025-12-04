extern crate alloc;

use alloc::collections::VecDeque;

use crate::{TapeLike, TapeLikeMut};

impl<T> TapeLike for Vec<T> {
	type Item = T;

	fn len(&self) -> usize {
		self.len()
	}

	fn get_item(&self, index: usize) -> Option<&Self::Item> {
		self.get(index)
	}
}

impl<T> TapeLikeMut for Vec<T> {
	fn get_item_mut(&mut self, index: usize) -> Option<&mut Self::Item> {
		self.get_mut(index)
	}

	fn set_item(&mut self, index: usize, item: Self::Item) {
		self.insert(index, item);
	}

	fn remove_item(&mut self, index: usize) -> Option<Self::Item> {
		(index < self.len()).then(|| self.remove(index))
	}

	fn clear(&mut self) {
		self.clear();
	}
}

impl<T> TapeLike for VecDeque<T> {
	type Item = T;

	fn len(&self) -> usize {
		self.len()
	}

	fn get_item(&self, index: usize) -> Option<&Self::Item> {
		self.get(index)
	}
}

impl<T> TapeLikeMut for VecDeque<T> {
	fn get_item_mut(&mut self, index: usize) -> Option<&mut Self::Item> {
		self.get_mut(index)
	}

	fn set_item(&mut self, index: usize, item: Self::Item) {
		self.insert(index, item);
	}

	fn remove_item(&mut self, index: usize) -> Option<Self::Item> {
		self.remove(index)
	}

	fn clear(&mut self) {
		self.clear();
	}
}
