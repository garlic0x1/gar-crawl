use std::sync::Arc;

use reqwest::{Client, Url};
use scraper::{ElementRef, Html};

/// Handlers are void fns
pub type Handler<'a> = Box<dyn FnMut(&HandlerArgs) + Send + Sync + 'a>;

// Propagators return a Url to queue (TODO: return a vec of Urls instead of a single Url)
pub type Propagator<'a> = Box<dyn FnMut(&HandlerArgs) -> Option<Url> + Send + Sync + 'a>;

/// Data to pass to the user as closure arguments
#[derive(Clone)]
pub struct HandlerArgs<'a> {
    /// current page
    pub page: &'a Page,
    /// CSS element if available
    pub element: Option<ElementRef<'a>>,
    /// Reqwest client
    pub client: Arc<Client>,
}

#[derive(Clone, Eq, PartialEq)]
pub struct Page {
    /// Url of the current location
    pub url: Url,
    /// Response body as a string
    pub text: String,
    /// Parsed HTML document
    pub doc: Html,
    /// Current crawl depth
    pub depth: u32,
}

/// These are the events you can hook into
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum HandlerEvent {
    /// Handle all found matches of a CSS selector
    OnSelector(String),
    /// Handle every page loaded
    OnPage,
}
