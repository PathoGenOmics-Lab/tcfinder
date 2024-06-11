use std::collections::HashMap;
use std::error::Error;
use std::fs::File;
use std::io::{self, prelude::*, BufReader};

use csv;
use petgraph::graph::NodeIndex;
use petgraph::Graph;


#[derive(serde::Deserialize, Debug)]
struct Phylo4Row {
    label: String,
    node: usize,
    ancestor: usize,
    nodetype: String
}


struct NodeW {
    index: usize,
    label: String,
    is_tip: bool,
    is_target: bool
}


struct EdgeW {
    distance: f64
}


fn read_phylo4<T: std::io::Read>(reader: T) -> Result<Graph<NodeW, EdgeW>, Box<dyn Error>> {
    // Init tree
    let mut tree: Graph<NodeW, EdgeW> = Graph::new();
    let mut tree_index: HashMap<usize, NodeIndex> = HashMap::new();
    // Init reader
    let mut rdr = csv::Reader::from_reader(reader);
    let headers = rdr.headers()?.clone();
    for result in rdr.records() {
        // Parse row
        let record = result?;
        let row: Phylo4Row = record.deserialize(Some(&headers))?;
        // Insert node in tree
        let is_tip: bool = row.nodetype == "tip";
        let weight = NodeW { index: row.node, label: row.label, is_tip, is_target: false };
        let node_index = tree.add_node(weight);
        // Insert node in index
        tree_index.insert(row.node, node_index);
    }
    // TODO: add edges
    todo!();
    Ok(tree)
}


fn annotate_targets(tree: &mut Graph<NodeW, EdgeW>, targets: &Vec<String>) -> Graph<NodeW, EdgeW> {
    todo!();
}


fn main() -> io::Result<()> {
    let file = File::open("tree.csv")?;
    
    let tree = read_phylo4(file);
    // println!("{:?}", tree);

    Ok(())
}
