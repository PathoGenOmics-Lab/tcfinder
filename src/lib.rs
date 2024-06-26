use std::{error::Error, fs::File};

use clap::Parser;
use log::*;
use simplelog::*;

mod clusters;
mod io;

/// tcfinder (transmission cluster finder)
/// finds clusters of samples from a list of identifiers within a phylo4 phylogeny
/// (see https://cran.r-project.org/web/packages/phylobase/vignettes/phylobase.html)
#[derive(Parser, Debug)]
#[command(author = "Miguel Álvarez Herrera <miguel.alvarez@uv.es>")]
#[command(version)]
struct Args {
    // Input tree in phylo4 format (mandatory CSV columns are 'label', 'node', 'ancestor' and 'nodetype')
    #[arg(short = 'i', long, required = true)]
    tree: String,

    // Input list of target labels plain text (one tip label per line)
    #[arg(short = 't', long, required = true)]
    targets: String,

    /// Output CSV file with clustering result
    #[arg(short = 'o', long, required = true)]
    output: String,

    /// Minimum number of tips (cluster size)
    #[arg(short = 's', long, default_value_t = 2)]
    minimum_size: usize,

    /// Minimum proportion of targets in cluster
    #[arg(short = 'p', long, default_value_t = 0.9)]
    minimum_prop: f64,

    /// Prints debug messages
    #[arg(short = 'v', long, default_value_t = false)]
    verbose: bool,
}

pub fn run() -> Result<(), Box<dyn Error>> {
    // Arguments
    let args = Args::parse();
    // Set log level
    if args.verbose {
        let _ = SimpleLogger::init(LevelFilter::Debug, Config::default());
    } else {
        let _ = SimpleLogger::init(LevelFilter::Info, Config::default());
    }
    // Init threshold
    let threshold = clusters::CladeTargetStats::threshold(args.minimum_prop, args.minimum_size);
    // Read targets
    info!("Reading input targets");
    let targets_file = File::open(args.targets)?;
    let targets: Vec<String> = io::read_targets(targets_file);
    // Read tree
    info!("Reading input tree");
    let tree_file = File::open(args.tree)?;
    let tree = io::read_phylo4(tree_file)?;
    let tree = clusters::annotate_targets(tree, &targets);
    // Find clusters
    info!("Calculating clusters");
    let clusters = clusters::tcfind(&tree, threshold);
    info!("Extracting tip labels");
    let labels = clusters::extract_clade_tip_labels(&tree, &clusters);
    // Write results
    info!("Writing results");
    io::write_cluster_table(&labels, args.output)
}

#[cfg(test)]
mod tests {

    use super::*;
    use petgraph::prelude::*;
    use std::io::{BufRead, BufReader};

    fn read_test_tree() -> DiGraph<clusters::NodeW, ()> {
        let file = File::open("test/rtree.csv").expect("Could not open tree file");
        io::read_phylo4(file).expect("Cannot parse tree")
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
        let _tree = read_test_tree();
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
        let tree = read_test_tree();
        let targets = read_test_targets();
        let tree = clusters::annotate_targets(tree, &targets);
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
        let root = clusters::find_root(&tree).unwrap();
        assert_eq!(root.index(), 100);
    }

    #[test]
    fn find_clusters() {
        let tree = read_test_tree();
        let targets = read_test_targets();
        let tree = clusters::annotate_targets(tree, &targets);
        let threshold = clusters::CladeTargetStats::threshold(0.9, 2);
        let clusters = clusters::tcfind(&tree, threshold);
        let labels = clusters::extract_clade_tip_labels(&tree, &clusters);
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
