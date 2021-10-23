// Copyright 2018-2021 Parity Technologies (UK) Ltd.
//
// Licensed under the Apache License, Version 2.0 (the "License");
// you may not use this file except in compliance with the License.
// You may obtain a copy of the License at
//
//     http://www.apache.org/licenses/LICENSE-2.0
//
// Unless required by applicable law or agreed to in writing, software
// distributed under the License is distributed on an "AS IS" BASIS,
// WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
// See the License for the specific language governing permissions and
// limitations under the License.

//! A simple mapping to contract storage.
//!
//! # Note
//!
//! This mapping doesn't actually "own" any data.
//! Instead it is just a simple wrapper around the contract storage facilities.

use crate::traits::{
    pull_packed_root_opt,
    push_packed_root,
    ExtKeyPtr,
    KeyPtr,
    PackedLayout,
    SpreadAllocate,
    SpreadLayout,
};
use core::{
    cell::RefCell,
    marker::PhantomData,
};
use ink_env::hash::Blake2x256;
use ink_primitives::Key;

/// A mapping of key-value pairs directly into contract storage.
#[derive(Default)]
pub struct Mapping<K, V> {
    offset_key: Key,
    index_key: RefCell<Key>,
    _marker: PhantomData<fn() -> (K, V)>,
}

#[cfg(feature = "std")]
const _: () = {
    use crate::traits::{
        LayoutCryptoHasher,
        StorageLayout,
    };
    use ink_env::hash::HashOutput;
    use ink_metadata::layout::{
        CellLayout,
        HashLayout,
        HashingStrategy,
        Layout,
        LayoutKey,
    };
    use scale_info::TypeInfo;

    impl<K, V> StorageLayout for Mapping<K, V>
    where
        K: Ord + scale::Encode,
        V: TypeInfo + 'static,
        Key: From<<Blake2x256 as HashOutput>::Type>,
    {
        fn layout(key_ptr: &mut KeyPtr) -> Layout {
            Layout::Hash(HashLayout::new(
                LayoutKey::from(key_ptr.advance_by(1)),
                HashingStrategy::new(
                    Blake2x256::crypto_hasher(),
                    b"mapping".to_vec(),
                    Vec::new(),
                ),
                Layout::Cell(CellLayout::new::<V>(LayoutKey::from(
                    key_ptr.advance_by(0),
                ))),
            ))
        }
    }

    impl<K, V> scale_info::TypeInfo for Mapping<K, V>
    where
        K: scale::Encode + TypeInfo + 'static + Ord,
        V: scale::Encode + TypeInfo + 'static,
        Key: From<<Blake2x256 as HashOutput>::Type>,
    {
        type Identity = Self;

        fn type_info() -> scale_info::Type {
            scale_info::Type::builder()
                .path(
                    scale_info::Path::from_segments(["ink_storage", "lazy", "Mapping"])
                        .expect("encountered invalid Rust path"),
                )
                .type_params([
                    scale_info::TypeParameter::new(
                        "K",
                        Some(scale_info::MetaType::new::<K>()),
                    ),
                    scale_info::TypeParameter::new(
                        "V",
                        Some(scale_info::MetaType::new::<V>()),
                    ),
                ])
                .composite(
                    scale_info::build::Fields::named()
                        .field(|f| f.name("offset_key").ty::<Key>().type_name("Key")),
                )
        }
    }
};

impl<K, V> core::fmt::Debug for Mapping<K, V> {
    fn fmt(&self, f: &mut core::fmt::Formatter) -> core::fmt::Result {
        f.debug_struct("Mapping")
            .field("offset_key", &self.offset_key)
            .finish()
    }
}

impl<K, V> Mapping<K, V> {
    /// Creates a new empty `Mapping`.
    fn new(offset_key: Key) -> Self {
        Self {
            offset_key,
            index_key: Default::default(),
            _marker: Default::default(),
        }
    }
}

impl<K, V> Mapping<K, V>
where
    K: PackedLayout,
    V: PackedLayout,
{
    /// Insert the given `value` to the contract storage.
    #[inline]
    pub fn insert<Q, R>(&mut self, key: Q, value: &R)
    where
        Q: scale::EncodeLike<K>,
        R: scale::EncodeLike<V> + PackedLayout,
    {
        // push_packed_root(value, &self.storage_key(key));
        self.index_into_storage_at(key, |at| {
            push_packed_root(value, at);
        })
    }

    /// Get the `value` at `key` from the contract storage.
    ///
    /// Returns `None` if no `value` exists at the given `key`.
    #[inline]
    pub fn get<Q>(&self, key: Q) -> Option<V>
    where
        Q: scale::EncodeLike<K>,
    {
        self.index_into_storage_at(key, pull_packed_root_opt)
    }

    #[inline]
    fn index_into_storage_at<Q, F, R>(&self, key: Q, f: F) -> R
    where
        Q: scale::EncodeLike<K>,
        F: FnOnce(&Key) -> R,
    {
        let encodedable_key = (&self.offset_key, key);
        let mut index_key = self.index_key.borrow_mut();
        ink_env::hash_encoded::<Blake2x256, _>(&encodedable_key, index_key.as_mut());
        f(&index_key)
    }
}

impl<K, V> SpreadLayout for Mapping<K, V> {
    const FOOTPRINT: u64 = 1;
    const REQUIRES_DEEP_CLEAN_UP: bool = false;

    #[inline]
    fn pull_spread(ptr: &mut KeyPtr) -> Self {
        // Note: There is no need to pull anything from the storage for the
        //       mapping type since it initializes itself entirely by the
        //       given key pointer.
        Self::new(*ExtKeyPtr::next_for::<Self>(ptr))
    }

    #[inline]
    fn push_spread(&self, ptr: &mut KeyPtr) {
        // Note: The mapping type does not store any state in its associated
        //       storage region, therefore only the pointer has to be incremented.
        ptr.advance_by(Self::FOOTPRINT);
    }

    #[inline]
    fn clear_spread(&self, ptr: &mut KeyPtr) {
        // Note: The mapping type is not aware of its elements, therefore
        //       it is not possible to clean up after itself.
        ptr.advance_by(Self::FOOTPRINT);
    }
}

impl<K, V> SpreadAllocate for Mapping<K, V> {
    #[inline]
    fn allocate_spread(ptr: &mut KeyPtr) -> Self {
        // Note: The mapping type initializes itself entirely by the key pointer.
        Self::new(*ExtKeyPtr::next_for::<Self>(ptr))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn insert_and_get_work() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mut mapping: Mapping<u8, _> = Mapping::new([0u8; 32].into());
            mapping.insert(&1, &2);
            assert_eq!(mapping.get(&1), Some(2));

            Ok(())
        })
        .unwrap()
    }

    #[test]
    fn gets_default_if_no_key_set() {
        ink_env::test::run_test::<ink_env::DefaultEnvironment, _>(|_| {
            let mapping: Mapping<u8, u8> = Mapping::new([0u8; 32].into());
            assert_eq!(mapping.get(&1), None);

            Ok(())
        })
        .unwrap()
    }
}
