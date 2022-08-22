pub mod crawler;
pub mod crawler_builder;

#[cfg(test)]
mod tests {
    use crate::crawler::*;
    use std::collections::HashSet;

    use reqwest::Url;
    use scraper::ElementRef;

    #[tokio::test]
    async fn it_works() {
        let mut seen: HashSet<String> = HashSet::new();

        Crawler::builder()
        .add_default_propagators()
        .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.5112.79 Safari/537.36".into())
        .add_handler("*[href]", move |el: ElementRef, url: Url| {
            if let Some(href) = el.value().attr("href") {
                if let Ok(abs_url) = url.join(href) {
                    seen.insert(abs_url.to_string());
                } else {
                    seen.insert(href.to_string());
                }
            }
        })
        .depth(1)
        .build().unwrap()
        .crawl("https://vim.org/weird.php")
        .await.unwrap();

        assert_eq!(seen.len(), 32);
    }
}
