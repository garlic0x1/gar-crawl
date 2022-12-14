// use super::courier;

use crate::auxiliary::*;
use crate::crawler::*;
use anyhow::{bail, Result};
use async_channel::*;
use reqwest::{Client, Url};
use scraper::{Html, Selector};
use std::collections::{HashMap, HashSet, VecDeque};
use std::sync::Arc;

/// A crawler object, use builder() to build with CrawlerBuilder
pub struct Crawler<'a> {
    handlers: HashMap<HandlerEvent, Vec<Handler<'a>>>,
    propagators: HashMap<HandlerEvent, Vec<Propagator<'a>>>,
    depth: usize,
    workers: usize,
    client: Arc<Client>,
    blacklist: Vec<String>,
    whitelist: Vec<String>,
    visited: HashSet<String>,
    revisit: bool,
}

impl<'a> Crawler<'a> {
    /// Get a CrawlerBuilder
    /// Equivalent to `CrawlerBuilder::new()`
    pub fn builder() -> CrawlerBuilder<'a> {
        CrawlerBuilder::new()
    }

    /// Create a crawler, consuming a CrawlerBuilder
    /// Equivalent to `CrawlerBuilder.build()`
    pub fn from_builder(builder: CrawlerBuilder<'a>) -> Result<Self> {
        Ok(Self {
            handlers: builder.handlers,
            propagators: builder.propagators,
            depth: builder.depth,
            workers: builder.workers,
            client: Arc::new(builder.client_builder.build()?),
            blacklist: builder.blacklist,
            whitelist: builder.whitelist,
            visited: HashSet::new(),
            revisit: builder.revisit,
        })
    }

    /// Start crawling at the provided URL and return errors that occur
    pub async fn crawl(&mut self, start_url: &str) -> Result<Vec<anyhow::Error>> {
        let uri: Url = Url::parse(start_url)?;
        let mut errors = vec![];
        let mut seen: HashSet<Url> = HashSet::new();
        seen.insert(uri.clone());

        // set up async
        let mut queue: VecDeque<(Url, usize)> = VecDeque::new();
        queue.push_back((uri.clone(), 0));
        let (s, r) = bounded(self.workers);
        let mut tasks = 0;

        // Loop while the queue is not empty or tasks are fetching pages.
        while queue.len() + tasks > 0 {
            // Limit the number of concurrent tasks.
            while tasks < s.capacity().unwrap() {
                // Process URLs in the queue and fetch more pages.
                match queue.pop_front() {
                    None => break,
                    Some(url) => {
                        tasks += 1;
                        tokio::spawn(courier::fetch(url.0, url.1, self.client.clone(), s.clone()));
                    }
                }
            }

            // Recieve a message
            let fetched = r.recv().await.unwrap();
            tasks -= 1;

            if let Err(fetch_err) = fetched {
                errors.push(fetch_err);
                continue;
            }

            // wrap up data for handlers
            let (url, text, depth) = fetched.unwrap();
            let doc = Html::parse_document(&text);
            let page = Page {
                url,
                text,
                doc,
                depth,
            };

            self.do_handlers(&page)?;

            if depth < self.depth {
                self.do_propagators(&page, &mut queue)?;
            }
        }

        Ok(errors)
    }

    fn do_propagators(&mut self, page: &Page, queue: &mut VecDeque<(Url, usize)>) -> Result<()> {
        let wl = &self.whitelist;
        let bl = &self.blacklist;
        let visited = &mut self.visited;
        let revisit = self.revisit;

        for (kind, props) in self.propagators.iter_mut() {
            match kind {
                HandlerEvent::OnSelector(sel) => {
                    if let Ok(sel) = Selector::parse(sel) {
                        props.iter_mut().for_each(|propagator| {
                            page.doc.select(&sel).for_each(|el| {
                                propagator(&HandlerArgs {
                                    page,
                                    element: Some(el),
                                    client: self.client.clone(),
                                })
                                .iter()
                                .filter(|u| {
                                    is_allowed(u, wl, bl) && (revisit || !is_visited(u, visited))
                                })
                                .for_each(|u| {
                                    queue.push_back((u.clone(), page.depth + 1));
                                });
                            });
                        });
                    } else {
                        bail!("invalid selector {}", sel);
                    }
                }
                HandlerEvent::OnPage => {
                    props.iter_mut().for_each(|propagator| {
                        propagator(&HandlerArgs {
                            page,
                            element: None,
                            client: self.client.clone(),
                        })
                        .iter()
                        .filter(|u| is_allowed(u, wl, bl) && (revisit || !is_visited(u, visited)))
                        .for_each(|u| {
                            queue.push_back((u.clone(), page.depth + 1));
                        });
                    });
                }
            }
        }
        Ok(())
    }

    fn do_handlers(&mut self, page: &Page) -> Result<()> {
        for (kind, handlers) in self.handlers.iter_mut() {
            match kind {
                HandlerEvent::OnSelector(sel) => {
                    if let Ok(sel) = Selector::parse(sel) {
                        handlers.iter_mut().for_each(|handler| {
                            page.doc.select(&sel).for_each(|el| {
                                handler(&HandlerArgs {
                                    page,
                                    element: Some(el),
                                    client: self.client.clone(),
                                });
                            });
                        });
                    } else {
                        bail!("invalid selector {}", sel);
                    }
                }
                HandlerEvent::OnPage => {
                    handlers.iter_mut().for_each(|handler| {
                        handler(&HandlerArgs {
                            page,
                            element: None,
                            client: self.client.clone(),
                        });
                    });
                }
            }
        }
        Ok(())
    }
}
