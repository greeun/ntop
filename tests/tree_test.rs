use nsm::process::tree::TreeBuilder;
use nsm::process::ProcessInfo;

fn make_process(pid: u32, ppid: u32, name: &str) -> ProcessInfo {
    let mut p = ProcessInfo::new(pid, name);
    p.ppid = ppid;
    p
}

#[test]
fn test_build_tree_simple() {
    let procs = vec![
        make_process(100, 1, "node-a"),
        make_process(101, 100, "node-b"),
        make_process(102, 101, "node-c"),
        make_process(200, 1, "node-d"),
    ];
    let trees = TreeBuilder::build(procs);
    assert_eq!(trees.len(), 2);
    // First root is pid 100 with chain 100→101→102
    assert_eq!(trees[0].pid, 100);
    assert_eq!(trees[0].children.len(), 1);
    assert_eq!(trees[0].children[0].pid, 101);
    assert_eq!(trees[0].children[0].children.len(), 1);
    assert_eq!(trees[0].children[0].children[0].pid, 102);
}

#[test]
fn test_build_tree_no_processes() {
    let procs: Vec<ProcessInfo> = vec![];
    let trees = TreeBuilder::build(procs);
    assert!(trees.is_empty());
}

#[test]
fn test_build_tree_all_roots() {
    let procs = vec![
        make_process(10, 1, "a"),
        make_process(20, 1, "b"),
        make_process(30, 1, "c"),
    ];
    let trees = TreeBuilder::build(procs);
    assert_eq!(trees.len(), 3);
    for tree in &trees {
        assert!(tree.children.is_empty());
    }
}

#[test]
fn test_flatten_tree() {
    let procs = vec![
        make_process(100, 1, "node-a"),
        make_process(101, 100, "node-b"),
        make_process(102, 101, "node-c"),
    ];
    let trees = TreeBuilder::build(procs);
    let flat = TreeBuilder::flatten_with_depth(&trees);
    assert_eq!(flat.len(), 3);
    assert_eq!(flat[0].0.pid, 100);
    assert_eq!(flat[0].1, 0);
    assert_eq!(flat[1].0.pid, 101);
    assert_eq!(flat[1].1, 1);
    assert_eq!(flat[2].0.pid, 102);
    assert_eq!(flat[2].1, 2);
}

#[test]
fn test_collect_subtree_pids() {
    let procs = vec![
        make_process(100, 1, "node-a"),
        make_process(101, 100, "node-b"),
        make_process(102, 101, "node-c"),
    ];
    let trees = TreeBuilder::build(procs);
    let pids = TreeBuilder::collect_pids(&trees[0]);
    assert_eq!(pids, vec![100, 101, 102]);
}
