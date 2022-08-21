use crate::crawler_builder::CrawlerBuilder;
use anyhow::Result;
use futures::future::join_all;
use reqwest::{Client, Url};
use scraper::{ElementRef, Html, Selector};
use std::collections::HashMap;

pub struct Crawler {
    handlers: HashMap<String, Vec<Box<dyn Fn(ElementRef, Url)>>>,
    propagators: HashMap<String, Vec<Box<dyn Fn(ElementRef, Url) -> Option<Url>>>>,
    depth: u32,
    client: Client,
    blacklist: Vec<String>,
    whitelist: Vec<String>,
}

impl Crawler {
    pub fn builder() -> CrawlerBuilder {
        CrawlerBuilder::new()
    }

    pub fn from_builder(builder: CrawlerBuilder) -> Result<Self> {
        Ok(Self {
            handlers: builder.handlers,
            propagators: builder.propagators,
            depth: builder.depth,
            client: builder.client_builder.build()?,
            blacklist: builder.blacklist,
            whitelist: builder.whitelist,
        })
    }

    pub async fn crawl(&self, url: &str) -> Result<()> {
        let uri = Url::parse(url)?;
        self.visit(uri, self.depth).await?;
        Ok(())
    }

    fn is_allowed(&self, url: &Url) -> bool {
        if self.whitelist.len() > 0 {
            let mut contains = false;
            for expr in self.whitelist.iter() {
                if url.to_string().contains(expr) {
                    contains = true;
                    break;
                }
            }
            if !contains {
                return false;
            }
        }

        for expr in self.blacklist.iter() {
            if url.to_string().contains(expr) {
                return false;
            }
        }

        true
    }

    #[async_recursion::async_recursion(?Send)]
    async fn visit(&self, url: Url, depth: u32) -> Result<()> {
        if !self.is_allowed(&url) {
            return Ok(());
        }
        let res = self.client.get(url.clone()).send().await?;
        let text = res.text().await?;
        let doc = Html::parse_document(&text);

        for handlers in self.handlers.iter() {
            if let Ok(sel) = Selector::parse(handlers.0) {
                for handler in handlers.1 {
                    for el in doc.select(&sel) {
                        handler(el, url.clone());
                    }
                }
            } else {
                eprintln!("invalid selector {}", handlers.0);
            }
        }

        // continue propagating while depth is positive nonzero
        if depth > 0 {
            for propagator in self.propagators.iter() {
                if let Ok(sel) = Selector::parse(propagator.0) {
                    for propagator in propagator.1 {
                        join_all(doc.select(&sel).filter_map(|el| {
                            if let Some(url) = propagator(el, url.clone()) {
                                Some(self.visit(url, depth - 1))
                            } else {
                                None
                            }
                        }))
                        .await;
                    }
                } else {
                    eprintln!("invalid selector {}", propagator.0);
                }
            }
        }

        Ok(())
    }
}
