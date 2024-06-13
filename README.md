# tcfinder

[![PGO badge](https://img.shields.io/badge/PathoGenOmics-Lab-yellow.svg)](https://pathogenomics.github.io/)
[![Conda badge](https://img.shields.io/conda/d/tcfinder/nextclade)](https://anaconda.org/bioconda/tcfinder)
[![Release](https://img.shields.io/github/release/PathoGenOmics-Lab/tcfinder.svg)](https://github.com/PathoGenOmics-Lab/tcfinder/releases)
![Build](https://github.com/PathoGenOmics-Lab/tcfinder/actions/workflows/build.yml/badge.svg)
![Test](https://github.com/PathoGenOmics-Lab/tcfinder/actions/workflows/test.yml/badge.svg)

A lightweight tool to find clusters of samples from a list of identifiers within a phylogeny in phylo4 format
(see [`phylobase`](https://cran.r-project.org/web/packages/phylobase/vignettes/phylobase.html)),
regarding a minimum cluster size and a minimum proportion of targets in the cluster.

## Installation

This tool is available through the [`bioconda`](https://anaconda.org/bioconda/) conda channel.
Check out the [package recipe](http://bioconda.github.io/recipes/tcfinder/README.html) for more information.
Run the following command to install it in the current conda environment.

```shell
conda install -c bioconda tcfinder
```

Binary files can be retrieved through the releases page (e.g. see [latest release](https://github.com/PathoGenOmics-Lab/tcfinder/releases/latest)).
As a Rust project, it can also be built and tested using [`cargo`](https://doc.rust-lang.org/cargo/commands/cargo-build.html).

## Usage

### Quick reference

```txt
Usage: tcfinder [OPTIONS] --tree <TREE> --targets <TARGETS> --output <OUTPUT>

Options:
  -i, --tree <TREE>                  Input tree in phylo4 format (mandatory CSV columns are 'label', 'node', 'ancestor' and 'nodetype')
  -t, --targets <TARGETS>            Input list of target labels plain text (one tip label per line)
  -o, --output <OUTPUT>              Output CSV file with clustering result
  -s, --minimum-size <MINIMUM_SIZE>  Minimum cluster size [default: 2]
  -p, --minimum-prop <MINIMUM_PROP>  Minimum proportion of targets in cluster [default: 0.9]
  -v, --verbose                      Prints debug messages
  -h, --help                         Print help
  -V, --version                      Print version
```

For example, the following command will analyze the test data using the default thresholds.

```shell
tcfinder -i test/rtree.csv -t test/targets.txt -o test/clusters.csv
```

### Building a phylo4 tree

Trees can easily be converted to phylo4 format using the [`phylobase`](https://cran.r-project.org/web/packages/phylobase/index.html) R package.
As an example, the following code was used for generating the [test tree](/test/rtree.csv).
A similar approach may be used to convert an existing tree after reading it from disk
(for instance, Newick files can be read using the `read.tree` function from the `ape` package).

```R
library(ape)        # v5.6-2
library(phylobase)  # v0.8.10

# Generate random tree with 100 tips
tree <- rtree(100)
# Convert to phylo4
tree.p4 <- as(tree, "phylo4")
# Convert to dataframe and set undotted column names
tree.p4.df <- as(tree.p4, "data.frame")
names(tree.p4.df) <- c("label", "node", "ancestor", "edgelength", "nodetype")
```

Note that column names are key. The input tree
file must contain the following columns:

- `label`: a node label string (often found only for tip nodes).
- `node`: an integer indicating the node index, starting from 1.
- `ancestor`: an integer indicating the node index of the ancestor
  of the `node`. In a tree, a `node` can only have a single ancestor.
  A value of 0 indicates that the `node` has no ancestors (i.e. it is the tree root).
- `nodetype`: a string, either "tip", "internal" or "root".
  It is only used for checking if the node is a tip.

Branch lengths are not considered in the current implementation.
Users can take this variable into consideration by collapsing nodes beforehand.

### Clade stats

Threshold clade stats are provided via the command line.
Qualifying clades must meet both criteria.

- `--minimum-size`: an integer indicating the minimum cluster size, i.e.
  the minimum number of target tips (or leaves) in a qualifying clade.
- `--minimum-prop`: a floating point number in [0,1] indicating the
  minimum proportion of target tips in a qualifying clade.

The following figure shows the stats of two clades, the first of whic
qualifies under the default thresholds (✓), while the second does not (✗).

![cluster stats scheme](/scheme.png)

<!-- ## Citation -->

<!-- Manuscript under review. -->

<!-- TODO: https://allcontributors.org/ -->
