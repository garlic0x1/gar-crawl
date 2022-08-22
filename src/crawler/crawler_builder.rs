use super::crawler::*;
use anyhow::Result;
use reqwest::{Client, ClientBuilder, Url};
use scraper::ElementRef;
use std::collections::HashMap;
use std::marker::Send;

/// Builder object for Crawler
pub struct CrawlerBuilder {
    pub client_builder: ClientBuilder,
    pub handlers: HashMap<HandlerEvent, Vec<HandlerFn>>,
    pub propagators: HashMap<HandlerEvent, Vec<PropagatorFn>>,
    pub depth: u32,
    pub blacklist: Vec<String>,
    pub whitelist: Vec<String>,
}

impl CrawlerBuilder {
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
    pub fn build(self) -> Result<Crawler> {
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

    /// set the request timeout ( seconds u64, nanoseconds: u32 )
    pub fn timeout(mut self, seconds: u64, nanoseconds: u32) -> Self {
        self.client_builder = self
            .client_builder
            .timeout(std::time::Duration::new(seconds, nanoseconds));
        self
    }

    pub fn on_page<F>(mut self, closure: F) -> Self
    where
        F: FnMut(&Page) + Send + Sync + 'static,
    {
        let closure: Box<dyn FnMut(&Page) + Send + Sync + 'static> = Box::new(closure);
        let wrapped = HandlerFn::OnPage(closure);
        if let Some(handlers) = self.handlers.get_mut(&HandlerEvent::OnPage) {
            handlers.push(wrapped)
        } else {
            self.handlers.insert(HandlerEvent::OnPage, vec![wrapped]);
        }
        self
    }

    /// add a handler
    /// selector: String
    /// closure: FnMut(ElementRef, Page)
    pub fn add_handler<F>(mut self, sel: &str, closure: F) -> Self
    where
        F: FnMut(ElementRef, &Page) + Send + Sync + 'static,
    {
        let sel = sel.to_string();
        let closure: Box<dyn FnMut(ElementRef, &Page) + Send + Sync + 'static> = Box::new(closure);
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
    /// selector: String
    /// closure: FnMut(&Self, ElementRef, source: Url, depth: u32)
    pub fn add_propagator<F>(mut self, sel: &str, closure: F) -> Self
    where
        F: FnMut(ElementRef, &Page) -> Option<Url> + 'static + Send + Sync,
    {
        let sel = sel.to_string();
        let closure: Box<dyn FnMut(ElementRef, &Page) -> Option<Url> + Send + Sync + 'static> =
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
                if let Ok(abs_url) = page.url.join(href) {
                    return Some(abs_url);
                } else {
                    return Some(page.url.clone());
                }
            }
            None
        };

        let src_prop = |el: ElementRef, page: &Page| -> Option<Url> {
            if let Some(href) = el.value().attr("src") {
                if let Ok(abs_url) = page.url.join(href) {
                    return Some(abs_url);
                } else {
                    return Some(page.url.clone());
                }
            }
            None
        };

        self = self.add_propagator("*[href]", href_prop);
        self = self.add_propagator("*[src]", src_prop);

        self
    }
}
