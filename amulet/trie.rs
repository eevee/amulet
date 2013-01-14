/** Trie implementation.
 *
 * Useful for figuring out when an entire terminal escape sequence has been
 * received, and what it is.
 */

// TODO way too much @ and copying in here but i get mut/ali errors otherwise.
// :(  revisit with 0.5 perhaps

extern mod std;

use core::cmp::Eq;
use core::hash::Hash;
use core::to_bytes::IterBytes;
use std::map::HashMap;

pub struct Trie<K: Eq IterBytes Hash Copy Const, V: Copy> {
    mut value: option::Option<V>,
    mut children: @HashMap<K, @Trie<K, V>>,

    // don't be copyable
    drop {}
}

/** Construct an empty trie. */
pub fn Trie<T: Eq IterBytes Hash Copy Const, U: Copy>() -> @Trie<T, U> {
    return @Trie{ value: option::None::<U>, children: @HashMap() };
}

impl<T: Eq IterBytes Hash Copy Const, U: Copy> Trie<T, U> {
    fn insert(@self, keys: &[T], value: U) {
        if keys.is_empty() {
            fail ~"Trie cannot have an empty key";
        }

        // TODO: error on duplicate key?

        let mut node = self;
        for uint::range(0, keys.len()) |k| {
            if node.children.contains_key(keys[k]) {
                node = node.children.get(keys[k]);
            }
            else {
                let new_node = @Trie{ value: option::None::<U>, children: @HashMap() };
                node.children.insert(keys[k], new_node);
                node = new_node;
            }
        }

        node.value = option::Some(value);
    }

    fn find(@self, keys: &[T]) -> option::Option<U> {
        if keys.is_empty() {
            fail ~"Trie cannot have an empty key";
        }

        let mut node = self;
        for uint::range(0, keys.len()) |k| {
            match node.children.find(keys[k]) {
                option::Some(child_node) => node = child_node,
                option::None => return None,
            }
        }

        return node.value;
    }

    fn find_prefix(@self, keys: &[T]) -> (option::Option<U>, ~[T]) {
        let mut node = self;
        for uint::range(0, keys.len()) |k| {
            match node.children.find(keys[k]) {
                option::Some(child_node) => node = child_node,
                option::None => return (node.value, vec::from_slice(keys.view(k, keys.len()))),
            }
        }

        return (node.value, ~[]);
    }

    fn _print_all() {
        self._print_all_impl(~[]);
    }
    fn _print_all_impl(key_prefix: &[T]) {
        match self.value {
            option::Some(copy value) => (io::stderr() as io::WriterUtil).write_line(fmt!("%? => %?", key_prefix, value)),
            option::None => (),
        }

        for self.children.each |key, node| {
            let new_prefix = vec::from_slice(key_prefix) + ~[key];
            node._print_all_impl(new_prefix);
        }
    }
}
