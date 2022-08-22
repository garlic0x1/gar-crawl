use anyhow::Result;
use clap::Parser;
use gar_crawl::crawler::*;
use reqwest::Url;
use scraper::ElementRef;
use std::collections::HashSet;

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
    let mut seen: HashSet<String> = HashSet::new();

    Crawler::builder()
        .add_default_propagators()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.5112.79 Safari/537.36".into())
        .add_handler("*[href]", move |el: ElementRef, url: Url| {
            seen.insert("aaa".into());
            if let Some(href) = el.value().attr("href") {
                if let Ok(abs_url) = url.join(href) {
                    out(abs_url.as_str(), &mut seen);
                } else {
                    out(href, &mut seen);
                }
            }
        })
        .depth(args.depth)
        .build()?
        .crawl(&args.url)
        .await?;

    Ok(())
}

fn out(message: &str, seen: &mut HashSet<String>) {
    if seen.insert(message.to_string()) {
        println!("{message}")
    }
}
