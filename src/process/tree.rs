// Process tree construction

use super::ProcessInfo;
use std::collections::{HashMap, HashSet};

/// Builds parent-child process trees from flat process lists.
pub struct TreeBuilder;

impl TreeBuilder {
    /// Build process trees from a flat list of processes.
    ///
    /// Collects all PIDs into a set, sorts by PID, then partitions into
    /// roots (whose ppid is not in the set) and children. Children are
    /// recursively attached via a HashMap lookup.
    pub fn build(mut flat: Vec<ProcessInfo>) -> Vec<ProcessInfo> {
        if flat.is_empty() {
            return Vec::new();
        }

        // Collect all PIDs present in the list
        let pid_set: HashSet<u32> = flat.iter().map(|p| p.pid).collect();

        // Sort by PID for deterministic ordering
        flat.sort_by_key(|p| p.pid);

        // Partition into roots and children
        let (roots, children): (Vec<ProcessInfo>, Vec<ProcessInfo>) =
            flat.into_iter().partition(|p| !pid_set.contains(&p.ppid));

        // Group children by their ppid
        let mut children_map: HashMap<u32, Vec<ProcessInfo>> = HashMap::new();
        for child in children {
            children_map.entry(child.ppid).or_default().push(child);
        }

        // Recursively attach children to their parents
        roots
            .into_iter()
            .map(|root| Self::attach_children(root, &mut children_map))
            .collect()
    }

    /// Recursively attach children from the map to the given node.
    fn attach_children(
        mut node: ProcessInfo,
        children_map: &mut HashMap<u32, Vec<ProcessInfo>>,
    ) -> ProcessInfo {
        if let Some(children) = children_map.remove(&node.pid) {
            node.children = children
                .into_iter()
                .map(|child| Self::attach_children(child, children_map))
                .collect();
        }
        node
    }

    /// DFS traversal returning process references with their depth level.
    pub fn flatten_with_depth(trees: &[ProcessInfo]) -> Vec<(&ProcessInfo, usize)> {
        let mut result = Vec::new();
        for tree in trees {
            Self::flatten_recursive(tree, 0, &mut result);
        }
        result
    }

    fn flatten_recursive<'a>(
        node: &'a ProcessInfo,
        depth: usize,
        result: &mut Vec<(&'a ProcessInfo, usize)>,
    ) {
        result.push((node, depth));
        for child in &node.children {
            Self::flatten_recursive(child, depth + 1, result);
        }
    }

    /// Sort trees recursively at each level using the given comparator.
    pub fn sort_recursive<F>(trees: &mut [ProcessInfo], cmp: &F)
    where
        F: Fn(&ProcessInfo, &ProcessInfo) -> std::cmp::Ordering,
    {
        trees.sort_by(cmp);
        for tree in trees.iter_mut() {
            Self::sort_recursive(&mut tree.children, cmp);
        }
    }

    /// Find the node with the given PID anywhere in the trees (depth-first).
    pub fn find_by_pid(trees: &[ProcessInfo], pid: u32) -> Option<&ProcessInfo> {
        for tree in trees {
            if tree.pid == pid {
                return Some(tree);
            }
            if let Some(found) = Self::find_by_pid(&tree.children, pid) {
                return Some(found);
            }
        }
        None
    }

    /// Collect the node's PID and all descendant PIDs recursively.
    pub fn collect_pids(node: &ProcessInfo) -> Vec<u32> {
        let mut pids = vec![node.pid];
        for child in &node.children {
            pids.extend(Self::collect_pids(child));
        }
        pids
    }
}
