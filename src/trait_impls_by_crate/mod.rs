macro_rules! forward_ref {
	() => {
		fn len(&self) -> usize {
			self.len()
		}

		fn get_item(&self, index: usize) -> Option<&Self::Item> {
			self.get(index)
		}
	};
}

macro_rules! forward_mut {
	(check_len_on_remove = $check_len:tt) => {
		forward_mut!(__inner, main);
		forward_mut!(__inner, remove, check_len = $check_len);
	};
	(__inner, main) => {
		fn get_item_mut(&mut self, index: usize) -> Option<&mut Self::Item> {
			self.get_mut(index)
		}

		fn set_item(&mut self, index: usize, element: Self::Item) {
			self.insert(index, element);
		}

		fn clear(&mut self) {
			self.clear();
		}
	};
	(__inner, remove, check_len = true) => {
		fn remove_item(&mut self, index: usize) -> Option<Self::Item> {
			(index < self.len()).then(|| self.remove(index))
		}
	};
	(__inner, remove, check_len = false) => {
		fn remove_item(&mut self, index: usize) -> Option<Self::Item> {
			self.remove(index)
		}
	};
}

/// Tests against `forward_ref!()` and `forward_mut!()`
#[cfg(test)]
mod forward_macro_tests {
	extern crate alloc;
	use core::ops::{Deref, DerefMut};

	use alloc::vec::Vec;

	use crate::{IndexableCollection, IndexableCollectionMut};

	/// Returns `None` on a bad remove
	#[derive(Default)]
	struct TestVec(Vec<i32>);

	impl TestVec {
		pub fn len(&self) -> usize {
			self.0.len()
		}

		pub fn get(&self, index: usize) -> Option<&i32> {
			self.0.get(index)
		}

		pub fn get_mut(&mut self, index: usize) -> Option<&mut i32> {
			self.0.get_mut(index)
		}

		pub fn insert(&mut self, index: usize, item: i32) {
			self.0.insert(index, item);
		}

		pub fn remove(&mut self, index: usize) -> Option<i32> {
			(index < self.len()).then(|| self.0.remove(index))
		}

		pub fn clear(&mut self) {
			self.0.clear();
		}
	}

	impl From<Vec<i32>> for TestVec {
		fn from(value: Vec<i32>) -> Self {
			Self(value)
		}
	}

	impl IndexableCollection for TestVec {
		type Item = i32;
		forward_ref!();
	}

	impl IndexableCollectionMut for TestVec {
		forward_mut!(check_len_on_remove = false);
	}

	/// Wrapper around `TestVec` to make it panic on a bad remove, so we can test the macro's "check
	/// len" functionality
	#[derive(Default)]
	struct PanicVec(TestVec);

	impl PanicVec {
		pub fn remove(&mut self, index: usize) -> i32 {
			self.0.remove(index).expect("removal index higher than len")
		}
	}

