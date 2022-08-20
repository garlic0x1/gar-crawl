use std::collections::HashMap;

use anyhow::Result;
use reqwest::*;
use scraper::*;
use tokio::net::TcpStream;

pub struct Crawler<'a> {
    handlers: HashMap<&'a str, Vec<Box<dyn Fn(ElementRef)>>>,
    depth: u32,
    workers: u32,
    headers: Vec<(String, String)>,
    client: Client,
}

impl<'a> Crawler<'a> {
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            depth: 2,
            workers: 4,
            headers: vec![("User-Agent".into(), "garlic_crawler".into())],
            client: Client::new(),
        }
    }

    pub fn add_handler(mut self, sel: &'a str, closure: Box<dyn Fn(ElementRef)>) -> Self {
        if let Some(handlers) = self.handlers.get_mut(sel) {
            handlers.push(closure)
        } else {
            self.handlers.insert(sel, vec![closure]);
        }
        self
    }

    pub async fn crawl(&self, url: &str) -> Result<()> {
        let uri = Url::parse(url)?;
        self.visit(uri, self.depth).await?;
        Ok(())
    }

    async fn visit(&self, url: Url, depth: u32) -> Result<()> {
        let res = self.client.get(url).send().await?;
        let text = res.text().await?;
        let doc = Html::parse_document(&text);

        for handlers in self.handlers.iter() {
            if let Ok(sel) = Selector::parse(handlers.0) {
                for handler in handlers.1 {
                    for el in doc.select(&sel) {
                        handler(el);
                    }
                }
            } else {
                eprintln!("invalid selector {}", handlers.0);
            }
        }
        Ok(())
    }
}
