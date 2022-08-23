use anyhow::Result;
use clap::Parser;
use gar_crawl::crawler::*;
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
        .whitelist(&args.url)
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.5112.79 Safari/537.36".into())
        .add_handler("*[href]", |args| {
            if let Some(href) = args.element.unwrap().value().attr("href") {
                if let Ok(abs_url) = args.page.url.join(href) {
                    out(abs_url.as_str(), &mut seen);
                } else {
                    out(href, &mut seen);
                }
            }
        })
        .depth(args.depth)
        .timeout(5, 0)
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
