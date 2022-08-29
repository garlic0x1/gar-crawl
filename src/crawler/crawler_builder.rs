use crate::absolute_url;

use super::crawler::*;
use super::handler::*;
use anyhow::Result;
use reqwest::{Client, Url};
use std::collections::HashMap;
use std::fs::File;
use std::io::Read;
use std::marker::Send;

/// Builder object for Crawler, fields are left public
pub struct CrawlerBuilder<'a> {
    pub client_builder: reqwest::ClientBuilder,
    pub handlers: HashMap<HandlerEvent, Vec<Handler<'a>>>,
    pub propagators: HashMap<HandlerEvent, Vec<Propagator<'a>>>,
    pub depth: usize,
    pub workers: usize,
    pub blacklist: Vec<String>,
    pub whitelist: Vec<String>,
    pub revisit: bool,
}

impl<'a> CrawlerBuilder<'a> {
    pub fn new() -> Self {
        Self {
            client_builder: Client::builder(),
            handlers: HashMap::new(),
            propagators: HashMap::new(),
            depth: 2,
            workers: 40,
            whitelist: vec![],
            blacklist: vec![],
            revisit: false,
        }
    }

    /// Consume the Builder and produce a Crawler
    pub fn build(self) -> Result<Crawler<'a>> {
        Crawler::from_builder(self)
    }

    /// Don't crawl a Url containing `expr`
    pub fn blacklist(mut self, expr: &str) -> Self {
        self.blacklist.push(expr.to_string());
        self
    }

    /// Only crawl Urls containing `expr`
    pub fn whitelist(mut self, expr: &str) -> Self {
        self.whitelist.push(expr.to_string());
        self
    }

    /// Set the crawl depth ( default: 2 )
    pub fn depth(mut self, depth: usize) -> Self {
        self.depth = depth;
        self
    }

    /// Revisit pages ( default: false )
    pub fn revisit(mut self, revisit: bool) -> Self {
        self.revisit = revisit;
        self
    }

    /// Set the concurrency limit ( default: 40 )
    pub fn workers(mut self, limit: usize) -> Self {
        self.workers = limit;
        self
    }
    /// Set the user agent
    pub fn user_agent(mut self, user_agent: &'a str) -> Self {
        self.client_builder = self.client_builder.user_agent(user_agent.to_string());
        self
    }

    /// Set an https proxy with a cacert.der file
    pub fn proxy(mut self, proxy_str: &str, ca_cert: &str) -> Result<Self> {
        let mut buf = Vec::new();
        File::open(ca_cert)?.read_to_end(&mut buf)?;
        let cert = reqwest::Certificate::from_der(&buf)?;

        let proxy = reqwest::Proxy::all(proxy_str)?;

        self.client_builder = self.client_builder.add_root_certificate(cert).proxy(proxy);
        Ok(self)
    }

    /// Set the request timeout
    pub fn timeout(mut self, seconds: u64, nanoseconds: u32) -> Self {
        self.client_builder = self
            .client_builder
            .timeout(std::time::Duration::new(seconds, nanoseconds));
        self
    }

    /// Add a handler  
    /// Closure type: `FnMut(&HandlerArgs)`  
    pub fn on_page<F>(mut self, closure: F) -> Self
    where
        F: FnMut(&HandlerArgs) + Send + Sync + 'a,
    {
        let closure: Box<dyn FnMut(&HandlerArgs) + Send + Sync + 'a> = Box::new(closure);
        if let Some(handlers) = self.handlers.get_mut(&HandlerEvent::OnPage) {
            handlers.push(closure)
        } else {
            self.handlers.insert(HandlerEvent::OnPage, vec![closure]);
        }
        self
    }

    /// Add a propagator  
    /// Closure type: `FnMut(&HandlerArgs) -> Vec<Url>`  
    pub fn on_page_propagator<F>(mut self, closure: F) -> Self
    where
        F: FnMut(&HandlerArgs) -> Vec<Url> + Send + Sync + 'a,
    {
        let closure: Propagator = Box::new(closure);
        if let Some(propagators) = self.propagators.get_mut(&HandlerEvent::OnPage) {
            propagators.push(closure)
        } else {
            self.propagators.insert(HandlerEvent::OnPage, vec![closure]);
        }
        self
    }

    /// Add a handler  
    /// Closure type: `FnMut(&HandlerArgs)`  
    pub fn add_handler<F>(mut self, sel: &str, closure: F) -> Self
    where
        F: FnMut(&HandlerArgs) + Send + Sync + 'a,
    {
        let sel = sel.to_string();
        let closure: Box<dyn FnMut(&HandlerArgs) + Send + Sync + 'a> = Box::new(closure);
        if let Some(handlers) = self
            .handlers
            .get_mut(&HandlerEvent::OnSelector(sel.clone()))
        {
            handlers.push(closure)
        } else {
            self.handlers
                .insert(HandlerEvent::OnSelector(sel), vec![closure]);
        }
        self
    }

    /// Add a propagator  
    /// Closure type: `FnMut(&HandlerArgs) -> Vec<Url>`
    pub fn add_propagator<F>(mut self, sel: &str, closure: F) -> Self
    where
        F: FnMut(&HandlerArgs) -> Vec<Url> + 'a + Send + Sync,
    {
        let sel = sel.to_string();
        let closure: Box<dyn FnMut(&HandlerArgs) -> Vec<Url> + Send + Sync + 'a> =
            Box::new(closure);
        if let Some(propagators) = self
            .propagators
            .get_mut(&HandlerEvent::OnSelector(sel.clone()))
        {
            propagators.push(closure)
        } else {
            self.propagators
                .insert(HandlerEvent::OnSelector(sel), vec![closure]);
        }
        self
    }

    /// Propagate on all href and src attributes  
    /// NOTE: "scheme://domain.tld/path" and "scheme://domain.tld/path/" may behave differently,  
    /// see <https://docs.rs/reqwest/0.10.8/reqwest/struct.Url.html#method.join> for info.
    pub fn add_default_propagators(mut self) -> Self {
        let href_prop = |args: &HandlerArgs| -> Vec<Url> {
            if let Some(href) = args.element.unwrap().value().attr("href") {
                if let Ok(url) = absolute_url(&args.page.url, href) {
                    return vec![url];
                }
            }
            vec![]
        };

        let src_prop = |args: &HandlerArgs| -> Vec<Url> {
            if let Some(href) = args.element.unwrap().value().attr("src") {
                if let Ok(url) = absolute_url(&args.page.url, href) {
                    return vec![url];
                }
            }
            vec![]
        };

        self = self.add_propagator("*[href]", href_prop);
        self = self.add_propagator("*[src]", src_prop);

        self
    }
}
