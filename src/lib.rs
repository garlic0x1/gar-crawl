pub mod crawler;
// pub mod worker;
// pub mod worker_pool;

#[cfg(test)]
mod tests {
    use super::crawler::*;
    use reqwest::Url;
    use scraper::ElementRef;

    #[tokio::test]
    async fn it_works() {
        let crawler = Crawler::new()
            .add_handler("*[href]".into(), |el: ElementRef, url: Url| {
                println!("{:?}", el.value().attr("href"));
            })
            .add_default_propagators();
        crawler.crawl("https://www.google.com").await.unwrap();

        assert_eq!(false, true);
    }
}
