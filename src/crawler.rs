use crate::crawler_builder::CrawlerBuilder;
use anyhow::{bail, Result};
use async_channel::*;
use reqwest::{Client, Url};
use scraper::{ElementRef, Html, Selector};
use std::collections::{HashMap, HashSet, VecDeque};
use std::marker::Send;

pub struct Crawler {
    handlers: HashMap<String, Vec<Box<dyn FnMut(ElementRef, Url) + Send + Sync + 'static>>>,
    propagators: HashMap<
        String,
        Vec<Box<dyn FnMut(ElementRef, Url) -> Option<Url> + Send + Sync + 'static>>,
    >,
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

    pub async fn crawl(&mut self, url: &str) -> Result<()> {
        let uri: Url = Url::parse(url)?;
        let client = self.client.clone();

        let mut queue: VecDeque<(Url, u32)> = VecDeque::new();
        let mut seen: HashSet<Url> = HashSet::new();
        seen.insert(uri.clone());
        queue.push_back((uri.clone(), self.depth));
        let (s, r) = bounded(100);
        let mut tasks = 0;

        // Loop while the queue is not empty or tasks are fetching pages.
        while queue.len() + tasks > 0 {
            // Limit the number of concurrent tasks.
            while tasks < s.capacity().unwrap() {
                // Process URLs in the queue and fetch more pages.
                match queue.pop_front() {
                    None => break,
                    Some(url) => {
                        if self.is_allowed(&url.0) {
                            tasks += 1;
                            tokio::spawn(Self::fetch(url.0, url.1, client.clone(), s.clone()));
                        }
                    }
                }
            }

            // Get a fetched web page.
            let (furl, text, depth) = r.recv().await.unwrap();
            let doc = Html::parse_document(&text);
            tasks -= 1;

            self.do_handlers(&furl, &doc)?;

            if depth > 0 {
                self.do_propagators(&furl, &doc, depth, &mut queue)?;
            }
        }

        Ok(())
    }

    fn do_propagators(
        &mut self,
        url: &Url,
        doc: &Html,
        depth: u32,
        queue: &mut VecDeque<(Url, u32)>,
    ) -> Result<()> {
        for propagator in self.propagators.iter_mut() {
            if let Ok(sel) = Selector::parse(propagator.0) {
                for propagator in propagator.1.iter_mut() {
                    for el in doc.select(&sel) {
                        if let Some(url) = propagator(el, url.clone()) {
                            queue.push_back((url, depth - 1));
                        }
                    }
                }
            } else {
                bail!("invalid selector {}", propagator.0);
            }
        }
        Ok(())
    }

    fn do_handlers(&mut self, url: &Url, doc: &Html) -> Result<()> {
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

    async fn fetch(
        url: Url,
        depth: u32,
        client: Client,
        sender: Sender<(Url, String, u32)>,
    ) -> Result<()> {
        let res = client.get(url.clone()).send().await?;
        let text = res.text().await?;
        sender.send((url, text, depth)).await?;
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
