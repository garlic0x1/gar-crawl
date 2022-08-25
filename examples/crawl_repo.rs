use anyhow::Result;
use gar_crawl::crawler::*;
use std::collections::HashMap;

#[tokio::main]
async fn main() -> Result<()> {
    let repo_url = "http://plugins.svn.wordpress.org/qiwi-button/trunk/";

    // filenames and contents
    let mut files: HashMap<String, String> = HashMap::new();

    let errs = Crawler::builder()
        .add_default_propagators()
        .whitelist(repo_url)
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.5112.79 Safari/537.36".into())
        .on_page(|args| {
            if args.page.url.to_string().contains(".php") {
                files.insert(args.page.url.to_string(), args.page.text.clone());
            }
        })
        .depth(1)
        .build()?
        .crawl(repo_url)
        .await?;

    println!("errs: {} \n{:?}", errs.len(), errs);
    println!(
        "files downloaded: {:?}",
        files
            .iter()
            .map(|(n, _t)| n.clone())
            .collect::<Vec<String>>()
    );

    Ok(())
}
