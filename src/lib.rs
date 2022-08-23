pub mod crawler;

#[cfg(test)]
mod tests {
    use super::crawler::*;
    use scraper::ElementRef;

    #[tokio::test]
    async fn crawl_test() {
        let mut c = 0;
        Crawler::builder()
            .add_default_propagators()
            .whitelist("qiwi-button")
            .user_agent("Mozilla/5.0 (X11; Linux x86_64)...".into())
            .add_handler("*[href]", |_el: ElementRef, _page: &Page| {
                c += 1;
            })
            .depth(1)
            .build()
            .unwrap()
            .crawl("http://plugins.svn.wordpress.org/qiwi-button")
            .await
            .unwrap();

        assert_eq!(c, 5);
    }
}
