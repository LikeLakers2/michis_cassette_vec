use generic_array::{ArrayLength, GenericArray};

use crate::{IndexableCollection, IndexableCollectionMut};

impl<T, N: ArrayLength> IndexableCollection for GenericArray<T, N> {
	type Item = T;

	forward_indexable!(get_item);

	fn len(&self) -> usize {
		N::USIZE
	}
}

impl<T, N: ArrayLength> IndexableCollectionMut for GenericArray<T, N> {
	forward_mutable!();
}
