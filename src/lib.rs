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

	pub fn insert_item_at_head(&mut self, item: Tape::Item) {
		self.inner.insert_item(self.pos, item);
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
	/// Inserts an item at a specific index, moving the item at the index and all items after it
	/// one index forward.
	fn insert_item(&mut self, index: usize, element: Self::Item);
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

	type TestVec = Vec<i32>;
	type TestCollection = CollectionCursor<TestVec>;

	fn test_vec() -> TestVec {
		let res = Vec::from([0, 1, 2, 3, 4, 5, 9, 8, 7, 6]);

		// Ensure that the length is a known value.
		// IF YOU CHANGE THIS, ENSURE TESTS ARE CHANGED TO MATCH.
		assert_eq!(res.len(), 10);

		res
	}

	fn test_collection() -> TestCollection {
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
			collection: &mut TestCollection,
			seek_from: SeekFrom,
			expected_result: Option<usize>,
			expected_pos: usize,
			error_message: &'static str,
		) {
			let new_pos = collection.seek(seek_from);
			assert_eq!(new_pos, expected_result, "{error_message}");
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
		inner(
			&mut collection,
			SeekFrom::Start(3),
			Some(3),
			3,
			"`Start(x)` should move the cursor within the bounds of the collection",
		);
		inner(
			&mut collection,
			SeekFrom::Start(0),
			Some(0),
			0,
			"`Start(0)` should move the cursor to the start of the collection",
		);

		inner(
			&mut collection,
			SeekFrom::Current(0),
			Some(0),
			0,
			"`Current(0) shouldn't move the cursor",
		);
		inner(
			&mut collection,
			SeekFrom::Current(7),
			Some(7),
			7,
			"`Current(x)` should move the cursor forward within the bounds of the collection",
		);
		inner(
			&mut collection,
			SeekFrom::Current(-2),
			Some(5),
			5,
			"`Current(-x) should move the cursor backwards within the bounds of the collection",
		);
		inner(
			&mut collection,
			SeekFrom::Current(-5),
			Some(0),
			0,
			"`Current(-current_pos) should move the cursor to the start of the collection",
		);

		inner(
			&mut collection,
			SeekFrom::End(0),
			Some(10),
			10,
			"`End(0)` should move the cursor to one past the end of the collection",
		);
		inner(
			&mut collection,
			SeekFrom::End(-1),
			Some(9),
			9,
			"`End(-1)` should move the cursor to the end of the collection",
		);
		inner(
			&mut collection,
			SeekFrom::End(-5),
			Some(5),
			5,
			"`End(-x)` should move the cursor within the bounds of the collection",
		);
		inner(
			&mut collection,
			SeekFrom::End(-10),
			Some(0),
			0,
			"`End(-len)` should move the cursor to the start of the collection",
		);

		// Seek to a known position. We reuse the testing function to ensure we're actually there,
		// just in case the test data has been messed with improperly.
		inner(
			&mut collection,
			SeekFrom::Start(7),
			Some(7),
			7,
			"this shouldn't fail",
		);

		// Seeking outside valid bounds should return `None` and *not* move the cursor
		inner(
			&mut collection,
			SeekFrom::Start(past_end_usize),
			None,
			7,
			"`Start(x)` shouldn't move past one index past the end of the collection",
		);

		inner(
			&mut collection,
			SeekFrom::Current(before_beginning),
			None,
			7,
			"`Current(-X)` shouldn't move past the start of the collection",
		);
		inner(
			&mut collection,
			SeekFrom::Current(past_end_isize),
			None,
			7,
			"`Current(x)` shouldn't move past the end of the collection",
		);

		inner(
			&mut collection,
			SeekFrom::End(1),
			None,
			7,
			"`End(1)` shouldn't move past the end of the collection",
		);
		inner(
			&mut collection,
			SeekFrom::End(before_beginning),
			None,
			7,
			"`End(-x)` shouldn't move past the start of the collection",
		);
		inner(
			&mut collection,
			SeekFrom::End(past_end_isize),
			None,
			7,
			"`End(x)` shouldn't move past the end of the collection",
		);
	}

	#[test]
	fn clamp_to_collection_bounds() {
		fn inner(
			collection: &mut TestCollection,
			initial_pos: usize,
			expected_pos: usize,
			error_message: &'static str,
		) {
			collection.pos = initial_pos;
			collection.clamp_to_collection_bounds();
			assert_eq!(collection.pos, expected_pos, "{error_message}");
		}

		let mut collection = self::test_collection();
		let collection_len = collection.inner.len();

		inner(
			&mut collection,
			usize::MAX,
			collection_len,
			"should move the cursor to one index past the end of the collection",
		);
		inner(
			&mut collection,
			2,
			2,
			"shouldn't move the cursor when already within the bounds of the collection",
		);
	}

	#[test]
	fn seek_to_start() {
		fn inner(collection: &mut TestCollection, initial_pos: usize) {
			collection.pos = initial_pos;
			collection.seek_to_start();
			assert_eq!(collection.pos, 0);
		}

		let mut collection = self::test_collection();

		// seek_to_end should ALWAYS succeed
		inner(&mut collection, 5);
		inner(&mut collection, usize::MAX);
	}

	#[test]
	fn seek_backward_one() {
		fn inner(
			collection: &mut TestCollection,
			should_succeed: bool,
			expected_new_pos: usize,
			error_message: &'static str,
		) {
			let seek_success = collection.seek_backward_one();
			assert_eq!(seek_success, should_succeed, "{error_message}");
			assert_eq!(
				collection.pos, expected_new_pos,
				"cursor was not moved to the expected position"
			);
		}

		let mut collection = self::test_collection();

		inner(
			&mut collection,
			false,
			0,
			"shouldn't seek past the beginning of the collection",
		);

		collection.pos = 5;
		inner(
			&mut collection,
			true,
			4,
			"should seek backward if within the bounds of the collection",
		);

		collection.pos = usize::MAX;
		inner(
			&mut collection,
			false,
			usize::MAX,
			"shouldn't seek if outside the bounds of the collection",
		);
	}

	#[test]
	fn seek_relative() {
		fn inner(
			collection: &mut TestCollection,
			offset: isize,
			expected_result: Option<usize>,
			expected_pos: usize,
			error_message: &'static str,
		) {
			let seek_res = collection.seek_relative(offset);
			assert_eq!(seek_res, expected_result, "{error_message}");
			assert_eq!(
				collection.pos, expected_pos,
				"the relative seek did not position the cursor as expected"
			);
		}

		let mut collection = self::test_collection();

		inner(&mut collection, 0, Some(0), 0, "shouldn't move at all");

		collection.pos = 5;
		inner(
			&mut collection,
			-2,
			Some(3),
			3,
			"should move when within the bounds of the collection",
		);
		inner(
			&mut collection,
			2,
			Some(5),
			5,
			"should move when within the bounds of the collection",
		);

		inner(&mut collection, 0, Some(5), 5, "shouldn't move at all");

		collection.pos = 5;
		inner(
			&mut collection,
			isize::MAX,
			None,
			5,
			"shouldn't move past the end of the collection",
		);
		inner(
			&mut collection,
			-isize::MAX,
			None,
			5,
			"shouldn't move before the beginning of the collection",
		);

		collection.pos = usize::MAX;
		inner(
			&mut collection,
			5,
			None,
			usize::MAX,
			"shouldn't move when outside the bounds of the collection",
		);
		inner(
			&mut collection,
			-5,
			None,
			usize::MAX,
			"shouldn't move when outside the bounds of the collection",
		);
	}

	#[test]
	fn seek_forward_one() {
		fn inner(
			collection: &mut TestCollection,
			should_succeed: bool,
			expected_new_pos: usize,
			error_message: &'static str,
		) {
			let seek_success = collection.seek_forward_one();
			assert_eq!(seek_success, should_succeed, "{error_message}");
			assert_eq!(
				collection.pos, expected_new_pos,
				"cursor was not moved to the expected position"
			);
		}

		let mut collection = self::test_collection();

		inner(
			&mut collection,
			true,
			1,
			"should seek forward when an item is available",
		);

		collection.pos = 5;
		inner(
			&mut collection,
			true,
			6,
			"should seek forward when an item is available",
		);

		let collection_len = collection.inner.len();

		collection.pos = collection_len - 1;
		inner(
			&mut collection,
			true,
			collection_len,
			"should seek one past the end of the bounds of the collection",
		);

		collection.pos = collection_len;
		inner(
			&mut collection,
			false,
			collection_len,
			"shouldn't seek past one index past the end of the bounds of the collection",
		);

		collection.pos = usize::MAX;
		inner(
			&mut collection,
			false,
			usize::MAX,
			"shouldn't seek at all past `len()` of the collection",
		);
	}

	#[test]
	fn seek_to_last_item() {
		fn inner(collection: &mut TestCollection, initial_pos: usize, expected_pos: usize) {
			collection.pos = initial_pos;
			collection.seek_to_last_item();
			assert_eq!(collection.pos, expected_pos);
		}

		let mut collection = self::test_collection();

		inner(&mut collection, 5, self::test_vec().len() - 1);
		inner(&mut collection, usize::MAX, self::test_vec().len() - 1);
	}

	#[test]
	fn seek_to_end() {
		fn inner(collection: &mut TestCollection, initial_pos: usize) {
			collection.pos = initial_pos;
			collection.seek_to_end();
			assert_eq!(collection.pos, collection.inner.len());
		}

		let mut collection = self::test_collection();

		// seek_to_end should ALWAYS succeed
		inner(&mut collection, 5);
		inner(&mut collection, usize::MAX);
	}

	#[test]
	fn get_item_at_head() {
		let test_vec = self::test_vec();
		let mut collection = self::test_collection();

		// We deliberately request one item past the end, to test if that's also the same
		for i in 0..=(test_vec.len()) {
			collection.pos = i;
			let test_vec_get = test_vec.get(i);
			let collection_get = collection.get_item_at_head();

			assert_eq!(
				test_vec_get, collection_get,
				"should get the same item from the same index (index = `{i}`)"
			);
		}
	}

	#[test]
	fn clear() {
		let mut test_vec = self::test_vec();
		let mut collection = self::test_collection();

		test_vec.clear();
		collection.clear();

		assert_eq!(collection.inner, test_vec);
	}

	#[test]
	fn get_item_at_head_mut() {
		let mut test_vec = self::test_vec();
		let mut collection = self::test_collection();

		// We deliberately request one item past the end, to test if that's also the same
		for i in 0..=(test_vec.len()) {
			collection.pos = i;
			let test_vec_get = test_vec.get_mut(i);
			let collection_get = collection.get_item_at_head_mut();

			assert_eq!(
				test_vec_get, collection_get,
				"should get the same item from the same index (index = `{i}`)"
			);
		}
	}

	#[test]
	fn set_item_at_head() {
		const AT_POS: usize = 5;
		const TO_VALUE: i32 = 52345;

		let mut test_vec = self::test_vec();
		let mut collection = self::test_collection();

		test_vec[AT_POS] = TO_VALUE;
		collection.pos = AT_POS;
		collection.set_item_at_head(TO_VALUE);

		assert_eq!(
			collection.inner.get(AT_POS),
			Some(&TO_VALUE),
			"should modify the inner collection to have the correct value at the head"
		);
		assert_eq!(collection.inner, test_vec, "should modify only one value");
	}

	#[test]
	fn insert_item_at_head() {
		const AT_POS: usize = 5;
		const TO_VALUE: i32 = 52345;

		let mut test_vec = self::test_vec();
		let mut collection = self::test_collection();

		test_vec.insert(AT_POS, TO_VALUE);
		collection.pos = AT_POS;
		collection.insert_item_at_head(TO_VALUE);

		assert_eq!(
			collection.inner.get(AT_POS),
			Some(&TO_VALUE),
			"should modify the inner collection to have the correct value at the head"
		);
		assert_eq!(collection.inner, test_vec, "should add only one value");
	}

	#[test]
	fn remove_item_at_head() {
		const AT_POS: usize = 5;

		let mut test_vec = self::test_vec();
		let mut collection = self::test_collection();

		let test_vec_res = test_vec.remove(5);
		collection.pos = AT_POS;
		let collection_res = collection.remove_item_at_head();
		assert_eq!(
			collection_res,
			Some(test_vec_res),
			"should return the right value"
		);
		assert_eq!(collection.inner, test_vec, "should remove only one value");

		// Additionally, test that this does NOT panic when out-of-bounds
		collection.pos = collection.inner.len() * 2;
		let collection_res = collection.remove_item_at_head();
		assert_eq!(
			collection_res, None,
			"should return `None` if the head was out-of-bounds"
		);
	}
}
