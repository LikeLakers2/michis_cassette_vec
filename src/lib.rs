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
	pub fn get_tape_ref(&self) -> &Tape {
		&self.tape
	}

	/// Gets a mutable reference to the underlying tape.
	///
	/// If the underlying tape's length is modified, you **must** ensure that the cassette head
	/// position remains within the bounds of the tape. Failure to do is a logic error. The behavior
	/// resulting from such a logic error is not specified, but will be encapsulated to the
	/// `CassetteVec` that observed the logic error and not result in undefined behavior. This could
	/// include panics, incorrect results, and other such unwanted behavior.
	pub fn get_tape_mut(&mut self) -> &mut Tape {
		&mut self.tape
	}

	pub fn into_inner(self) -> Tape {
		self.tape
	}
}

// Head operations
impl<Tape> CassetteVec<Tape> {
	pub fn seek(&mut self, pos: usize) {
		self.head = pos;
	}

	pub fn clamp_to_tape_bounds<Item>(&mut self) -> usize
	where
		Tape: AsRef<[Item]>,
	{
		let new_head = self.head.min(self.tape.as_ref().len());
		self.head = new_head;
		new_head
	}

	pub fn seek_to_beginning(&mut self) {
		self.head = 0;
	}

	pub fn move_forward(&mut self) -> Option<usize> {
		self.head
			.checked_add(1)
			.inspect(|&new_head| self.head = new_head)
	}

	pub fn move_forward_checked<Item>(&mut self) -> Option<usize>
	where
		Tape: AsRef<[Item]>,
	{
		self.head
			.checked_add(1)
			.take_if(|&mut new_head| new_head <= self.tape.as_ref().len())
			.inspect(|&new_head| self.head = new_head)
	}

	pub fn seek_relative(&mut self, offset: isize) -> Option<usize> {
		self.head
			.checked_add_signed(offset)
			.inspect(|&new_head| self.head = new_head)
	}

	pub fn seek_relative_checked<Item>(&mut self, offset: isize) -> Option<usize>
	where
		Tape: AsRef<[Item]>,
	{
		self.head
			.checked_add_signed(offset)
			.take_if(|&mut new_head| new_head <= self.tape.as_ref().len())
			.inspect(|&new_head| self.head = new_head)
	}

	pub fn move_backward(&mut self) -> Option<usize> {
		self.head
			.checked_sub(1)
			.inspect(|&new_head| self.head = new_head)
	}

	pub fn seek_to_end<Item>(&mut self) -> usize
	where
		Tape: AsRef<[Item]>,
	{
		let new_head = self.tape.as_ref().len();
		self.head = new_head;
		new_head
	}
}

// Tape operations
impl<Tape> CassetteVec<Tape> {
	pub fn get_item_at_head<Item>(&self) -> Option<&Item>
	where
		Tape: AsRef<[Item]>,
	{
		self.tape.as_ref().get(self.head)
	}

	pub fn get_item_at_head_mut<Item>(&mut self) -> Option<&mut Item>
	where
		Tape: AsMut<[Item]>,
	{
		self.tape.as_mut().get_mut(self.head)
	}
	
	pub fn set_item_at_head<Item>(&mut self, new_item: Item)
	where
		Tape: AsMut<[Item]>
	{
		self.tape.as_mut()[self.head] = new_item;
	}
}
