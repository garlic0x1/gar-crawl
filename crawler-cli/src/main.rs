use anyhow::Result;
use clap::Parser;
use crawler::crawler::*;

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Arguments {
    /// start url
    #[clap(value_parser)]
    url: String,

    /// crawl depth
    #[clap(short, long)]
    depth: u32,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Arguments::parse();

    Crawler::new()
        .add_default_propagators()
        .crawl(&args.url)
        .await?;

    Ok(())
}
