use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};

use csv;
use petgraph::prelude::*;

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
fn read_phylo4<T: std::io::Read>(reader: T) -> Result<Graph<NodeW, ()>, Box<dyn Error>> {
    // Init tree
    let mut tree: Graph<NodeW, ()> = Graph::new();
    let mut tree_index: HashMap<usize, NodeIndex> = HashMap::new();
    // Init reader
    let mut rdr = csv::Reader::from_reader(reader);
    let headers = rdr.headers()?.clone();
    // First pass: insert nodes
    for result in rdr.records() {
        // Parse row
        let record = result?;
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
    for result in rdr.records() {
        // Parse row
        let record = result?;
        let row: Phylo4Row = record.deserialize(Some(&headers))?;
        // Get source (ancestor) and target (node) from index
        let &source = tree_index.get(&row.ancestor).unwrap();
        let &target = tree_index.get(&row.node).unwrap();
        // Insert edge in tree
        tree.add_edge(source, target, ());
    }
    Ok(tree)
}

/// Annotates targets in place
fn annotate_targets(tree: &mut Graph<NodeW, ()>, targets: &Vec<String>) {
    for node in tree.node_indices() {
        let weight = tree.node_weight_mut(node).unwrap();
        weight.is_target = targets.contains(&weight.label);
    }
}

fn main() -> io::Result<()> {
    todo!();
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn read_test_tree() -> Graph<NodeW, ()> {
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
        assert_eq!(targets.len(), 6);
    }

    #[test]
    fn count_target_tips() {
        let mut tree = read_test_tree();
        let targets = read_test_targets();
        annotate_targets(&mut tree, &targets);
        let n_targets = tree
            .node_indices()
            .map(|node| tree.node_weight(node))
            .filter(|w| w.unwrap().is_target)
            .count();
        assert_eq!(n_targets, 6);
    }
}
