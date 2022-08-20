use anyhow::Result;
use clap::Parser;
use crawler::crawler::*;
use scraper::ElementRef;

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

    Crawler::builder()
        .add_default_propagators()
        .add_handler("*[href]", |el: ElementRef, _| {
            println!("{:?}", el.value().attr("href"))
        })
        .depth(args.depth)
        .build()?
        .crawl(&args.url)
        .await?;

    Ok(())
}
