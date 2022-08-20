use hyper::header::{Headers, UserAgent};
use hyper::*;

pub struct Worker<C> {
    client: Client<C>,
}

impl<C> Worker<C> {
    pub fn new(headers: Vec<String>) -> Self {
        let mut headers = Headers::new();
        headers.set(UserAgent("hyper/0.5.2".to_owned()));
        Self {
            client: Client::configure().
        }
    }
}
