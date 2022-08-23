use anyhow::Result;
use gar_crawl::crawler::*;
use reqwest::Url;
use scraper::ElementRef;
use std::collections::HashSet;

#[tokio::main]
async fn main() -> Result<()> {
    let repo_url = "http://plugins.svn.wordpress.org/qiwi-button/trunk";

    let mut seen: HashSet<String> = HashSet::new();

    let broken_link_prop = |el: ElementRef, page: &Page| -> Option<Url> {
        if let Some(href) = el.value().attr("href") {
            println!("{href}");
            if href.starts_with("http") || href.starts_with("/") {
                if let Ok(abs_url) = page.url.join(href) {
                    Some(abs_url)
                } else {
                    if let Ok(url) = Url::parse(href) {
                        Some(url)
                    } else {
                        println!("unparsable");
                        None
                    }
                }
            } else {
                let href = &format!("/{}", href);
                println!("{href}");
                if let Ok(abs_url) = page.url.join(href) {
                    println!("{}", abs_url);
                    Some(abs_url)
                } else {
                    if let Ok(url) = Url::parse(href) {
                        println!("{}", url);
                        Some(url)
                    } else {
                        println!("unparsable");
                        None
                    }
                }
            }
        } else {
            None
        }
    };

    Crawler::builder()
        .add_propagator("*[href]", broken_link_prop)
        .whitelist(repo_url)
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.5112.79 Safari/537.36".into())
        .on_page( |page: &Page| {
            if page.url.to_string().contains(".php") {
                out(&page.url.to_string(),  &mut seen);
            }
        })
        .depth(1)
        .build()?
        .crawl(repo_url)
        .await?;

    println!("{}", seen.len());

    Ok(())
}

fn out(message: &str, seen: &mut HashSet<String>) {
    if seen.insert(message.to_string()) {
        println!("{message}")
    }
}
