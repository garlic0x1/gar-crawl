use anyhow::Result;
use clap::Parser;
use futures::future::join_all;
use gar_crawl::*;
use std::{collections::HashSet, sync::Arc};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Arguments {
    /// start url
    #[clap(short, long)]
    url: Option<String>,

    /// crawl depth
    #[clap(default_value_t = 2, short, long)]
    depth: usize,

    /// concurrency limit
    #[clap(default_value_t = 40, short, long)]
    workers: usize,

    /// request timeout ( seconds )
    #[clap(default_value_t = 10, short, long)]
    timeout: u64,

    /// revisit urls
    #[clap(short, long)]
    revisit: bool,

    /// verbose output
    #[clap(short, long)]
    verbose: bool,

    /// confine crawl inside given path ( alias of whitelist(url) )
    #[clap(short, long)]
    confine: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    let args = Arc::new(Arguments::parse());

    let futures = match &args.url {
        Some(url) => vec![crawl(args.clone(), url.clone())],
        None => std::io::stdin()
            .lines()
            .flatten()
            .map(|line| crawl(args.clone(), line))
            .collect(),
    };

    join_all(futures).await;

    Ok(())
}

async fn crawl(args: Arc<Arguments>, url: String) -> Result<()> {
    let mut seen: HashSet<String> = HashSet::new();

    let mut builder = Crawler::builder()
        .add_default_propagators()
        // .proxy("127.0.0.1:8080", "examples/cacert.der")?
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.5112.79 Safari/537.36".into())
        .workers(args.workers)
        .revisit(args.revisit)
        .on_page(|_args| {
            //println!("{}", &args.page.url.as_str());
        })
        .add_handler("*[href]", |args| {
            if let Some(href) = args.element.unwrap().value().attr("href") {
                if let Ok(abs_url) = absolute_url(&args.page.url, href) {
        if seen.insert(abs_url.to_string()) {
            println!("{}", abs_url.as_str());
        }
                }
            }
        })
        .depth(args.depth)
        .timeout(args.timeout, 0);

    if args.confine {
        builder = builder.whitelist(&url);
    }

    builder.build()?.crawl(&url).await?;

    Ok(())
}
