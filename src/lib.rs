pub mod crawler;
// pub mod worker;
// pub mod worker_pool;

#[cfg(test)]
mod tests {
    use super::crawler::*;
    use scraper::ElementRef;

    #[tokio::test]
    async fn it_works() {
        let mut crawler = Crawler::new();
        crawler.add_handler(
            "a[href]",
            Box::new(|el: ElementRef| {
                println!("{:?}", el.value());
            }),
        );
        crawler.crawl("https://www.google.com").await.unwrap();

        assert_eq!(false, true);
    }
}
