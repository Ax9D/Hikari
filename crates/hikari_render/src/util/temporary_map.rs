use arrayvec::ArrayVec;
use std::{collections::HashMap, hash::Hash};

use crate::util::intrusive_linked_list::Node;

use super::intrusive_linked_list::IntrusiveLinkedList;

// struct KeyRef<K> {
//     key: *const K,
// }

// impl<K: Hash> Hash for KeyRef<K> {
//     fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
//         unsafe { &*self.key }.hash(state);
//     }
// }
// impl<K: PartialEq> PartialEq for KeyRef<K> {
//     fn eq(&self, other: &Self) -> bool {
//         unsafe { *self.key == *other.key }
//     }
// }
// impl<K: Eq> Eq for KeyRef<K> {}

struct Entry<K, V> {
    key: K,
    value: V,
    frame: usize,
}

impl<K: std::fmt::Debug, V: std::fmt::Debug> std::fmt::Debug for Entry<K, V> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Entry")
            .field(&self.key)
            .field(&self.value)
            .finish()
    }
}

unsafe impl<K, V, const N: usize, H> Sync for TemporaryMap<K, V, N, H> {}
unsafe impl<K, V, const N: usize, H> Send for TemporaryMap<K, V, N, H> {}
type Map<K, V, H> = HashMap<K, *mut Node<Entry<K, V>>, H>;
pub struct TemporaryMap<K, V, const N: usize, H = crate::util::BuildHasher> {
    map: Map<K, V, H>,
    frames: [IntrusiveLinkedList<Entry<K, V>>; N],
    current_frame: usize,
}
impl<K: Hash + Eq + Copy, V, const N: usize> TemporaryMap<K, V, N, crate::util::BuildHasher> {
    pub fn new() -> Self {
        Self::with_hasher(crate::util::hasher_builder())
    }
}
impl<K: Hash + Eq + Copy, V, const N: usize, H: std::hash::BuildHasher> TemporaryMap<K, V, N, H> {
    pub fn with_hasher(hasher: H) -> Self {
        let mut frames = ArrayVec::<IntrusiveLinkedList<Entry<K, V>>, N>::new();
        for _ in 0..N {
            frames.push(IntrusiveLinkedList::new());
        }

        let frames = frames
            .into_inner()
            .map_err(|_| "Array Vec to array conversion failed")
            .unwrap();

        Self {
            map: Map::with_hasher(hasher),
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
    pub fn insert(&mut self, key: K, value: V) -> Option<V> {
        hikari_dev::profile_function!();

        let current_frame = self.current_frame;

        match self.map.get(&key) {
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

                self.map.insert(key, new_node);

                None
            }
        }
    }
    pub fn get<'a>(&'a mut self, key: &K) -> Option<&'a V> {
        hikari_dev::profile_function!();

        match self.map.get(key) {
            Some(&node) => {
                self.touch(node);

                Some(&unsafe { &*node }.data().value)
            }
            None => None,
        }
    }
    pub fn get_mut<'a>(&'a mut self, key: &K) -> Option<&'a mut V> {
        hikari_dev::profile_function!();

        match self.map.get_mut(key) {
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
    pub fn new_frame(&mut self) -> Removed<K, V, H> {
        self.current_frame = (self.current_frame + 1) % N;
        Removed {
            map: &mut self.map,
            drain: self.frames[self.current_frame].drain(),
        }
    }
}
use super::intrusive_linked_list::Drain;
pub struct Removed<'a, K, V, H> {
    map: &'a mut Map<K, V, H>,
    drain: Drain<'a, Entry<K, V>>,
}
impl<'a, K: Hash + Eq, V, H: std::hash::BuildHasher> Iterator for Removed<'a, K, V, H> {
    type Item = V;

    fn next(&mut self) -> Option<Self::Item> {
        self.drain.next().map(|entry| {
            self.map
                .remove(&entry.key)
                .expect("Map doesn't contain key");
            entry.value
        })
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::TemporaryMap;

    macro_rules! set {
        ( $( $x:expr ),* ) => {  // Match zero or more comma delimited items
            {
                let mut temp_set = HashSet::new();  // Create a mutable HashSet
                $(
                    temp_set.insert($x); // Insert each item matched into the HashSet
                )*
                temp_set // Return the populated HashSet
            }
        };
    }

    #[test]
    fn frame_update() {
        let mut map = TemporaryMap::<_, _, 4>::new();
        map.insert(1, "Foo");
        map.insert(2, "Bar");

        for _ in 0..120 {
            map.new_frame();
        }
    }

    #[test]
    fn auto_remove() {
        let mut map = TemporaryMap::<_, _, 4>::new();
        map.insert(1, "Foo");
        map.insert(2, "Bar");

        assert!(map.new_frame().next() == None);
        assert!(map.new_frame().next() == None);
        assert!(map.new_frame().next() == None);
        assert!(map.new_frame().collect::<HashSet<&str>>() == set!["Foo", "Bar"]);
        assert!(map.new_frame().next() == None);
    }

    #[test]
    fn keep() {
        let values = ["Foo", "Bar", "Lorem", "Ipsum"];
        let mut map = TemporaryMap::<_, _, 4>::new();

        for (ix, &value) in values.iter().enumerate() {
            map.insert(ix, value);
        }

        assert!(map.new_frame().next() == None);
        map.get(&0);
        assert!(map.new_frame().next() == None);
        assert!(map.new_frame().next() == None);
        assert!(map.new_frame().collect::<HashSet<&str>>() == set!["Bar", "Lorem", "Ipsum"]);
        assert!(map.new_frame().next() == Some("Foo"));
    }
}
