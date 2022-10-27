use std::{
    cell::RefCell,
    collections::{HashMap, LinkedList},
    hash::Hash,
    rc::Rc,
};

pub struct PathTree<K, T>
where
    K: Hash + Eq + PartialEq,
{
    children: HashMap<K, PathTree<K, T>>,
    val: Option<T>,
}

impl<K, T> PathTree<K, T>
where
    K: Hash + Eq + PartialEq,
{
    fn new() -> Self {
        Self {
            children: HashMap::new(),
            val: None,
        }
    }

    fn with_capacity(cap: usize) -> Self {
        Self {
            children: HashMap::with_capacity(cap),
            val: None,
        }
    }

    fn insert(&mut self, mut keys: LinkedList<K>, val: T) {
        let first = match keys.pop_front() {
            Some(first) => first,
            None => return,
        };

        self.children
            .entry(first)
            .or_insert(PathTree::with_capacity(keys.len()))
            .insert(keys, val);
    }

    fn get(&self, keys: &LinkedList<K>) -> Option<&T> {
        let mut tree = self;
        for key in keys {
            if let Some(child_tree) = tree.children.get(key) {
                tree = child_tree;
                continue;
            }

            return None;
        }

        tree.val.as_ref()
    }
}
