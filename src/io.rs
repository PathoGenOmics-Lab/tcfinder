use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};
use std::collections::HashMap;

use petgraph::prelude::*;
use csv;

use crate::clusters::NodeW;


#[derive(serde::Deserialize, Debug)]
struct Phylo4Row {
    label: String,
    node: usize,
    ancestor: usize,
    nodetype: String,
}

/// Read a list of targets (one per line) from a plain text file
pub fn read_targets(reader: File) -> Vec<String> {
    let buf = BufReader::new(reader);
    buf.lines()
        .map(|line| line.expect("Could not parse line"))
        .collect()
}

/// Read a phylogeny in phylo4 format
pub fn read_phylo4(reader: File) -> Result<DiGraph<NodeW, ()>, Box<dyn Error>> {
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
