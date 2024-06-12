use std::collections::{HashMap, VecDeque};
use std::error::Error;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};

use csv;
use petgraph::prelude::*;
use petgraph::visit::{IntoNeighborsDirected, Walker};

#[derive(serde::Deserialize, Debug)]
struct Phylo4Row {
    label: String,
    node: usize,
    ancestor: usize,
    nodetype: String,
}

struct NodeW {
    index: usize,
    label: String,
    is_tip: bool,
    is_target: bool,
}

/// Read a phylogeny in phylo4 format
fn read_phylo4(reader: File) -> Result<DiGraph<NodeW, ()>, Box<dyn Error>> {
    // Init tree
    let mut tree: DiGraph<NodeW, ()> = DiGraph::new();
    let mut tree_index: HashMap<usize, NodeIndex> = HashMap::new();
    // Init reader
    let mut rdr = csv::Reader::from_reader(reader);
    let headers = rdr.headers()?.clone();
    let results: Result<Vec<_>, csv::Error> = rdr.records().collect();
    let records = results?;
    // First pass: insert nodes
    for record in &records {
        // Parse row
        let row: Phylo4Row = record.deserialize(Some(&headers))?;
        // Insert node in tree
        let is_tip: bool = row.nodetype == "tip";
        let weight = NodeW {
            index: row.node,
            label: row.label,
            is_tip,
            is_target: false,
        };
        let node_index = tree.add_node(weight);
        // Insert node in index
        tree_index.insert(row.node, node_index);
    }
    // Second pass: insert edges
    for record in &records {
        // Parse row
        let row: Phylo4Row = record.deserialize(Some(&headers))?;
        if row.ancestor != 0 {
            // Get source (ancestor) and target (node) from index
            let &source = tree_index.get(&row.ancestor).unwrap();
            let &target = tree_index.get(&row.node).unwrap();
            // Insert edge in tree
            tree.add_edge(source, target, ());
        }
    }
    Ok(tree)
}

/// Annotate targets in place
fn annotate_targets(mut tree: DiGraph<NodeW, ()>, targets: &Vec<String>) -> Graph<NodeW, ()> {
    for node in tree.node_indices() {
        let weight = tree.node_weight_mut(node).unwrap();
        weight.is_target = targets.contains(&weight.label);
    }
    tree
}

struct CladeTargetStats {
    /// Proportion of targets in clade
    prop: f64,
    /// Number of tips in clade
    size: usize,
}

fn find_root(tree: &DiGraph<NodeW, ()>) -> Option<NodeIndex> {
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
fn tcfind(tree: &DiGraph<NodeW, ()>, threshold: CladeTargetStats) -> Vec<NodeIndex> {
    println!("TCFIND: Starting");
    // Init results and queue
    let mut results: Vec<NodeIndex> = Vec::new();
    let mut queue: VecDeque<NodeIndex> = VecDeque::new();
    // Select root
    let root = find_root(&tree).unwrap();
    // Check first node
    println!("TCFIND: Checking root {:?}", root);
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
        println!("TCFIND: processing {:?}", node);
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
fn extract_clade_tip_labels(tree: &DiGraph<NodeW, ()>, nodes: &Vec<NodeIndex>) -> Vec<Vec<String>> {
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

fn main() -> io::Result<()> {
    todo!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_test_tree() -> DiGraph<NodeW, ()> {
        let file = File::open("test/rtree.csv").expect("Could not open tree file");
        read_phylo4(file).expect("Cannot parse tree")
    }

    fn read_test_targets() -> Vec<String> {
        let file = File::open("test/targets.txt").expect("Could not open targets file");
        let buf = BufReader::new(file);
        buf.lines()
            .map(|line| line.expect("Could not parse line"))
            .collect()
    }

    #[test]
    fn read_tree() {
        let tree = read_test_tree();
    }

    #[test]
    fn count_tips() {
        let tree = read_test_tree();
        let n_tips = tree
            .node_indices()
            .map(|node| tree.node_weight(node))
            .filter(|w| w.unwrap().is_tip)
            .count();
        assert_eq!(n_tips, 100);
    }

    #[test]
    fn count_targets() {
        let targets = read_test_targets();
        assert_eq!(targets.len(), 13);
    }

    #[test]
    fn count_target_tips() {
        let mut tree = read_test_tree();
        let targets = read_test_targets();
        let tree = annotate_targets(tree, &targets);
        let n_targets = tree
            .node_indices()
            .map(|node| tree.node_weight(node))
            .filter(|w| w.unwrap().is_target)
            .count();
        assert_eq!(n_targets, 13);
    }

    #[test]
    fn find_test_root() {
        let tree = read_test_tree();
        let root = find_root(&tree).unwrap();
        assert_eq!(root.index(), 100);
    }

    #[test]
    fn find_clusters() {
        let mut tree = read_test_tree();
        let targets = read_test_targets();
        let tree = annotate_targets(tree, &targets);
        let threshold = CladeTargetStats { prop: 0.9, size: 2 };
        let clusters = tcfind(&tree, threshold);
        let labels = extract_clade_tip_labels(&tree, &clusters);
        assert_eq!(
            labels,
            vec![
                vec!["t100", "t35"],
                vec!["t21", "t47", "t51", "t70"],
                vec!["t48", "t75", "t98"],
                vec!["t78", "t81", "t82"]
            ]
        );
    }
}
