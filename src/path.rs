use std::{collections::HashMap, path::Path};
use walkdir::WalkDir;

#[derive(Debug)]
pub struct PathTree<'a> {
    children: HashMap<&'a Path, PathTree<'a>>,
    size: Option<u64>,
}

impl<'a> PathTree<'a> {
    pub fn new() -> Self {
        Self::with_capacity(0)
    }

    pub fn with_capacity(cap: usize) -> Self {
        Self {
            children: HashMap::with_capacity(cap),
            size: None,
        }
    }

    /// Inserts a path into the prefix tree and returns the size
    /// if the operation was successful.
    ///
    /// Considers two scenarios:
    /// 1. Ingores paths for which a parent path is already in the tree.
    /// 2. Removes all children if a parent path is inserted.
    pub fn insert(&mut self, path: &'a Path) -> Option<u64> {
        let calc_size = || get_path_size(path);
        let size = self.insert_with(path, calc_size);
        //log::debug!("insert {:?}: size {:?}", path, size);
        size
    }

    pub fn insert_with<F: Fn() -> u64>(&mut self, path: &'a Path, calc_size: F) -> Option<u64> {
        // path: /tmp/a
        let first = match path.iter().next() {
            Some(first) => first,
            None => {
                // if the sub path is empty, then this node is a leaf
                // and we calc the size
                self.size = Some(calc_size());
                self.children.clear();
                return self.size;
            }
        };

        // never add children to leafs
        if self.is_leaf() {
            return None;
        }

        let sub_path = path.strip_prefix(first).unwrap(); // tmp/a
        let child_size = self
            .children
            .entry(first.as_ref())
            .or_insert_with(|| PathTree::with_capacity(1))
            .insert_with(sub_path, calc_size);

        // add child node size to current
        if child_size.is_some() {
            self.size = Some(
                self.children
                    .values()
                    .filter_map(|val| val.size)
                    .sum::<u64>(),
            );
        }

        // propagate size to parent
        child_size
    }

    fn is_leaf(&self) -> bool {
        self.size.is_some() && self.children.is_empty()
    }

    fn traverse_tree<P: AsRef<Path>>(&self, path: P) -> Option<&Self> {
        let first = match path.as_ref().iter().next() {
            Some(first) => first,
            None => {
                // if the path is empty, then this node is a leaf
                return Some(self);
            }
        };

        match self.children.get(Path::new(first)) {
            Some(child_tree) => {
                child_tree.traverse_tree(path.as_ref().strip_prefix(first).unwrap().as_os_str())
            }
            _ => None,
        }
    }

    pub fn get<P: AsRef<Path>>(&self, path: P) -> Option<u64> {
        self.traverse_tree(path)?.size
    }

    pub fn contains_subpath<P: AsRef<Path>>(&self, subpath: P) -> bool {
        self.traverse_tree(subpath).is_some()
    }

    pub fn get_size(&self) -> Option<u64> {
        self.size
    }
}

//impl<'a> Iterator for PathTree<'a> {
//    type Item = u64;

//    fn next(&mut self) -> Option<Self::Item> {
//        if self.children.is_empty() {
//            return self.size;
//        }

//        self.children
//            .values()
//            .nth(self.iter_nth)?
//            .into_iter()
//            .next()
//    }
//}

fn get_path_size<P: AsRef<Path>>(path: P) -> u64 {
    WalkDir::new(path)
        .into_iter()
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.metadata().unwrap().len())
        .sum()
}

#[cfg(test)]
mod tests {
    use super::PathTree;
    use std::path::Path;

    #[test]
    fn insert_and_get() {
        let mut path_tree = PathTree::new();
        path_tree.insert(Path::new("/tmp/a/b"));

        assert_eq!(path_tree.get("/tmp/a/b"), Some(0));
        //assert_eq!(path_tree.get("/tmp/a"), None);
        //assert_eq!(path_tree.get("/tmp/a/b/c"), None);
        //assert_eq!(path_tree.get("/tmp/a/b.a"), None);
        //assert_eq!(path_tree.get("tmp/a/b"), None);
    }

    #[test]
    fn contains_subpath() {
        let mut path_tree = PathTree::new();
        path_tree.insert(Path::new("/tmp/a/b"));

        assert!(path_tree.contains_subpath("/"));
        assert!(path_tree.contains_subpath("/tmp"));
        assert!(path_tree.contains_subpath("/tmp/a"));
        assert!(path_tree.contains_subpath("/tmp/a/b"));
        assert!(!path_tree.contains_subpath("/tmp/a/c"));
        assert!(!path_tree.contains_subpath("tmp/a/b"));
    }

    #[test]
    fn insert_parent_path_removes_child() {
        let mut path_tree = PathTree::new();
        path_tree.insert(Path::new("/tmp/a/b"));
        path_tree.insert(Path::new("/tmp/a"));

        assert_eq!(path_tree.get("/tmp/a"), Some(0));
        assert_eq!(path_tree.get("/tmp/a/b"), None);
    }

    #[test]
    fn insert_child_path_is_ignored() {
        let mut path_tree = PathTree::new();
        path_tree.insert(Path::new("/tmp/a"));
        path_tree.insert(Path::new("/tmp/a/b"));

        assert_eq!(path_tree.get("/tmp/a"), Some(0));
        assert_eq!(path_tree.get("/tmp/a/b"), None);
    }

    #[test]
    fn insert_with_correct_size() {
        let mut path_tree = PathTree::new();
        path_tree.insert_with(Path::new("/tmp/a"), || 2);
        path_tree.insert_with(Path::new("/tmp/b"), || 4);
        path_tree.insert_with(Path::new("/home/potato"), || 8);
        path_tree.insert_with(Path::new("/tmp/d/e"), || 2);

        assert_eq!(path_tree.get("/"), Some(16));
    }
}

/*
PathTree {
    children: {
        "/": PathTree {
            children: {
                "tmp": PathTree { children: {"a": PathTree { children: {}, size: Some(2) }, "b": PathTree { children: {}, size: Some(4) }}, size: Some(6) },
                "home": PathTree { children: {"potato": PathTree { children: {}, size: Some(8) }}, size: Some(8) }},
        size: Some(16) }
    },
    size: Some(26)
}
 */
