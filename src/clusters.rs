use petgraph::prelude::*;
use std::collections::VecDeque;

pub struct NodeW {
    pub index: usize,
    pub label: String,
    pub is_tip: bool,
    pub is_target: bool,
}

pub struct CladeTargetStats {
    /// Proportion of targets in clade
    pub prop: f64,
    /// Number of tips in clade
    pub size: usize,
}

/// Annotate targets in place
pub fn annotate_targets(mut tree: DiGraph<NodeW, ()>, targets: &Vec<String>) -> Graph<NodeW, ()> {
    for node in tree.node_indices() {
        let weight = tree.node_weight_mut(node).unwrap();
        weight.is_target = targets.contains(&weight.label);
    }
    tree
}

pub fn find_root(tree: &DiGraph<NodeW, ()>) -> Option<NodeIndex> {
    tree.node_indices().find(|&node| {
        tree.edges_directed(node, Direction::Incoming)
            .next()
            .is_none()
    })
}

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

fn calculate_clade_stats(tree: &DiGraph<NodeW, ()>, node: &NodeIndex) -> CladeTargetStats {
    let tips = get_descendant_leaves(tree, node);
    let n_tips = tips.len();
    let n_targets = tips
        .iter()
        .filter(|&tip| tree.node_weight(*tip).unwrap().is_target)
        .count();
    CladeTargetStats {
        prop: n_targets as f64 / n_tips as f64,
        size: n_tips,
    }
}

/// Find transmission clusters
pub fn tcfind(tree: &DiGraph<NodeW, ()>, threshold: CladeTargetStats) -> Vec<NodeIndex> {
    // Init results and queue
    let mut results: Vec<NodeIndex> = Vec::new();
    let mut queue: VecDeque<NodeIndex> = VecDeque::new();
    // Select root
    let root = find_root(&tree).unwrap();
    // Check first node
    let stats = calculate_clade_stats(tree, &root);
    if (stats.prop >= threshold.prop) && (stats.size >= threshold.size) {
        // The root is enough
        results.push(root);
    } else if stats.prop * (stats.size as f64) < (threshold.size as f64) {
        // There are no clusters
        return results;
    } else {
        // Enqueue to start subsequent search
        queue.push_back(root);
    }
    // Check the rest of nodes
    while let Some(node) = queue.pop_front() {
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
                results.push(child_node);
            } else if stats.prop * (stats.size as f64) < (threshold.size as f64) {
                // Not enough target nodes in subclade to qualify
            } else {
                // Some subclade would still be selected
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
    labels.sort();
    labels
}
