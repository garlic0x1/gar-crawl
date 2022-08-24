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

    /// set the crawl depth ( default 2 )
    pub fn depth(mut self, depth: u32) -> Self {
        self.depth = depth;
        self
    }

    /// set the user agent
    pub fn user_agent(mut self, user_agent: &'a str) -> Self {
        self.client_builder = self.client_builder.user_agent(user_agent.to_string());
        self
    }

    /// set an https proxy with a cacert.der file
    pub fn proxy(mut self, proxy_str: &str, ca_cert: &str) -> Self {
        let mut buf = Vec::new();
        File::open(ca_cert).unwrap().read_to_end(&mut buf).unwrap();
        let cert = reqwest::Certificate::from_der(&buf).unwrap();

        let proxy = reqwest::Proxy::all(proxy_str).unwrap();

        self.client_builder = self.client_builder.add_root_certificate(cert).proxy(proxy);
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
    /// closure type: `FnMut(&HandlerArgs)`  
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

    /// add a propagator  
    /// closure type: `FnMut(&HandlerArgs) -> Option<Url>`  
    pub fn on_page_propagator<F>(mut self, closure: F) -> Self
    where
        F: FnMut(&HandlerArgs) -> Option<Url> + Send + Sync + 'a,
    {
        let closure: Propagator = Box::new(closure);
        if let Some(propagators) = self.propagators.get_mut(&HandlerEvent::OnPage) {
            propagators.push(closure)
        } else {
            self.propagators.insert(HandlerEvent::OnPage, vec![closure]);
        }
        self
    }
    /// add a handler  
    /// closure type: `FnMut(&HandlerArgs)`  
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
                .insert(HandlerEvent::OnSelector(sel.clone()), vec![closure]);
        }
        self
    }

    /// add a propagator  
    /// closure type: `FnMut(&HandlerArgs) -> Option<Url>`
    pub fn add_propagator<F>(mut self, sel: &str, closure: F) -> Self
    where
        F: FnMut(&HandlerArgs) -> Option<Url> + 'a + Send + Sync,
    {
        let sel = sel.to_string();
        let closure: Box<dyn FnMut(&HandlerArgs) -> Option<Url> + Send + Sync + 'a> =
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

    /// propagate on all href and src attributes
    pub fn add_default_propagators(mut self) -> Self {
        let href_prop = |args: &HandlerArgs| -> Option<Url> {
            if let Some(href) = args.element.unwrap().value().attr("href") {
                if let Ok(url) = Url::parse(href) {
                    Some(url)
                } else if let Ok(url) = args.page.url.join(href) {
                    Some(url)
                } else {
                    None
                }
            } else {
                None
            }
        };

        let src_prop = |args: &HandlerArgs| -> Option<Url> {
            if let Some(href) = args.element.unwrap().value().attr("src") {
                if let Ok(url) = Url::parse(href) {
                    Some(url)
                } else if let Ok(url) = args.page.url.join(href) {
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
