use crate::crawler_builder::CrawlerBuilder;
use anyhow::{bail, Result};
use async_channel::*;
use futures::{future::join_all, Future};
use reqwest::{Client, Url};
use scraper::{ElementRef, Html, Selector};
use std::collections::{HashMap, HashSet, VecDeque};
use std::marker::Send;

pub struct Crawler {
    handlers: HashMap<String, Vec<Box<dyn FnMut(ElementRef, Url) + Send + 'static>>>,
    propagators:
        HashMap<String, Vec<Box<dyn FnMut(ElementRef, Url) -> Option<Url> + Send + 'static>>>,
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

    pub async fn crawl(self, url: &str) -> Result<()> {
        smol::block_on(async {
            let uri = Url::parse(url)?;

            let mut queue: VecDeque<(Url, u32)> = VecDeque::new();
            let mut seen: HashSet<Url> = HashSet::new();
            seen.insert(uri.clone());
            queue.push_back((uri.clone(), self.depth));
            let (s, r): (Sender<(Url, u32)>, Receiver<(Url, u32)>) = bounded(100);
            let mut tasks = 0;

            // Loop while the queue is not empty or tasks are fetching pages.
            while queue.len() + tasks > 0 {
                // Limit the number of concurrent tasks.
                while tasks < s.capacity().unwrap() {
                    // Process URLs in the queue and fetch more pages.
                    match queue.pop_front() {
                        None => break,
                        Some(url) => {
                            println!("{}", url);
                            tasks += 1;
                            smol::spawn(self.visit(url, s.clone())).detach();
                        }
                    }
                }
            }

            Ok(())
        })
    }

    async fn visit(&mut self, url: Url, depth: u32) -> Result<()> {
        if !self.is_allowed(&url) {
            return Ok(());
        }
        let res = self.client.get(url.clone()).send().await?;
        let text = res.text().await?;
        let doc = Html::parse_document(&text);

        self.do_handlers(&url, &doc).await?;

        // continue propagating while depth is positive nonzero
        if depth > 0 {
            self.propagate(&url, &doc, depth).await?;
        }

        Ok(())
    }

    async fn do_handlers(&mut self, url: &Url, doc: &Html) -> Result<()> {
        for handlers in self.handlers.iter_mut() {
            if let Ok(sel) = Selector::parse(handlers.0) {
                for handler in handlers.1.iter_mut() {
                    for el in doc.select(&sel) {
                        handler(el, url.clone());
                    }
                }
            } else {
                bail!("invalid selector {}", handlers.0);
            }
        }
        Ok(())
    }

    async fn propagate(&mut self, url: &Url, doc: &Html, depth: u32) -> Result<()> {
        for propagator in self.propagators.iter_mut() {
            if let Ok(sel) = Selector::parse(propagator.0) {
                for propagator in propagator.1.iter_mut() {
                    for el in doc.select(&sel) {
                        if let Some(url) = propagator(el, url.clone()) {
                            self.queue.0.send((url, depth - 1)).await?;
                        }
                    }
                }
            } else {
                bail!("invalid selector {}", propagator.0);
            }
        }
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
}
