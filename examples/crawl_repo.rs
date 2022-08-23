use anyhow::Result;
use gar_crawl::crawler::*;
use std::collections::HashSet;

#[tokio::main]
async fn main() -> Result<()> {
    let repo_url = "http://plugins.svn.wordpress.org/qiwi-button/trunk/";

    let mut seen: HashSet<String> = HashSet::new();

    let errs = Crawler::builder()
        .add_default_propagators()
        .whitelist(repo_url)
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.5112.79 Safari/537.36".into())
        .on_page(|page: &Page| {
            if page.url.to_string().contains(".php") {
                out(&page.url.to_string(),  &mut seen);
            }
        })
        .depth(1)
        .build()?
        .crawl(repo_url)
        .await?;

    println!("{}", seen.len());
    println!("errs: {} \n{:?}", errs.len(), errs);

    Ok(())
}

fn out(message: &str, seen: &mut HashSet<String>) {
    if seen.insert(message.to_string()) {
        println!("{message}")
    }
}
