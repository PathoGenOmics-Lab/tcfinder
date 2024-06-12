use clap::Parser;
use std::{error::Error, fs::File};

mod tree;

/// tcfinder (transmission cluster finder)
/// finds transmission clusters in a phylo4 phylogeny
/// (see https://cran.r-project.org/web/packages/phylobase/vignettes/phylobase.html)
#[derive(Parser, Debug)]
#[command(author = "Miguel Álvarez Herrera <miguel.alvarez@uv.es>")]
#[command(version)]
struct Args {
    /// input tree in phylo4 format (mandatory CSV columns are 'label', 'node', 'ancestor' and 'nodetype')
    #[arg(short = 'i', long, required = true)]
    tree: String,

    /// input list of target labels plain text (one tip label per line)
    #[arg(short = 't', long, required = true)]
    targets: String,

    /// Output directory
    #[arg(short = 'o', long, required = true)]
    clusters: String,

    /// Minimum cluster size
    #[arg(short = 's', long, default_value_t = 2)]
    minimum_size: usize,

    /// Minimum proportion of targets in cluster
    #[arg(short = 'p', long, default_value_t = 0.9)]
    minimum_prop: f64,
}

pub fn run() -> Result<(), Box<dyn Error>> {
    // Arguments
    let args = Args::parse();
    // Init threshold
    let threshold = tree::CladeTargetStats {
        prop: args.minimum_prop,
        size: args.minimum_size,
    };
    // Read targets
    let targets_file = File::open(args.targets)?;
    let targets = tree::read_targets(targets_file);
    // Read tree
    let tree_file = File::open(args.tree)?;
    let tree = tree::read_phylo4(tree_file)?;
    let tree = tree::annotate_targets(tree, &targets);
    let clusters = tree::tcfind(&tree, threshold);
    let labels = tree::extract_clade_tip_labels(&tree, &clusters);
    Ok(())
}
