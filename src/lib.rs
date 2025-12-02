#[derive(Clone, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub struct CassetteVec<T> {
	/// The items being stored.
	tape: Vec<T>,
	/// An index representing a position into the tape. The exact meaning of this number is
	/// dependant on the use-case, but generally this will point to the "beginning" of a cell -
	/// usually of the cell that we want to process next.
	head: usize,
}

impl<T> CassetteVec<T> {
	pub fn new() -> Self {
		Default::default()
	}

	/// Gets the underlying container as a slice.
	pub fn as_slice(&self) -> &[T] {
		self.tape.as_slice()
	}

	/// Gets the underlying container as a mutable slice.
	///
	/// Depending on how `CassetteVec` is being used, editing the slice's contents may result in odd
	/// behavior. For example, an undo-redo system may wish to prevent arbitrary editing of the
	/// slice's contents, to ensure that undoing and redoing actions gives consistent results.
	pub fn as_mut_slice(&mut self) -> &mut [T] {
		self.tape.as_mut_slice()
	}
}

// Head operations - designed similar to the Seek trait
impl<T> CassetteVec<T> {
	/// Returns the current position of the cassette head.
	///
	/// This can be assumed to uphold `0 <= head_position <= tape_len`, where `head_position` is the
	/// value returned by this function, and `tape_len` is the value returned by
	/// `self.as_tape().len()`.
	pub fn head_position(&self) -> usize {
		self.head
	}

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
impl<T> CassetteVec<T> {
	pub fn get_item_at_head(&self) -> Option<&T> {
		self.tape.get(self.head)
	}

	pub fn get_item_at_head_mut(&mut self) -> Option<&mut T> {
		self.tape.get_mut(self.head)
	}
}

impl<T> Default for CassetteVec<T> {
	fn default() -> Self {
		Self {
			tape: Default::default(),
			head: Default::default(),
		}
	}
}

impl<T> From<Vec<T>> for CassetteVec<T> {
	fn from(value: Vec<T>) -> Self {
		Self {
			tape: value,
			..Default::default()
		}
	}
}

#[derive(Clone, Copy, Debug, Hash, PartialEq, Eq, PartialOrd, Ord)]
pub enum SeekFrom {
	Start(usize),
	End(isize),
	Current(isize),
}
