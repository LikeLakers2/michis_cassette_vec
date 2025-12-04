mod tapelike_impls;

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
	/// `self.tape_len()`.
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
	/// position is within the bounds of the tape before the next read/write. Failure to do is a
	/// logic error. The behavior resulting from such a logic error is not specified, but will be
	/// encapsulated to the `CassetteVec` that observed the logic error and not result in undefined
	/// behavior. This could include panics, incorrect results, and other such unwanted behavior.
	pub fn get_mut(&mut self) -> &mut Tape {
		&mut self.tape
	}

	pub fn into_inner(self) -> Tape {
		self.tape
	}
}

// Head operations
impl<Tape: TapeLike> CassetteVec<Tape> {
	// TODO: Change to something like `Result<usize, OutOfBoundsError>`
	pub fn seek(&mut self, pos: SeekFrom) -> Option<usize> {
		let tape_len = self.tape.len();

		let desired_position = match pos {
			SeekFrom::Start(p) => Some(p),
			SeekFrom::End(p) => tape_len.checked_add_signed(p),
			SeekFrom::Current(p) => self.head.checked_add_signed(p),
		};

		desired_position
			.filter(|&pos| pos <= tape_len)
			.inspect(|&new_pos| self.head = new_pos)
	}

	pub fn seek_to_start(&mut self) {
		self.head = 0;
	}

	pub fn move_backwards(&mut self) -> bool {
		self.seek_relative(-1).is_some()
	}

	// TODO: Change to something like `Result<usize, OutOfBoundsError>`
	pub fn seek_relative(&mut self, offset: isize) -> Option<usize> {
		self.seek(SeekFrom::Current(offset))
	}

	pub fn move_forward(&mut self) -> bool {
		self.seek_relative(1).is_some()
	}

	pub fn seek_to_last_item(&mut self) {
		self.head = self.tape.len().checked_sub(1).unwrap_or_default();
	}

	pub fn seek_to_end(&mut self) {
		self.head = self.tape.len();
	}
}

// Tape ref operations
impl<Tape: TapeLike> CassetteVec<Tape> {
	pub fn tape_len(&self) -> usize {
		self.tape.len()
	}

	pub fn get_item_at_head(&self) -> Option<&Tape::Item> {
		self.tape.get_item(self.head)
	}
}

// Tape mut operations
impl<Tape: TapeLikeMut> CassetteVec<Tape> {
	pub fn clear(&mut self) {
		self.tape.clear();
		self.head = 0;
	}

	pub fn get_item_at_head_mut(&mut self) -> Option<&mut Tape::Item> {
		self.tape.get_item_mut(self.head)
	}

	pub fn set_item_at_head(&mut self, item: Tape::Item) {
		self.tape.set_item(self.head, item);
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
	/// Moves the cassette head to the provided index.
	///
	/// # Examples
	/// * `SeekFrom::Start(0)` will move the cursor to the first item
	/// * `SeekFrom::Start(5)` will move the cursor to the sixth item
	Start(usize),
	/// Moves the cassette head to the cassette's length (as provided by [`CassetteVec::tape_len`])
	/// plus the provided number of indices.
	///
	/// It is an error to seek before the first index, or more than one index past the last item.
	///
	/// # Examples
	/// * `SeekFrom::End(-1)` will move the cursor to the last item, if one exists
	/// * `SeekFrom::End(0)` will move the cursor to the index just after the last item
	End(isize),
	/// Moves the cassette head to the current position (as provided by
	/// [`CassetteVec::head_position`]) plus the provided number of indices.
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
pub trait TapeLike {
	type Item;

	fn len(&self) -> usize;
	fn get_item(&self, index: usize) -> Option<&Self::Item>;
}

pub trait TapeLikeMut: TapeLike {
	fn get_item_mut(&mut self, index: usize) -> Option<&mut Self::Item>;
	fn set_item(&mut self, index: usize, item: Self::Item);
	fn clear(&mut self);
}
