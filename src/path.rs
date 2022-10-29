use std::{collections::HashMap, fmt::Debug, path::Path};

#[derive(Debug)]
pub struct PathTree<'a, T> {
    children: HashMap<&'a Path, PathTree<'a, T>>,
    val: Option<T>,
}

impl<'a, T> PathTree<'a, T>
where
    T: Debug,
{
    pub fn new() -> Self {
        Self {
            children: HashMap::new(),
            val: None,
        }
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            children: HashMap::with_capacity(cap),
            val: None,
        }
    }

    pub fn insert(&mut self, path: &'a Path, val: T) {
        let mut path_iter = path.iter();
        let first = match path_iter.next() {
            Some(first) => first,
            None => {
                // if the path is empty, then this node is a leaf
                self.val = Some(val);
                self.children.clear();
                return;
            }
        };

        // don't add children if this node is included
        if self.val.is_some() {
            return;
        }

        self.children
            .entry(first.as_ref())
            .or_insert(PathTree::with_capacity(0))
            .insert(path.strip_prefix(first).unwrap(), val);
    }

    fn traverse_tree(&self, path: &'a Path) -> Option<&Self> {
        let mut tree = self;
        for subpath in path.iter() {
            let subpath = tree.children.get(&subpath.as_ref());
            println!("traverse_tree: {:?}", subpath);
            if let Some(child_tree) = subpath {
                tree = child_tree;
                continue;
            }

            return None;
        }

        Some(tree)
    }

    pub fn get(&self, path: &'a Path) -> Option<&T> {
        self.traverse_tree(path)?.val.as_ref()
    }

    pub fn contains_subpath(&self, subpath: &'a Path) -> bool {
        self.traverse_tree(subpath).is_some()
    }
}

// scenario 1:
// PathTree contains: /tmp/a/b
// add: /tmp/a
// action: remove /tmp/a/**
//
// scenario 2:
// PathTree contains: /tmp/a
// add: /tmp/a/b
// action: do nothing

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::PathTree;

    #[test]
    fn insert_and_get() {
        let mut path_tree = PathTree::new();
        path_tree.insert("/tmp/a/b".as_ref(), ());

        assert_eq!(path_tree.get("/tmp/a/b".as_ref()), Some(&()));
        assert_eq!(path_tree.get("/tmp/a".as_ref()), None);
        assert_eq!(path_tree.get("/tmp/a/b/c".as_ref()), None);
        assert_eq!(path_tree.get("/tmp/a/b.a".as_ref()), None);
        assert_eq!(path_tree.get("tmp/a/b".as_ref()), None);
    }

    #[test]
    fn contains_subpath() {
        let mut path_tree = PathTree::new();
        path_tree.insert("/tmp/a/b".as_ref(), ());

        assert!(path_tree.contains_subpath("/".as_ref()));
        assert!(path_tree.contains_subpath("/tmp".as_ref()));
        assert!(path_tree.contains_subpath("/tmp/a".as_ref()));
        assert!(path_tree.contains_subpath("/tmp/a/b".as_ref()));
        assert!(!path_tree.contains_subpath("/tmp/a/c".as_ref()));
        assert!(!path_tree.contains_subpath("tmp/a/b".as_ref()));
    }
}
