#![no_std]

mod trait_impls_by_crate;

#[derive(Clone, Debug, Default, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub struct CollectionCursor<Tape> {
	/// The underlying collection that the cursor will point into.
	inner: Tape,
	/// An index representing a position into the collection. The exact meaning of this number is
	/// dependant on the use-case, but generally this will point to the "beginning" of a cell -
	/// usually of the cell that we want to process next.
	///
	/// The cursor is constrained to `0 <= pos <= self.inner.len()`, except in cases where a user
	/// calls `self.get_mut()`, changes the length to be less than the pos, and forgets to clamp
	/// the pos back within the collection's bounds. However, such a thing is a logic error, and is
	/// on the user of the struct to avoid.
	pos: usize,
}

impl<Tape> CollectionCursor<Tape> {
	/// Creates a new `CollectionCursor` wrapping the provided collection.
	///
	/// The cursor's initial position will always be `0`.
	pub fn new(inner: Tape) -> Self {
		Self {
			inner,
			pos: Default::default(),
		}
	}

	/// Returns the current position of the cursor.
	///
	/// This can be assumed to uphold `0 <= cursor_position <= self.get_ref().len()`, where
	/// `cursor_position` is the value returned by this function.
	pub fn position(&self) -> usize {
		self.pos
	}

	/// Gets a reference to the underlying collection.
	pub fn get_ref(&self) -> &Tape {
		&self.inner
	}

	/// Gets a mutable reference to the underlying collection.
	///
	/// # Warning
	/// If the underlying collection's length is modified, you should ensure that
	/// `0 <= self.position() <= self.get_ref().len()` is upheld before the next attempt to
	/// read/write at the cursor.
	///
	/// Failure to do so is a logic error. The behavior resulting from such a logic error is not
	/// specified, but will be encapsulated to the `CollectionCursor` that observed the logic error and
	/// not result in undefined behavior. This could include panics, incorrect results, and other
	/// such unwanted behavior.
	pub fn get_mut(&mut self) -> &mut Tape {
		&mut self.inner
	}

	pub fn into_inner(self) -> Tape {
		self.inner
	}
}

// Cursor operations
impl<Tape: IndexableCollection> CollectionCursor<Tape> {
	/// Moves the cursor to a new index.
	///
	/// It is an error to seek to a position before `0` or after `self.get_ref().len()`. In these
	/// cases, `None` will be returned.
	///
	/// Otherwise, this will return `Some(new_pos)`=, where `new_pos` is the new position of the
	/// cursor.
	// TODO: Change to something like `Result<usize, OutOfBoundsError>`
	pub fn seek(&mut self, pos: SeekFrom) -> Option<usize> {
		let collection_len = self.inner.len();

		let desired_position = match pos {
			SeekFrom::Start(p) => Some(p),
			SeekFrom::End(p) => collection_len.checked_add_signed(p),
			SeekFrom::Current(p) => self.pos.checked_add_signed(p),
		};

		desired_position
			.filter(|&pos| pos <= collection_len)
			.inspect(|&new_pos| self.pos = new_pos)
	}

	pub fn clamp_to_collection_bounds(&mut self) {
		// `usize`, by its nature, cannot be below `0`. Thus, we only need to know which is the
		// smaller value: the collection length, or the head position
		self.pos = self.pos.min(self.inner.len());
	}

	pub fn seek_to_start(&mut self) {
		self.pos = 0;
	}

	pub fn seek_backward_one(&mut self) -> bool {
		self.seek_relative(-1).is_some()
	}

	// TODO: Change to something like `Result<usize, OutOfBoundsError>`
	pub fn seek_relative(&mut self, offset: isize) -> Option<usize> {
		self.seek(SeekFrom::Current(offset))
	}

	pub fn seek_forward_one(&mut self) -> bool {
		self.seek_relative(1).is_some()
	}

	pub fn seek_to_last_item(&mut self) {
		self.pos = self.inner.len().checked_sub(1).unwrap_or_default();
	}

