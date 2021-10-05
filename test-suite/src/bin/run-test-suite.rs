use borscht::display::DisplayTree;
use structopt::StructOpt;

use borscht_visualizer::draw_to_file;

#[derive(Debug, StructOpt)]
#[structopt(name = "test-runner", about = "A test-running application.")]
struct AppOpts {
    #[structopt(subcommand)]
    dist: Distribution,
    #[structopt(long, default_value = "100")]
    count: usize,
    #[structopt(long)]
    depth: Option<usize>,
}

#[derive(Debug, StructOpt)]
#[structopt(name = "distribution")]
enum Distribution {
    Sample,
    MultivariateNormal,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let seed = 0u64;
    let opts = AppOpts::from_args();
    let tree = match opts.dist {
        Distribution::Sample => test_suite::sample::generate(seed),
        Distribution::MultivariateNormal => test_suite::mvn::generate(seed, opts.count),
    };
    match opts.depth {
        Some(depth) => tree.display_tree_to_depth(depth),
        None => tree.display_tree(),
    }
    draw_to_file("output.png", &tree)?;
    Ok(())
}
