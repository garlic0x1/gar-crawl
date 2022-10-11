use anyhow::{anyhow, Result};
use async_channel::*;
use reqwest::{Client, Url};
use std::sync::Arc;

/// make a request and send the results on the async chan
pub async fn fetch(
    url: Url,
    depth: usize,
    client: Arc<Client>,
    sender: Sender<Result<(Url, String, usize)>>,
) -> Result<()> {
    // Must send a message or die trying
    match client.get(url.clone()).send().await {
        Ok(res) => match res.text().await {
            Ok(text) => {
                sender.send(Ok((url, text, depth))).await.unwrap();
                Ok(())
            }
            Err(err) => {
                let err = anyhow!(err);
                sender.send(Err(err)).await.unwrap();
                Err(anyhow!("Failed read"))
            }
        },
        Err(err) => {
            let err = anyhow!(err);
            sender.send(Err(err)).await.unwrap();
            Err(anyhow!("Failed request"))
        }
    }
}
