use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{BufRead, BufReader};

use csv;
use petgraph::prelude::*;
use log::*;

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
    debug!("Initializing tree and tree index");
    let mut tree: DiGraph<NodeW, ()> = DiGraph::new();
    let mut tree_index: HashMap<usize, NodeIndex> = HashMap::new();
    // Init reader
    debug!("Reading tree rows");
    let mut rdr = csv::Reader::from_reader(reader);
    let headers = rdr.headers()?.clone();
    let results: Result<Vec<_>, csv::Error> = rdr.records().collect();
    let records = results?;
    // First pass: insert nodes
    debug!("Inserting nodes");
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
        debug!("Inserting node={}", row.node);
        let node_index = tree.add_node(weight);
        // Insert node in index
        tree_index.insert(row.node, node_index);
    }
    // Second pass: insert edges
    debug!("Inserting edges");
    for record in &records {
        // Parse row
        let row: Phylo4Row = record.deserialize(Some(&headers))?;
        if row.ancestor != 0 {
            // Get source (ancestor) and target (node) from index
            let &source = tree_index.get(&row.ancestor).unwrap();
            let &target = tree_index.get(&row.node).unwrap();
            // Insert edge in tree
            debug!("Inserting edge node={:?} -> node={:?}", row.ancestor, row.node);
            tree.add_edge(source, target, ());
        } else {
            debug!("Skipping root (ancestor=0)");
        }
    }
    Ok(tree)
}

#[derive(serde::Serialize)]
struct OutputRow {
    cluster_id: usize,
    label: String,
}

/// Writes a simple CSV (cluster_id, label)
pub fn write_cluster_table(
    clusters: &Vec<Vec<String>>,
    path: String,
) -> Result<(), Box<dyn Error>> {
    let mut wtr = csv::Writer::from_path(path)?;
    for (i, cluster_labels) in clusters.iter().enumerate() {
        debug!("Processing cluster_id={} with size {}", i+1, cluster_labels.len());
        for label in cluster_labels {
            wtr.serialize(OutputRow {
                cluster_id: i + 1,
                label: label.to_string(),
            })?;
        }
    }
    Ok(())
}
