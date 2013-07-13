/** Trie implementation.
 *
 * Useful for figuring out when an entire terminal escape sequence has been
 * received, and what it is.
 */

// TODO way too much @ and copying in here but i get mut/ali errors otherwise.
// :(  revisit with 0.5 perhaps

use std::cmp::Eq;
use std::hash::Hash;
use std::hashmap::HashMap;
use std::io::WriterUtil;
use std::to_bytes::IterBytes;
use std::vec;
use std::uint;
use std::io;

pub struct Trie<K, V> {
    value: Option<V>,
    children: @mut HashMap<K, @mut Trie<K, V>>,
}

// don't be copyable
#[unsafe_destructor]
impl<K, V> Drop for Trie<K, V> {
    fn drop(&self) {}
}

/** Construct an empty trie. */
pub fn Trie<T: Eq + IterBytes + Hash + Copy + Freeze, U: Copy>() -> @mut Trie<T, U> {
    return @mut Trie{ value: None::<U>, children: @mut HashMap::new() };
}

impl<T: Eq + IterBytes + Hash + Copy + Freeze, U: Copy> Trie<T, U> {
    pub fn insert(@mut self, keys: &[T], value: U) {
        if keys.is_empty() {
            fail!(~"Trie cannot have an empty key");
        }

        // TODO: error on duplicate key?

        let mut node = self;
        for uint::range(0, keys.len()) |k| {
            if node.children.contains_key(&keys[k]) {
                node = *node.children.get(&keys[k]);
            }
            else {
                let new_node = @mut Trie{ value: None::<U>, children: @mut HashMap::new() };
                node.children.insert(copy keys[k], new_node);
                node = new_node;
            }
        }

        node.value = Some(value);
    }

    fn find(@mut self, keys: &[T]) -> Option<U> {
        if keys.is_empty() {
            fail!(~"Trie cannot have an empty key");
        }

        let mut node = self;
        for uint::range(0, keys.len()) |k| {
            match node.children.find(&keys[k]) {
                Some(child_node) => node = *child_node,
                None => return None,
            }
        }

        return copy node.value;
    }

    pub fn find_prefix(@mut self, keys: &[T]) -> (Option<U>, ~[T]) {
        let mut node = self;
        for uint::range(0, keys.len()) |k| {
            match node.children.find(&keys[k]) {
                Some(child_node) => node = *child_node,
                None => return (copy node.value, vec::to_owned(keys.slice(k, keys.len()))),
            }
        }

        return (copy node.value, ~[]);
    }

    fn _print_all(&self) {
        self._print_all_impl(~[]);
    }
    fn _print_all_impl(&self, key_prefix: &[T]) {
        match copy self.value {
            Some(value) => io::stderr().write_line(fmt!("%? => %?", key_prefix, value)),
            None => (),
        }

        for self.children.iter().advance |(key, node)| {
            let new_prefix = vec::append_one(vec::to_owned(key_prefix), copy *key);
            node._print_all_impl(new_prefix);
        }
    }
}
