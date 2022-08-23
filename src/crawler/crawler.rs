use crate::crawler::CrawlerBuilder;
use anyhow::{anyhow, bail, Result};
use async_channel::*;
use reqwest::{Client, Url};
use scraper::{ElementRef, Html, Selector};
use std::collections::{HashMap, HashSet, VecDeque};
use std::marker::Send;
use std::sync::Arc;

/// Data to pass to the user as closure arguments
#[derive(Clone, Eq, PartialEq)]
pub struct HandlerArgs<'a> {
    /// current page
    pub page: &'a Page,
    /// CSS element if available
    pub element: Option<ElementRef<'a>>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct Page {
    /// Url of the current location
    pub url: Url,
    /// Response body as a string
    pub text: String,
    /// Parsed HTML document
    pub doc: Html,
}

/// These are the events you can hook into
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum HandlerEvent {
    /// Handle all found matches of a CSS selector
    OnSelector(String),
    /// Handle every page loaded
    OnPage,
}

/// Closure types for handlers
pub enum HandlerFn<'a> {
    OnSelector(Box<dyn FnMut(&HandlerArgs) + Send + Sync + 'a>),
    OnPage(Box<dyn FnMut(&HandlerArgs) + Send + Sync + 'a>),
}

/// Closure types for propagators
pub enum PropagatorFn<'a> {
    OnSelector(Box<dyn FnMut(&HandlerArgs) -> Option<Url> + Send + Sync + 'a>),
    OnPage(Box<dyn FnMut(&HandlerArgs) -> Option<Url> + Send + Sync + 'a>),
}

/// A crawler object, use builder() to build with CrawlerBuilder
pub struct Crawler<'a> {
    handlers: HashMap<HandlerEvent, Vec<HandlerFn<'a>>>,
    propagators: HashMap<HandlerEvent, Vec<PropagatorFn<'a>>>,
    depth: u32,
    client: Client,
    blacklist: Vec<String>,
    whitelist: Vec<String>,
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
            client: builder.client_builder.build()?,
            blacklist: builder.blacklist,
            whitelist: builder.whitelist,
        })
    }

    /// Start crawling at the provided URL
    /// NOTE: "https://example.com/path" and "https://example.com/path/" may behave differently
    /// see https://docs.rs/reqwest/0.10.8/reqwest/struct.Url.html#method.join
    pub async fn crawl(&mut self, start_url: &str) -> Result<Vec<anyhow::Error>> {
        let uri: Url = Url::parse(start_url)?;
        let client = Arc::new(self.client.clone());
        let mut errors = vec![];
        let mut seen: HashSet<Url> = HashSet::new();
        seen.insert(uri.clone());

        // set up async
        let mut queue: VecDeque<(Url, u32)> = VecDeque::new();
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
                        tasks += 1;
                        tokio::spawn(Self::fetch(url.0, url.1, client.clone(), s.clone()));
                    }
                }
            }

            // Get a fetched web page.
            let fetched = r.recv().await.unwrap();
            tasks -= 1;

            if let Err(fetch_err) = fetched {
                errors.push(fetch_err);
                continue;
            }

            let (url, text, depth) = fetched.unwrap();
            let doc = Html::parse_document(&text);
            let page = Page { url, text, doc };

            self.do_handlers(&page)?;

            if depth > 0 {
                self.do_propagators(&page, depth, &mut queue)?;
            }
        }

        Ok(errors)
    }

    fn do_propagators(
        &mut self,
        page: &Page,
        depth: u32,
        queue: &mut VecDeque<(Url, u32)>,
    ) -> Result<()> {
        let props = &mut self.propagators;
        for propagator in props.iter_mut() {
            match propagator.0 {
                HandlerEvent::OnSelector(sel) => {
                    if let Ok(sel) = Selector::parse(&sel) {
                        for propagator in propagator.1.iter_mut() {
                            match propagator {
                                PropagatorFn::OnSelector(prop) => {
                                    for el in page.doc.select(&sel) {
                                        let args = HandlerArgs {
                                            page,
                                            element: Some(el),
                                        };
                                        if let Some(url) = prop(&args) {
                                            if Self::is_allowed(
                                                &url,
                                                &self.whitelist,
                                                &self.blacklist,
                                            ) {
                                                queue.push_back((url, depth - 1));
                                            }
                                        }
                                    }
                                }
                                PropagatorFn::OnPage(_) => (), // wrong kind
                            }
                        }
                    } else {
                        bail!("invalid selector {}", sel);
                    }
                }
                HandlerEvent::OnPage => (), // TODO
            }
        }
        Ok(())
    }

    fn do_handlers(&mut self, page: &Page) -> Result<()> {
        for handlers in self.handlers.iter_mut() {
            match handlers.0 {
                HandlerEvent::OnSelector(sel) => {
                    if let Ok(sel) = Selector::parse(sel) {
                        for handler in handlers.1.iter_mut() {
                            if let HandlerFn::OnSelector(handler) = handler {
                                for el in page.doc.select(&sel) {
                                    let args = HandlerArgs {
                                        page,
                                        element: Some(el),
                                    };
                                    handler(&args);
                                }
                            }
                        }
                    } else {
                        bail!("invalid selector {}", sel);
                    }
                }
                HandlerEvent::OnPage => {
                    handlers.1.iter_mut().for_each(|h| {
                        if let HandlerFn::OnPage(handler) = h {
                            let args = HandlerArgs {
                                page,
                                element: None,
                            };
                            handler(&args);
                        }
                    });
                }
            }
        }
        Ok(())
    }

    /// make a request and send the results on the async chan
    async fn fetch(
        url: Url,
        depth: u32,
        client: Arc<Client>,
        sender: Sender<Result<(Url, String, u32)>>,
    ) -> Result<()> {
        if let Ok(res) = client.get(url.clone()).send().await {
            if let Ok(text) = res.text().await {
                sender.send(Ok((url, text, depth))).await.unwrap();
                return Ok(());
            }
        }
        sender.send(Err(anyhow!(""))).await.unwrap();
        bail!("");
    }

    /// match whitelist/blacklist rules
    fn is_allowed(url: &Url, wl: &Vec<String>, bl: &Vec<String>) -> bool {
        let surl = url.to_string();
        if wl
            .iter()
            .filter(|expr| surl.contains(expr.as_str()))
            .take(1)
            .count()
            == 0
            && wl.len() > 0
        {
            false
        } else if bl
            .iter()
            .filter(|expr| surl.contains(expr.as_str()))
            .take(1)
            .count()
            != 0
        {
            false
        } else {
            true
        }
    }
}
