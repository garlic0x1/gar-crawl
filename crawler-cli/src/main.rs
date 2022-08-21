use anyhow::Result;
use clap::Parser;
use crawler::crawler::*;
use reqwest::Url;
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
        //.whitelist(&args.url)
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.5112.79 Safari/537.36".into())
        .add_handler("*[href]", |el: ElementRef, url: Url| {
            if let Some(href) = el.value().attr("href") {
                if let Ok(abs_url) = url.join(href) {
                    println!("{}", abs_url.as_str());
                } else {
                    println!("{}", url.as_str());
                }
            }
        })
        .depth(args.depth)
        .build()?
        .crawl(&args.url)
        .await?;

    Ok(())
}
