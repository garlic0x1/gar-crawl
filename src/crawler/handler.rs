use reqwest::{Client, Url};
use scraper::{ElementRef, Html};
use std::sync::Arc;

/// Handlers are void Fns
pub type Handler<'a> = Box<dyn FnMut(&HandlerArgs) + Send + Sync + 'a>;

/// Propagators return a `Vec<Url>` to queue
pub type Propagator<'a> = Box<dyn FnMut(&HandlerArgs) -> Vec<Url> + Send + Sync + 'a>;

/// Data to pass to the user as closure arguments
#[derive(Clone)]
pub struct HandlerArgs<'a> {
    /// Current page
    pub page: &'a Page,
    /// CSS element if available
    pub element: Option<ElementRef<'a>>,
    /// Reqwest client
    pub client: Arc<Client>,
}

/// Information about the current page
#[derive(Clone, Eq, PartialEq)]
pub struct Page {
    /// Url of the current location
    pub url: Url,
    /// Response body as a string
    pub text: String,
    /// Parsed HTML document
    pub doc: Html,
    /// Current crawl depth
    pub depth: usize,
}

/// These are the events you can hook into
#[derive(Clone, Eq, PartialEq, Hash)]
pub enum HandlerEvent {
    /// Handle all found matches of a CSS selector
    OnSelector(String),
    /// Handle every page loaded
    OnPage,
}
