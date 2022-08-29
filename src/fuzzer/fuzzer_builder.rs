use super::fuzzer::*;
use super::handler::*;

use anyhow::Result;
use reqwest::Client;
use std::fs::File;
use std::io::Read;
use std::marker::Send;

pub struct FuzzerBuilder<'a> {
    pub client_builder: reqwest::ClientBuilder,
    pub handlers: Vec<FuzzHandler<'a>>,
    pub workers: usize,
}

impl<'a> FuzzerBuilder<'a> {
    pub fn new() -> Self {
        Self {
            client_builder: Client::builder(),
            handlers: Vec::new(),
            workers: 40,
        }
    }

    /// Add a handler  
    /// Closure type: `FnMut(&FuzzHandlerArgs)`  
    pub fn add_handler<F>(mut self, closure: F) -> Self
    where
        F: FnMut(FuzzHandlerArgs) + Send + Sync + 'a,
    {
        let closure: Box<dyn FnMut(FuzzHandlerArgs) + Send + Sync + 'a> = Box::new(closure);
        self.handlers.push(closure);
        self
    }

    pub fn build(self) -> Result<Fuzzer<'a>> {
        Fuzzer::from_builder(self)
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

    /// Set the concurrency limit ( default: 40 )
    pub fn workers(mut self, limit: usize) -> Self {
        self.workers = limit;
        self
    }
}
