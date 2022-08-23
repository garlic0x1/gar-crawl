use reqwest::Url;
use scraper::{ElementRef, Html};

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
