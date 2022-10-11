use reqwest::{Client, Request, Response};
use std::sync::Arc;

/// Handlers are void Fns
pub type FuzzHandler<'a> = Box<dyn FnMut(FuzzHandlerArgs) + Send + Sync + 'a>;

/// Data to pass to the user as closure arguments
pub struct FuzzHandlerArgs<'a> {
    /// Original Request
    pub request: &'a Request,
    /// HTTP Response
    pub response: &'a Response,
    /// Reqwest client
    pub client: Arc<Client>,
}