	pub fn seek_to_end(&mut self) {
		self.pos = self.inner.len();
	}
}

// Tape ref operations
impl<Tape: IndexableCollection> CollectionCursor<Tape> {
	pub fn get_item_at_head(&self) -> Option<&Tape::Item> {
		self.inner.get_item(self.pos)
	}
}

// Tape mut operations
impl<Tape: IndexableCollectionMut> CollectionCursor<Tape> {
	pub fn clear(&mut self) {
		self.inner.clear();
		self.pos = 0;
	}

	pub fn get_item_at_head_mut(&mut self) -> Option<&mut Tape::Item> {
		self.inner.get_item_mut(self.pos)
	}

	pub fn set_item_at_head(&mut self, item: Tape::Item) {
		self.inner.set_item(self.pos, item);
	}

	pub fn remove_item_at_head(&mut self) -> Option<Tape::Item> {
		self.inner.remove_item(self.pos)
	}
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
#[cfg_attr(feature = "serde", derive(serde::Serialize, serde::Deserialize))]
pub enum SeekFrom {
	/// Moves the cursor to the provided index.
	///
	/// # Examples
	/// * `SeekFrom::Start(0)` will move the cursor to the first item
	/// * `SeekFrom::Start(5)` will move the cursor to the sixth item
	Start(usize),
	/// Moves the cursor to the cassette's length (as provided by [`IndexableCollection::len`]) plus
	/// the provided number of indices.
	///
	/// It is an error to seek before the first index, or more than one index past the last item.
	///
	/// # Examples
	/// * `SeekFrom::End(-1)` will move the cursor to the last item, if one exists
	/// * `SeekFrom::End(0)` will move the cursor to the index just after the last item
	End(isize),
	/// Moves the cursor to the current position (as provided by [`CollectionCursor::position`])
	/// plus the provided number of indices.
	///
	/// It is an error to seek before the first index, or more than one index past the last item.
	///
	/// # Examples
	/// * `SeekFrom::Current(-2)` will move the cursor back two indices
	/// * `SeekFrom::Current(0)` won't move anywhere
	/// * `SeekFrom::Current(5)` will move the cursor forward five indices
	Current(isize),
}

#[allow(
	clippy::len_without_is_empty,
	reason = "While is_empty would normally be useful, we don't have a use for it here"
)]
pub trait IndexableCollection {
	/// The type of item this container contains.
	type Item;

	/// Gets the number of items this container currently contains.
	fn len(&self) -> usize;
	/// Gets a reference to the item at index `index`.
	///
	/// Returns `None` if no item exists at `index`.
	fn get_item(&self, index: usize) -> Option<&Self::Item>;
}

pub trait IndexableCollectionMut: IndexableCollection {
	/// Gets a mutable reference to the item at index `index`.
	///
	/// Returns `None` if no item exists at `index`.
	fn get_item_mut(&mut self, index: usize) -> Option<&mut Self::Item>;
	/// Sets an item at a specific index.
	fn set_item(&mut self, index: usize, element: Self::Item);
	/// Removes the item at index `index` from the container, and returns the item.
	///
	/// Returns `None` if no item exists at index `index`.
	fn remove_item(&mut self, index: usize) -> Option<Self::Item>;
	/// Clears the container's contents.
	fn clear(&mut self);
}

#[cfg(test)]
mod collection_cursor_tests {
	extern crate alloc;

	use super::*;
	use alloc::vec::Vec;

	fn test_vec() -> Vec<i32> {
		let res = Vec::from([0, 1, 2, 3, 4, 5, 9, 8, 7, 6]);

		// Ensure that the length is a known value.
		// IF YOU CHANGE THIS, ENSURE TESTS ARE CHANGED TO MATCH.
		assert_eq!(res.len(), 10);

		res
	}

	fn test_collection() -> CollectionCursor<Vec<i32>> {
		let res = CollectionCursor {
			inner: self::test_vec(),
			pos: Default::default(),
		};

		// Ensure that the cursor position is a known value.
		// IF YOU CHANGE THIS, ENSURE TESTS ARE CHANGED TO MATCH.
		assert_eq!(res.pos, Default::default());

		res
	}

