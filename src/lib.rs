extern crate alloc;

use alloc::collections::VecDeque;

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CassetteVec<Tape> {
	/// The underlying tape that the head will point into.
	tape: Tape,
	/// An index representing a position into the tape. The exact meaning of this number is
	/// dependant on the use-case, but generally this will point to the "beginning" of a cell -
	/// usually of the cell that we want to process next.
	head: usize,
}

impl<Tape> CassetteVec<Tape> {
	/// Creates a new `CassetteVec` wrapping the provided tape.
	///
	/// The tapehead's initial position will always be `0`.
	pub fn new(inner: Tape) -> Self {
		Self {
			tape: inner,
			head: Default::default(),
		}
	}

	/// Returns the current position of the cassette head.
	///
	/// This can be assumed to uphold `0 <= head_position <= tape_len`, where `head_position` is the
	/// value returned by this function, and `tape_len` is the value returned by
	/// `self.as_tape().len()`.
	pub fn head_position(&self) -> usize {
		self.head
	}

	/// Gets a reference to the underlying tape.
	pub fn get_ref(&self) -> &Tape {
		&self.tape
	}

	/// Gets a mutable reference to the underlying tape.
	///
	/// If the underlying tape's length is modified, you **must** ensure that the cassette head
	/// position remains within the bounds of the tape. Failure to do is a logic error. The behavior
	/// resulting from such a logic error is not specified, but will be encapsulated to the
	/// `CassetteVec` that observed the logic error and not result in undefined behavior. This could
	/// include panics, incorrect results, and other such unwanted behavior.
	pub fn get_mut(&mut self) -> &mut Tape {
		&mut self.tape
	}

	pub fn into_inner(self) -> Tape {
		self.tape
	}
}

impl<Tape: TapeLike> CassetteVec<Tape> {
	/// # Panics
	/// Panics if an attempt is made to seek before `usize::MIN`, or seek past the `len()` of the
	/// vec.
	// TODO: Make panics into a Result
	pub fn seek(&mut self, pos: SeekFrom) -> usize {
		let tape_len = self.tape.len();

		let new_head = match pos {
			SeekFrom::Start(p) => p,
			SeekFrom::End(p) => tape_len.checked_sub_signed(p).unwrap(),
			SeekFrom::Current(p) => self.head.checked_add_signed(p).unwrap(),
		};

		if new_head <= tape_len {
			self.head = new_head;
		} else {
			panic!();
		}

		self.head
	}

	pub fn rewind(&mut self) {
		self.head = 0;
	}

	/// # Panics
	/// Panics if an attempt is made to seek before `usize::MIN`, or seek past the `len()` of the
	/// vec.
	// TODO: Make panics into a Result
	pub fn seek_relative(&mut self, offset: isize) -> usize {
		self.seek(SeekFrom::Current(offset))
	}

	pub fn seek_to_end(&mut self) {
		self.head = self.tape.len();
	}
}

// Tape operations
impl<Tape: TapeLike> CassetteVec<Tape> {
	pub fn clear(&mut self) {
		self.tape.clear();
		self.head = 0;
	}

	pub fn tape_len(&self) -> usize {
		self.tape.len()
	}

	pub fn get_item_at_head(&self) -> Option<&Tape::Item> {
		self.tape.get_item(self.head)
	}

	pub fn get_item_at_head_mut(&mut self) -> Option<&mut Tape::Item> {
		self.tape.get_item_mut(self.head)
	}

	pub fn set_item_at_head(&mut self, item: Tape::Item) {
		self.tape.set_item_at(self.head, item);
	}

	// TODO: Should this be added back the same, or should it be replaced by a "truncate" function?
	/*
	pub fn remove_item_at_head(&mut self) -> Option<Tape::Item> {
		match self.head {
			i if i >= self.tape.len() => None,
			_ => Some(self.tape.remove(self.head)),
		}
	}
	*/
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeekFrom {
	Start(usize),
	End(isize),
	Current(isize),
}

#[allow(
	clippy::len_without_is_empty,
	reason = "While is_empty would normally be useful, we don't have a use for it here"
)]
pub trait TapeLike {
	type Item;

	fn len(&self) -> usize;
	fn get_item(&self, index: usize) -> Option<&Self::Item>;
	fn get_item_mut(&mut self, index: usize) -> Option<&mut Self::Item>;
	fn set_item_at(&mut self, index: usize, item: Self::Item);
	fn clear(&mut self);
}

impl<T> TapeLike for Vec<T> {
	type Item = T;

	fn len(&self) -> usize {
		self.len()
	}

	fn get_item(&self, index: usize) -> Option<&Self::Item> {
		self.get(index)
	}

	fn get_item_mut(&mut self, index: usize) -> Option<&mut Self::Item> {
		self.get_mut(index)
	}

	fn set_item_at(&mut self, index: usize, item: Self::Item) {
		self.insert(index, item);
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

	fn get_item_mut(&mut self, index: usize) -> Option<&mut Self::Item> {
		self.get_mut(index)
	}

	fn set_item_at(&mut self, index: usize, item: Self::Item) {
		self.insert(index, item);
	}

	fn clear(&mut self) {
		self.clear();
	}
}
