use crate::fuzzer::*;
use anyhow::{anyhow, Result};
use async_channel::*;
use reqwest::{Client, Url};
use reqwest::{Request, Response};
use std::sync::Arc;

/// A crawler object, use builder() to build with FuzzerBuilder
pub struct Fuzzer<'a> {
    handlers: Vec<FuzzHandler<'a>>,
    workers: usize,
    client: Arc<Client>,
}

impl<'a> Fuzzer<'a> {
    /// Get a FuzzerBuilder
    /// Equivalent to `FuzzerBuilder::new()`
    pub fn builder() -> FuzzerBuilder<'a> {
        FuzzerBuilder::new()
    }

    /// Create a crawler, consuming a FuzzerBuilder
    /// Equivalent to `FuzzerBuilder.build()`
    pub fn from_builder(builder: FuzzerBuilder<'a>) -> Result<Self> {
        Ok(Self {
            handlers: builder.handlers,
            workers: builder.workers,
            client: Arc::new(builder.client_builder.build()?),
        })
    }

    pub async fn fuzz_post<T: ToString>(
        &mut self,
        urls: &mut impl Iterator<Item = T>,
        data: &mut impl Iterator<Item = T>,
    ) -> Result<Vec<anyhow::Error>> {
        let mut iter = urls.zip(data);

        let mut errors = vec![];

        // set up async
        let (s, r) = bounded(self.workers);
        let mut tasks = 0;
        let mut empty = false;

        // Loop while the queue is not empty or tasks are fetching pages.
        while !empty || tasks > 0 {
            // Limit the number of concurrent tasks.
            while tasks < s.capacity().unwrap() {
                // Process URLs in the queue and fetch more pages.
                match iter.next() {
                    None => {
                        empty = true;
                        break;
                    }
                    Some((url, data)) => {
                        if let Ok(url) = Url::parse(&url.to_string()) {
                            tasks += 1;
                            tokio::spawn(Self::post_data(
                                url,
                                data.to_string(),
                                self.client.clone(),
                                s.clone(),
                            ));
                        }
                    }
                }
            }

            // Recieve a message
            let fetched = r.recv().await.unwrap();
            tasks -= 1;

            match fetched {
                Ok((url, res)) => {
                    self.do_handlers(&url, &res)?;
                }
                Err(err) => {
                    errors.push(err);
                }
            }
        }

        Ok(errors)
    }

    /// Request all urls in provided iterator, handling responses
    /// If a second iterator is given, POST requests will be made
    /// with data from that iterator
    pub async fn fuzz_get<T: ToString>(
        &mut self,
        urls: &mut impl Iterator<Item = T>,
    ) -> Result<Vec<anyhow::Error>> {
        let mut errors = vec![];

        // set up async
        let (s, r) = bounded(self.workers);
        let mut tasks = 0;
        let mut empty = false;

        // Loop while the queue is not empty or tasks are fetching pages.
        while !empty || tasks > 0 {
            // Limit the number of concurrent tasks.
            while tasks < s.capacity().unwrap() {
                // Process URLs in the queue and fetch more pages.
                match urls.next() {
                    None => {
                        empty = true;
                        break;
                    }
                    Some(url) => {
                        if let Ok(url) = Url::parse(&url.to_string()) {
                            tasks += 1;
                            tokio::spawn(Self::fetch(url, self.client.clone(), s.clone()));
                        }
                    }
                }
            }

            // Recieve a message
            let fetched = r.recv().await.unwrap();
            tasks -= 1;

            match fetched {
                Ok((req, res)) => {
                    self.do_handlers(&req, &res)?;
                }
                Err(err) => {
                    errors.push(err);
                }
            }
        }

        Ok(errors)
    }

    fn do_handlers(&mut self, request: &Request, response: &Response) -> Result<()> {
        for handler in self.handlers.iter_mut() {
            handler(FuzzHandlerArgs {
                request,
                response,
                client: self.client.clone(),
            });
        }
        Ok(())
    }

    /// make a request and send the results on the async chan
    async fn fetch(url: Url, client: Arc<Client>, sender: Sender<Result<(Request, Response)>>) {
        // Must send a message or die trying
        match client.get(url.clone()).build() {
            Ok(req) => match client.execute(req.try_clone().unwrap()).await {
                Ok(res) => {
                    sender.send(Ok((req, res))).await.unwrap();
                }
                Err(err) => {
                    let err = anyhow!(err);
                    sender.send(Err(err)).await.unwrap();
                }
            },
            Err(err) => {
                let err = anyhow!(err);
                sender.send(Err(err)).await.unwrap();
            }
        }
    }

    /// make a post request and send the results on the async chan
    async fn post_data(
        url: Url,
        data: String,
        client: Arc<Client>,
        sender: Sender<Result<(Request, Response)>>,
    ) {
        // Must send a message or die trying
        match client.post(url.clone()).body(data).build() {
            Ok(req) => match client.execute(req.try_clone().unwrap()).await {
                Ok(res) => {
                    sender.send(Ok((req, res))).await.unwrap();
                }
                Err(err) => {
                    let err = anyhow!(err);
                    sender.send(Err(err)).await.unwrap();
                }
            },
            Err(err) => {
                let err = anyhow!(err);
                sender.send(Err(err)).await.unwrap();
            }
        }
    }
}