	#[allow(
		dead_code,
		reason = "These are here solely to prevent infinite recursion warnings on the macro, due \
		to trait methods taking priority over a deref. In a real scenario, either a deref wouldn't \
		happen, or the user would handle it themselves."
	)]
	impl PanicVec {
		pub fn len(&self) -> usize {
			self.deref().len()
		}

		pub fn clear(&mut self) {
			self.deref_mut().clear();
		}
	}

	impl Deref for PanicVec {
		type Target = TestVec;
		fn deref(&self) -> &Self::Target {
			&self.0
		}
	}

	impl DerefMut for PanicVec {
		fn deref_mut(&mut self) -> &mut Self::Target {
			&mut self.0
		}
	}

	impl From<Vec<i32>> for PanicVec {
		fn from(value: Vec<i32>) -> Self {
			Self(value.into())
		}
	}

	impl IndexableCollection for PanicVec {
		type Item = i32;
		forward_ref!();
	}

	impl IndexableCollectionMut for PanicVec {
		forward_mut!(check_len_on_remove = true);
	}

	/// Ensure that the length reported by the trait is the same as the length reported by the inner
	/// collection.
	#[test]
	fn len_consistency() {
		let inputs: [Vec<i32>; _] = [[].into(), [0].into(), [1, 2].into(), [3, 4, 5].into()];

		inputs.into_iter().for_each(|input| {
			let collection = TestVec(input);
			let collection_defined_len = collection.len();
			let trait_defined_len = IndexableCollection::len(&collection);

			assert_eq!(
				trait_defined_len, collection_defined_len,
				"the length returned by the trait was not the same as the length returned by the inner collection"
			);
		});
	}

	#[test]
	fn get_item_consistency() {
		let input = Vec::from([1, 2, 3]);

		let regular_vec = Vec::from_iter(input.clone());
		let test_vec = TestVec(regular_vec.clone());

		// We deliberately request one item past the end, to test if that is also the same
		for index in 0..=(input.len()) {
			let reg_res = regular_vec.get(index);
			let test_res = IndexableCollection::get_item(&test_vec, index);

			assert_eq!(
				reg_res, test_res,
				"the item returned by the trait was not the same as the item returned by the inner collection"
			);
		}
	}

	#[test]
	fn get_item_mut_consistency() {
		let input = Vec::from([1, 2, 3]);

		let mut regular_vec = Vec::from_iter(input.clone());
		let mut test_vec = TestVec(regular_vec.clone());

		// We deliberately request one item past the end, to test if that is also the same
		for index in 0..=(input.len()) {
			let reg_res = regular_vec.get_mut(index);
			let test_res = IndexableCollectionMut::get_item_mut(&mut test_vec, index);

			assert_eq!(
				reg_res, test_res,
				"the mutable item returned by the trait was not the same as the mutable item returned by the inner collection"
			);
		}
	}

	#[test]
	fn set_item_consistency() {
		let inputs: [(usize, i32); _] = [(0, 2), (2, 4), (5, 6)];

		let mut regular_vec = Vec::from_iter([0, 5, 10]);
		let mut test_vec = TestVec(regular_vec.clone());

		inputs.into_iter().for_each(|(index, element)| {
			regular_vec.insert(index, element);
			IndexableCollectionMut::set_item(&mut test_vec, index, element);
			assert_eq!(
				test_vec.0, regular_vec,
				"inserting an item didn't result in an identical collection"
			);
		});
	}
	mod remove_item {
		use super::*;

		macro_rules! make_test {
			($s:tt, $path_to_inner:tt) => {
				let inputs: [usize; _] = [5, 2, 0];

				let mut regular_vec = Vec::from_iter([2, 0, 4, 5, 10, 6]);
				let mut test_vec = $s::from(regular_vec.clone());

				// Test that `IndexableCollectionMut::remove_item()` removes and returns the same
				// items as removing them directly from the collection.
				inputs.into_iter().for_each(|index| {
					let reg_res = regular_vec.remove(index);
					let test_res = IndexableCollectionMut::remove_item(&mut test_vec, index);
					assert_eq!(Some(reg_res), test_res, "the returned item wasn't the same");
					assert_eq!(
						test_vec.$path_to_inner, regular_vec,
						"the collections weren't modified in the same way"
					);
				});

				// Test that attempting to remove past the end will return `None` - and importantly,
				// not panic!
				let index_past_the_end = test_vec.$path_to_inner.len();
				let test_res =
					IndexableCollectionMut::remove_item(&mut test_vec, index_past_the_end);
				assert_eq!(
					test_res, None,
					"removing an item past the end did not return `None`"
				);
			};
		}

		#[test]
		fn without_check_len() {
			make_test!(TestVec, 0);
		}

		#[test]
		fn with_check_len() {
			make_test!(PanicVec, 0.0);
		}
	}

	#[test]
	fn clear_consistency() {
		let mut regular_vec = Vec::from_iter([2, 0, 4, 5, 10, 6]);
		let mut test_vec = TestVec(regular_vec.clone());

		assert_eq!(
			test_vec.0, regular_vec,
			"the regular and test items weren't the same"
		);

		regular_vec.clear();
		IndexableCollectionMut::clear(&mut test_vec);
		assert_eq!(
			test_vec.0, regular_vec,
			"clearing the vecs did not result in the same list of items"
		);
	}
}

#[cfg(feature = "alloc")]
mod alloc;

#[cfg(feature = "arrayvec")]
mod arrayvec;

#[cfg(feature = "smallvec")]
mod smallvec;

#[cfg(feature = "tinyvec")]
mod tinyvec;
