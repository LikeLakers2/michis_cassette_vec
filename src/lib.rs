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
		self.pos = self.pos.max(self.inner.len());
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
	fn set_item(&mut self, index: usize, item: Self::Item);
	/// Removes the item at index `index` from the container, and returns the item.
	///
	/// Returns `None` if no item exists at index `index`.
	fn remove_item(&mut self, index: usize) -> Option<Self::Item>;
	/// Clears the container's contents.
	fn clear(&mut self);
}
