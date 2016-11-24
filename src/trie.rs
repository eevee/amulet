/** Trie implementation.
 *
 * Useful for figuring out when an entire terminal escape sequence has been
 * received, and what it is.
 */

// TODO way too much @ and copying in here but i get mut/ali errors otherwise.
// :(  revisit with 0.5 perhaps

use std::cmp::Eq;
use std::hash::Hash;
use std::collections::HashMap;
use std::collections::hash_map::Entry;
use std::fmt::Debug;
use std::vec;
use std::io::{self, Write};

pub struct Trie<K, V> {
    value: Option<V>,
    children: HashMap<K, Trie<K, V>>,
}

/** Construct an empty trie. */
pub fn Trie<T: Eq + Hash + Clone + Debug + 'static, U: Clone + Debug + 'static>() -> Trie<T, U> {
    return Trie::new();
}

impl<T: Eq + Hash + Clone + Debug + 'static, U: Clone + Debug + 'static> Trie<T, U> {
    pub fn new() -> Trie<T, U> {
        return Trie{ value: None::<U>, children: HashMap::new() };
    }

    pub fn insert(&mut self, keys: &[T], value: U) {
        if keys.is_empty() {
            self.value = Some(value);
            return;
        }

        let key = keys[0].clone();
        let child_node = match self.children.entry(key) {
            Entry::Occupied(child_node) => child_node.into_mut(),
            Entry::Vacant(child_node) => child_node.insert(Trie::new()),
        };
        child_node.insert(&keys[1..], value);
    }

    fn find(&self, keys: &[T]) -> Option<U> {
        if keys.is_empty() {
            panic!("Trie cannot have an empty key");
        }

        let mut node = self;
        for k in 0..keys.len() {
            match node.children.get(&keys[k]) {
                Some(child_node) => node = child_node,
                None => return None,
            }
        }

        return node.value.clone();
    }

    pub fn find_prefix(&self, keys: &[T]) -> (Option<U>, Vec<T>) {
        let mut node = self;
        for k in 0..keys.len() {
            match node.children.get(&keys[k]) {
                Some(child_node) => node = child_node,
                None => return (node.value.clone(), keys[k..].to_vec()),
            }
        }

        return (node.value.clone(), vec![]);
    }

    fn _print_all(&self) {
        self._print_all_impl(&mut vec![]);
    }
    fn _print_all_impl(&self, prefix: &mut Vec<T>) {
        for value in self.value.iter() {
            writeln!(io::stderr(), "{:?} => {:?}", prefix, value).unwrap();
        }

        for (key, node) in self.children.iter() {
            prefix.push(key.clone());
            node._print_all_impl(prefix);
            prefix.pop();
        }
    }
}
