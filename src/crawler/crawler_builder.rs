use super::crawler::*;
use anyhow::Result;
use reqwest::{Client, Url};
use scraper::ElementRef;
use std::collections::HashMap;
use std::marker::Send;

/// Builder object for Crawler, fields are left public
pub struct CrawlerBuilder<'a> {
    pub client_builder: reqwest::ClientBuilder,
    pub handlers: HashMap<HandlerEvent, Vec<HandlerFn<'a>>>,
    pub propagators: HashMap<HandlerEvent, Vec<PropagatorFn<'a>>>,
    pub depth: u32,
    pub blacklist: Vec<String>,
    pub whitelist: Vec<String>,
}

impl<'a> CrawlerBuilder<'a> {
    pub fn new() -> Self {
        Self {
            client_builder: Client::builder(),
            handlers: HashMap::new(),
            propagators: HashMap::new(),
            depth: 2,
            whitelist: vec![],
            blacklist: vec![],
        }
    }

    /// consume the Builder and produce a Crawler
    pub fn build(self) -> Result<Crawler<'a>> {
        Crawler::from_builder(self)
    }

    /// dont crawl a url containing expr
    pub fn blacklist(mut self, expr: &str) -> Self {
        self.blacklist.push(expr.to_string());
        self
    }

    /// only crawl urls containing expr
    pub fn whitelist(mut self, expr: &str) -> Self {
        self.whitelist.push(expr.to_string());
        self
    }

    /// set the user agent
    pub fn user_agent(mut self, user_agent: String) -> Self {
        self.client_builder = self.client_builder.user_agent(user_agent);
        self
    }

    /// set the crawl depth ( default 2 )
    pub fn depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }

    /// set the request timeout
    pub fn timeout(mut self, seconds: u64, nanoseconds: u32) -> Self {
        self.client_builder = self
            .client_builder
            .timeout(std::time::Duration::new(seconds, nanoseconds));
        self
    }

    /// add a handler  
    /// closure type: `FnMut(&Page)`  
    pub fn on_page<F>(mut self, closure: F) -> Self
    where
        F: FnMut(&Page) + Send + Sync + 'a,
    {
        let closure: Box<dyn FnMut(&Page) + Send + Sync + 'a> = Box::new(closure);
        let wrapped = HandlerFn::OnPage(closure);
        if let Some(handlers) = self.handlers.get_mut(&HandlerEvent::OnPage) {
            handlers.push(wrapped)
        } else {
            self.handlers.insert(HandlerEvent::OnPage, vec![wrapped]);
        }
        self
    }

    /// add a handler  
    /// closure type: `FnMut(ElementRef, &Page)`  
    pub fn add_handler<F>(mut self, sel: &str, closure: F) -> Self
    where
        F: FnMut(ElementRef, &Page) + Send + Sync + 'a,
    {
        let sel = sel.to_string();
        let closure: Box<dyn FnMut(ElementRef, &Page) + Send + Sync + 'a> = Box::new(closure);
        let wrapped = HandlerFn::OnSelector(closure);
        if let Some(handlers) = self
            .handlers
            .get_mut(&HandlerEvent::OnSelector(sel.clone()))
        {
            handlers.push(wrapped)
        } else {
            self.handlers
                .insert(HandlerEvent::OnSelector(sel.clone()), vec![wrapped]);
        }
        self
    }

    /// add a propagator  
    /// closure type: `FnMut(ElementRef, &Page) -> Option<Url>`
    pub fn add_propagator<F>(mut self, sel: &str, closure: F) -> Self
    where
        F: FnMut(ElementRef, &Page) -> Option<Url> + 'a + Send + Sync,
    {
        let sel = sel.to_string();
        let closure: Box<dyn FnMut(ElementRef, &Page) -> Option<Url> + Send + Sync + 'a> =
            Box::new(closure);
        let wrapped = PropagatorFn::OnSelector(closure);
        if let Some(propagators) = self
            .propagators
            .get_mut(&HandlerEvent::OnSelector(sel.clone()))
        {
            propagators.push(wrapped)
        } else {
            self.propagators
                .insert(HandlerEvent::OnSelector(sel), vec![wrapped]);
        }
        self
    }

    /// propagate on all href and src attributes
    pub fn add_default_propagators(mut self) -> Self {
        let href_prop = |el: ElementRef, page: &Page| -> Option<Url> {
            if let Some(href) = el.value().attr("href") {
                if let Ok(url) = Url::parse(href) {
                    Some(url)
                } else if let Ok(url) = page.url.join(href) {
                    Some(url)
                } else {
                    None
                }
            } else {
                None
            }
        };

        let src_prop = |el: ElementRef, page: &Page| -> Option<Url> {
            if let Some(href) = el.value().attr("src") {
                if let Ok(url) = Url::parse(href) {
                    Some(url)
                } else if let Ok(url) = page.url.join(href) {
                    Some(url)
                } else {
                    None
                }
            } else {
                None
            }
        };

        self = self.add_propagator("*[href]", href_prop);
        self = self.add_propagator("*[src]", src_prop);

        self
    }
}
