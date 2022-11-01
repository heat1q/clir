use std::{collections::HashMap, ffi::OsStr, fmt::Debug, path::Path};

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

    /// Inserts a path into the prefix tree.
    ///
    /// Considers two scenarios:
    /// 1. Ingores paths for which a parent path is already in the tree.
    /// 2. Removes all children if a parent path is inserted.
    pub fn insert(&mut self, raw_path: &'a OsStr, val: T) {
        let path = Path::new(raw_path);
        let first = match path.iter().next() {
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
            .or_insert(PathTree::with_capacity(1))
            .insert(path.strip_prefix(first).unwrap().as_os_str(), val);
    }

    fn traverse_tree<S: AsRef<OsStr>>(&self, os_path: S) -> Option<&Self> {
        let path = Path::new(&os_path);
        let first = match path.iter().next() {
            Some(first) => first,
            None => {
                // if the path is empty, then this node is a leaf
                return Some(self);
            }
        };

        match self.children.get(Path::new(first)) {
            Some(child_tree) => {
                child_tree.traverse_tree(path.strip_prefix(first).unwrap().as_os_str())
            }
            _ => None,
        }
    }

    pub fn get<S: AsRef<OsStr>>(&self, os_path: S) -> Option<&T> {
        self.traverse_tree(os_path)?.val.as_ref()
    }

    pub fn contains_subpath<S: AsRef<OsStr>>(&self, os_subpath: S) -> bool {
        self.traverse_tree(os_subpath).is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::PathTree;
    use std::ffi::OsStr;

    #[test]
    fn insert_and_get() {
        let mut path_tree = PathTree::new();
        path_tree.insert(OsStr::new("/tmp/a/b"), ());

        assert_eq!(path_tree.get("/tmp/a/b"), Some(&()));
        assert_eq!(path_tree.get("/tmp/a"), None);
        assert_eq!(path_tree.get("/tmp/a/b/c"), None);
        assert_eq!(path_tree.get("/tmp/a/b.a"), None);
        assert_eq!(path_tree.get("tmp/a/b"), None);
    }

    #[test]
    fn contains_subpath() {
        let mut path_tree = PathTree::new();
        path_tree.insert(OsStr::new("/tmp/a/b"), ());

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
        path_tree.insert(OsStr::new("/tmp/a/b"), ());
        path_tree.insert(OsStr::new("/tmp/a"), ());

        assert_eq!(path_tree.get("/tmp/a"), Some(&()));
        assert_eq!(path_tree.get("/tmp/a/b"), None);
    }

    #[test]
    fn insert_child_path_is_ignored() {
        let mut path_tree = PathTree::new();
        path_tree.insert(OsStr::new("/tmp/a"), ());
        path_tree.insert(OsStr::new("/tmp/a/b"), ());

        assert_eq!(path_tree.get("/tmp/a"), Some(&()));
        assert_eq!(path_tree.get("/tmp/a/b"), None);
    }
}
