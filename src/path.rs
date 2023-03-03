use rayon::prelude::*;
use std::{
    collections::HashMap,
    fs::{self, Metadata},
    path::{Component, Path, PathBuf},
};

#[derive(Debug)]
pub struct PathTree {
    children: HashMap<PathBuf, PathTree>,
    size: Option<u64>,
}

impl PathTree {
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
    pub fn insert(&mut self, path: &Path) -> Option<u64> {
        let calc_size = || get_path_size_par(path, None);
        self.insert_with(path, calc_size)
    }

    pub fn insert_with<F: Fn() -> u64>(&mut self, path: &Path, calc_size: F) -> Option<u64> {
        // path: /tmp/a
        let Some(first) = path.iter().next() else {
            // if the sub path is empty, then this node is a leaf
            // and we calc the size
            let size = calc_size();
            let diff = size - self.size.unwrap_or(0);
            self.size = Some(size);
            self.children.clear();
            // return the diff in size when the node is updated
            return Some(diff);
        };

        // never add children to leafs
        if self.is_leaf() {
            return None;
        }

        let sub_path = path.strip_prefix(first).unwrap(); // tmp/a
        let child_size = self
            .children
            .entry(PathBuf::from(first))
            .or_insert_with(|| PathTree::with_capacity(1))
            .insert_with(sub_path, calc_size);

        // add child node size to current
        if let Some(child_sz) = child_size {
            self.size = Some(self.size.unwrap_or(0) + child_sz);
        }

        // propagate size to parent
        child_size
    }

    fn is_leaf(&self) -> bool {
        self.size.is_some() && self.children.is_empty()
    }

    fn traverse_tree<P: AsRef<Path>>(&self, path: P) -> Option<&Self> {
        let Some(first) = path.as_ref().iter().next() else {
            // if the path is empty, then this node is a leaf
            return Some(self);
        };

        self.children
            .get(Path::new(first))
            .and_then(|p| p.traverse_tree(path.as_ref().strip_prefix(first).ok()?.as_os_str()))
    }

    pub fn contains_parent<P: AsRef<Path>>(&self, path: P) -> bool {
        self.traverse_tree(path).is_some()
    }

    pub fn contains_subpath<P: AsRef<Path>>(&self, subpath: P) -> bool {
        let Some(subpath) = canonicalize(subpath) else {
            return false;
        };
        let mut tree = self;
        for p in subpath.iter() {
            let Some(t) = tree.children.get(Path::new(p)) else {
                return tree.is_leaf();
            };
            tree = t;
        }

        tree.is_leaf()
    }

    pub fn get_size(&self) -> Option<u64> {
        self.size
    }

    pub fn get_size_at<P: AsRef<Path>>(&self, path: P) -> Option<u64> {
        self.traverse_tree(path)?.size
    }
}

pub(super) fn get_path_size_par<P: AsRef<Path>>(path: P, meta: Option<Metadata>) -> u64 {
    let Some(meta) = meta.or_else(|| fs::metadata(&path).ok()) else {
        return 0;
    };

    if meta.is_file() || meta.is_symlink() {
        return meta.len();
    }

    if meta.is_dir() {
        if let Ok(dir_path) = fs::read_dir(path) {
            return dir_path
                .par_bridge()
                .filter_map(|entry| entry.ok())
                .map(|entry| get_path_size_par(entry.path(), entry.metadata().ok()))
                .sum();
        }
    }

    0
}

pub(super) fn canonicalize<P: AsRef<Path>>(path: P) -> Option<PathBuf> {
    let path = path.as_ref();
    let mut components: Vec<Component> = vec![];

    if !matches!(path.components().peekable().peek()?, Component::RootDir) {
        return None;
    }

    for c in path.components() {
        if matches!(c, Component::ParentDir) {
            components.pop()?;
            continue;
        }
        components.push(c)
    }

    Some(components.iter().map(|c| c.as_os_str()).collect())
}

#[cfg(test)]
mod tests {
    use super::PathTree;
    use crate::path::canonicalize;
    use std::path::{Path, PathBuf};

    #[test]
    fn canonicalize_glob() {
        assert_eq!(canonicalize("/tmp/..").unwrap(), PathBuf::from("/"));
        assert_eq!(
            canonicalize("/tmp//a/./../*.rs").unwrap(),
            PathBuf::from("/tmp/*.rs")
        );
    }

    #[test]
    fn insert_and_get() {
        let mut path_tree = PathTree::new();
        path_tree.insert(Path::new("/tmp/a/b"));

        assert_eq!(path_tree.get_size_at("/tmp/a/b"), Some(0));
    }

    #[test]
    fn contains_subpath() {
        let mut path_tree = PathTree::new();
        path_tree.insert_with(Path::new("/tmp/a"), || 1);

        assert!(!path_tree.contains_subpath("/tmp"));
        assert!(path_tree.contains_subpath("/tmp/a"));
        assert!(path_tree.contains_subpath("/tmp/a/"));
        assert!(path_tree.contains_subpath("/tmp/a/c/**/*.rs"));
    }

    #[test]
    fn contains_parent() {
        let mut path_tree = PathTree::new();
        path_tree.insert(Path::new("/tmp/a/b"));

        assert!(path_tree.contains_parent("/"));
        assert!(path_tree.contains_parent("/tmp"));
        assert!(path_tree.contains_parent("/tmp/a"));
        assert!(path_tree.contains_parent("/tmp/a/b"));
        assert!(!path_tree.contains_parent("/tmp/a/c"));
        assert!(!path_tree.contains_parent("tmp/a/b"));
    }

    #[test]
    fn insert_parent_path_removes_child() {
        let mut path_tree = PathTree::new();
        path_tree.insert(Path::new("/tmp/a/b"));
        path_tree.insert(Path::new("/tmp/a"));

        assert_eq!(path_tree.get_size_at("/tmp/a"), Some(0));
        assert_eq!(path_tree.get_size_at("/tmp/a/b"), None);
    }

    #[test]
    fn insert_child_path_is_ignored() {
        let mut path_tree = PathTree::new();
        path_tree.insert(Path::new("/tmp/a"));
        path_tree.insert(Path::new("/tmp/a/b"));

        assert_eq!(path_tree.get_size_at("/tmp/a"), Some(0));
        assert_eq!(path_tree.get_size_at("/tmp/a/b"), None);
    }

    #[test]
    fn insert_with_correct_size() {
        let mut path_tree = PathTree::new();
        path_tree.insert_with(Path::new("/tmp/a"), || 2);
        path_tree.insert_with(Path::new("/tmp/b"), || 4);
        path_tree.insert_with(Path::new("/home/potato"), || 8);
        path_tree.insert_with(Path::new("/tmp/d/e"), || 2);

        assert_eq!(path_tree.get_size_at("/"), Some(16));
    }

    #[test]
    fn insert_with_overwrite() {
        let mut path_tree = PathTree::new();
        path_tree.insert_with(Path::new("/tmp/a.tmp"), || 2);
        path_tree.insert_with(Path::new("/tmp/b.tmp"), || 4);
        path_tree.insert_with(Path::new("/tmp/c"), || 4);
        path_tree.insert_with(Path::new("/tmp/c/d.tmp"), || 4);
        path_tree.insert_with(Path::new("/tmp/f/f.tmp"), || 4);
        path_tree.insert_with(Path::new("/tmp"), || 16);

        assert_eq!(path_tree.get_size_at("/"), Some(16));
    }
}
