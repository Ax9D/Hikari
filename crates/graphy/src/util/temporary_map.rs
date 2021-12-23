use arrayvec::ArrayVec;
use std::{borrow::BorrowMut, hash::Hash, collections::HashMap};

use crate::util::intrusive_linked_list::Node;

use super::intrusive_linked_list::IntrusiveLinkedList;

struct KeyRef<K> {
    key: *const K,
}

impl<K: Hash> Hash for KeyRef<K> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        unsafe { &*self.key }.hash(state);
    }
}
impl<K: PartialEq> PartialEq for KeyRef<K> {
    fn eq(&self, other: &Self) -> bool {
        unsafe { *self.key == *other.key }
    }
}
impl<K: Eq> Eq for KeyRef<K> {}

struct Entry<K, V> {
    key: K,
    value: V,
    frame: usize,
}

pub struct TemporaryMap<K, V, const N: usize> {
    hashmap: HashMap<KeyRef<K>, *mut Node<Entry<K, V>>, crate::util::BuildHasher>,
    frames: [IntrusiveLinkedList<Entry<K, V>>; N],
    current_frame: usize,
}

impl<K: Hash + Eq, V, const N: usize> TemporaryMap<K, V, N> {
    pub fn new() -> Self {
        let mut frames = ArrayVec::<IntrusiveLinkedList<Entry<K, V>>, N>::new();
        for _ in 0..N {
            frames.push(IntrusiveLinkedList::new());
        }

        let frames = frames
            .into_inner()
            .map_err(|_| "Array Vec to array conversion failed")
            .unwrap();

        Self {
            hashmap: HashMap::with_hasher(crate::util::hasher_builder()),
            frames,
            current_frame: 0,
        }
    }
    fn current_frame_data(&mut self) -> &mut IntrusiveLinkedList<Entry<K, V>> {
        self.frame_data(self.current_frame)
    }
    fn frame_data(&mut self, frame: usize) -> &mut IntrusiveLinkedList<Entry<K, V>> {
        &mut self.frames[frame]
    }
    fn touch(&mut self, node: *mut Node<Entry<K, V>>) {
        unsafe {
            let node_frame = (*node).data().frame;

            if self.current_frame != node_frame {
                let mut node = self.frame_data(node_frame).remove_node(node);

                node.data_mut().frame = self.current_frame;

                self.current_frame_data().append_node(node);
            }
        }
    }
    pub fn insert<'a>(&'a mut self, key: K, value: V) -> Option<V> {
        let current_frame = self.current_frame;

        let key_ref = KeyRef { key: &key };

        match self.hashmap.get(&key_ref) {
            Some(&node) => {
                self.touch(node);

                let node_data = unsafe { &mut *node }.data_mut();

                let old_value = std::mem::replace(&mut node_data.value, value);

                Some(old_value)
            }

            None => {
                let new_node = self
                    .current_frame_data()
                    .append_node(Node::new_boxed(Entry {
                        key,
                        value,
                        frame: current_frame,
                    }));

                self.hashmap.insert(
                    KeyRef {
                        key: &unsafe { &mut *new_node }.data_mut().key,
                    },
                    new_node,
                );

                None
            }
        }
    }
    pub fn get<'a>(&'a mut self, key: &K) -> Option<&'a V> {
        match self.hashmap.get(&KeyRef { key }) {
            Some(&node) => {
                self.touch(node);

                Some(&unsafe { &*node }.data().value)
            }
            None => None,
        }
    }
    pub fn get_mut<'a>(&'a mut self, key: &K) -> Option<&'a mut V> {
        match self.hashmap.get_mut(&KeyRef { key }) {
            Some(&mut node) => {
                self.touch(node);

                Some(&mut unsafe { &mut *node }.data_mut().value)
            }
            None => None,
        }
    }
    // pub fn get_or_insert_with<'a>(&'a mut self, key: K, insert_fn: impl FnOnce(&K) -> V) -> &'a mut V {
    //     match self.get_mut(&key) {
    //         Some(value) => value,
    //         None => {
    //             let value = (insert_fn)(&key);

    //             let new_node = self.current_frame_data().append_node(
    //                 Node::new_boxed(
    //                 Entry {
    //                         key,
    //                         value,
    //                         frame: self.current_frame
    //                     }
    //             ));

    //             self.hashmap.insert(KeyRef {key: &unsafe { &*new_node }.data().key}, new_node);

    //             &mut unsafe { &mut *new_node }.data_mut().value
    //         },
    //     }
    // }
    pub fn new_frame(&mut self, mut process_fn: impl FnMut(V)) {
        self.current_frame = (self.current_frame + 1) % N;

        for entry in self.frames[self.current_frame].drain() {
            let key_ref = KeyRef { key: &entry.key };
            self.hashmap.remove(&key_ref);
            (process_fn)(entry.value);
        }
    }
}
