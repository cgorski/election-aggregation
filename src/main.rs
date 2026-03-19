//! Election Aggregation CLI
//!
//! Entry point for the election data processing pipeline.

use election_aggregation::VERSION;

fn main() {
    println!("election-aggregation v{VERSION}");
    println!();
    println!("A multi-layer pipeline for collecting, normalizing, and unifying");
    println!("US local election results from heterogeneous sources.");
    println!();
    println!("See the mdbook documentation for usage:");
    println!("  https://github.com/cgorski/election-aggregation/tree/main/docs");
    println!();
    println!("This tool does not ship with election data.");
    println!("Run `election-aggregation sources` to see where to download data.");
}
