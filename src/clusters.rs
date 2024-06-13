use std::collections::VecDeque;

use log::*;
use petgraph::prelude::*;

/// Node features
pub struct NodeW {
    pub index: usize,
    pub label: String,
    pub is_tip: bool,
    pub is_target: bool,
}

/// Clade stats regarding target tips/leaves
#[derive(Debug)]
pub struct CladeTargetStats {
    /// Proportion of targets in clade
    prop: f64,
    /// Number of tips in clade
    size: usize,
    /// Number of targets in clade
    targets: usize,
}

impl CladeTargetStats {
    pub fn threshold(prop: f64, size: usize) -> Self {
        Self {
            prop,
            size,
            targets: f64::round(prop * size as f64) as usize,
        }
    }
    pub fn new(size: usize, targets: usize) -> Self {
        Self {
            prop: targets as f64 / size as f64,
            size,
            targets,
        }
    }
}

/// Annotate targets in place
pub fn annotate_targets(mut tree: DiGraph<NodeW, ()>, targets: &Vec<String>) -> Graph<NodeW, ()> {
    for node in tree.node_indices() {
        let weight = tree.node_weight_mut(node).unwrap();
        weight.is_target = targets.contains(&weight.label);
    }
    tree
}

/// Find the root of the tree (the one node with no incoming edges)
pub fn find_root(tree: &DiGraph<NodeW, ()>) -> Option<NodeIndex> {
    tree.node_indices().find(|&node| {
        tree.edges_directed(node, Direction::Incoming)
            .next()
            .is_none()
    })
}

/// Search the tips/leaves from the given node
fn get_descendant_leaves(graph: &DiGraph<NodeW, ()>, node: &NodeIndex) -> Vec<NodeIndex> {
    let mut leaves = Vec::new();
    let mut dfs = Dfs::new(graph, *node);
    while let Some(node) = dfs.next(&graph) {
        if graph.node_weight(node).unwrap().is_tip {
            leaves.push(node);
        }
    }
    leaves
}

/// Calculate the clade stats regarding target tips/leaves from the given node
fn calculate_clade_stats(tree: &DiGraph<NodeW, ()>, node: &NodeIndex) -> CladeTargetStats {
    let tips = get_descendant_leaves(tree, node);
    let n_tips = tips.len();
    let n_targets = tips
        .iter()
        .filter(|&tip| tree.node_weight(*tip).unwrap().is_target)
        .count();
    CladeTargetStats::new(n_tips, n_targets)
}

/// Find transmission clusters
pub fn tcfind(tree: &DiGraph<NodeW, ()>, threshold: CladeTargetStats) -> Vec<NodeIndex> {
    // Init results and queue
    debug!("Initializing");
    let mut results: Vec<NodeIndex> = Vec::new();
    let mut queue: VecDeque<NodeIndex> = VecDeque::new();
    // Select root
    debug!("Searching root");
    let root = find_root(&tree).unwrap();
    debug!(
        "Found root: node={:?}",
        tree.node_weight(root).unwrap().index
    );
    // Check first node
    debug!("Calculating root stats");
    let stats = calculate_clade_stats(tree, &root);
    if (stats.prop >= threshold.prop) && (stats.size >= threshold.size) {
        // The root is enough
        debug!("Root node qualifies");
        results.push(root);
    } else if stats.targets < threshold.targets {
        // There are no clusters
        debug!("Skipping search - no clusters in this tree");
        return results;
    } else {
        // Enqueue to start subsequent search
        debug!(
            "Enqueueing root to start search (prop={}, size={})",
            stats.prop, stats.size
        );
        queue.push_back(root);
    }
    // Check the rest of nodes
    while let Some(node) = queue.pop_front() {
        debug!(
            "Calculating stats for node={:?} children",
            tree.node_weight(node).unwrap().index
        );
        // Calculate child stats
        let children_stats: Vec<_> = tree
            // Get immediate descendants of node
            .edges_directed(node, Direction::Outgoing)
            .map(|edge| edge.target())
            // Select internal children
            .filter(|&node| !tree.node_weight(node).unwrap().is_tip)
            .map(|node| (node, calculate_clade_stats(tree, &node)))
            .collect();
        // Check qualification
        for (child_node, stats) in children_stats {
            if stats.prop >= threshold.prop && stats.size >= threshold.size {
                // Child qualifies
                debug!(
                    "Child node={:?} qualifies",
                    tree.node_weight(child_node).unwrap().index
                );
                results.push(child_node);
            } else if stats.targets < threshold.targets {
                // Not enough target nodes in subclade to qualify
                debug!(
                    "Skipping search from node={:?} - no clusters anywhere in its subclade",
                    tree.node_weight(child_node).unwrap().index
                );
            } else {
                // Some subclade would still be selected
                debug!(
                    "Enqueueing node={:?} (prop={}, size={})",
                    tree.node_weight(child_node).unwrap().index,
                    stats.prop,
                    stats.size
                );
                queue.push_back(child_node);
            }
        }
    }
    results
}

/// Extracts the tip labels of a vector of nodes representing clade roots, sorted
pub fn extract_clade_tip_labels(
    tree: &DiGraph<NodeW, ()>,
    nodes: &Vec<NodeIndex>,
) -> Vec<Vec<String>> {
    debug!("Extracting labels from subclades");
    let mut labels: Vec<Vec<String>> = nodes
        .iter()
        .map(|node| {
            let mut cluster_labels: Vec<String> = get_descendant_leaves(tree, node)
                .iter()
                .map(|tip| tree.node_weight(*tip).unwrap().label.clone())
                .collect();
            cluster_labels.sort();
            cluster_labels
        })
        .collect();
    debug!("Sorting cluster label lists");
    labels.sort();
    labels
}