	#[test]
	fn new() {
		let new_collection = CollectionCursor::new(self::test_vec());
		let test_collection = self::test_collection();

		assert_eq!(new_collection, test_collection);
	}

	#[test]
	fn position() {
		let mut collection = self::test_collection();
		assert_eq!(collection.position(), 0);

		collection.pos = 5;
		assert_eq!(collection.position(), 5);

		collection.pos = usize::MAX;
		assert_eq!(collection.position(), usize::MAX);
	}

	#[test]
	fn get_ref() {
		let collection = self::test_collection();
		assert_eq!(collection.get_ref(), &self::test_vec());
	}

	#[test]
	fn get_mut() {
		let mut collection = self::test_collection();
		assert_eq!(collection.get_mut(), &mut self::test_vec());
	}

	#[test]
	fn into_inner() {
		let collection = self::test_collection();
		assert_eq!(collection.into_inner(), self::test_vec());
	}

	#[test]
	fn seek() {
		fn inner(
			collection: &mut CollectionCursor<Vec<i32>>,
			seek_from: SeekFrom,
			expected_result: Option<usize>,
			expected_pos: usize,
		) {
			let new_pos = collection.seek(seek_from);
			assert_eq!(
				new_pos, expected_result,
				"the seek did not return the expected value"
			);
			assert_eq!(
				collection.pos, expected_pos,
				"the seek did not place the cursor at the expected position"
			);
		}
		let mut collection = self::test_collection();

		let past_end_usize: usize = test_collection().inner.len() * 2;
		let past_end_isize: isize = past_end_usize as isize;
		let before_beginning: isize = -past_end_isize;

		// Seeking to within valid bounds should return the `Some(the new position)` and move the
		// cursor
		inner(&mut collection, SeekFrom::Start(3), Some(3), 3);
		inner(&mut collection, SeekFrom::Start(0), Some(0), 0);

		inner(&mut collection, SeekFrom::Current(0), Some(0), 0);
		inner(&mut collection, SeekFrom::Current(7), Some(7), 7);
		inner(&mut collection, SeekFrom::Current(-2), Some(5), 5);
		inner(&mut collection, SeekFrom::Current(-5), Some(0), 0);

		inner(&mut collection, SeekFrom::End(0), Some(10), 10);
		inner(&mut collection, SeekFrom::End(-1), Some(9), 9);
		inner(&mut collection, SeekFrom::End(-5), Some(5), 5);
		inner(&mut collection, SeekFrom::End(-10), Some(0), 0);

		// Seek to a known position. We reuse the testing function to ensure we're actually there,
		// just in case the test data has been messed with improperly.
		inner(&mut collection, SeekFrom::Start(7), Some(7), 7);

		// Seeking outside valid bounds should return `None` and *not* move the cursor
		inner(&mut collection, SeekFrom::Start(past_end_usize), None, 7);

		inner(
			&mut collection,
			SeekFrom::Current(before_beginning),
			None,
			7,
		);
		inner(&mut collection, SeekFrom::Current(past_end_isize), None, 7);

		inner(&mut collection, SeekFrom::End(1), None, 7);
		inner(&mut collection, SeekFrom::End(before_beginning), None, 7);
		inner(&mut collection, SeekFrom::End(past_end_isize), None, 7);
	}

	#[test]
	fn clamp_to_collection_bounds() {
		// Create a messed up collection, and test clamping
		let mut collection = self::test_collection();
		collection.pos = usize::MAX;
		assert!(collection.pos > collection.inner.len());

		collection.clamp_to_collection_bounds();
		assert_eq!(collection.pos, collection.inner.len());

		// Create a normal collection, and test that clamping does nothing
		collection.pos = 2;
		collection.clamp_to_collection_bounds();
		assert_eq!(collection.pos, 2);
	}
}
