pub mod crawler;

#[cfg(test)]
mod tests {
    use super::crawler::*;
    use reqwest::Url;
    use scraper::ElementRef;
    use std::collections::HashSet;

    #[tokio::test(flavor = "multi_thread", worker_threads = 2)]
    async fn it_works() {
        let mut c = 0;
        Crawler::builder()
            .add_default_propagators()
            .whitelist("qiwi-button")
            .user_agent("Mozilla/5.0 (X11; Linux x86_64)...".into())
            .add_handler("*[href]", move |_el: ElementRef, _page: &Page| {
                let counter = &mut c;
                // println!("incrementing counter");
                counter += 1;
                // c = *counter;
            })
            .depth(1)
            .build()
            .unwrap()
            .crawl("http://plugins.svn.wordpress.org/qiwi-button")
            .await
            .unwrap();

        assert_eq!(c, 32);
    }
}
