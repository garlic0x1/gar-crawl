pub mod crawler;

#[cfg(test)]
mod tests {
    use std::collections::HashSet;

    use super::crawler::*;

    #[tokio::test]
    async fn crawl_test() {
        let mut seen = HashSet::new();
        let errs = Crawler::builder()
            .add_default_propagators()
            .whitelist("qiwi-button")
            .user_agent("Mozilla/5.0 (X11; Linux x86_64)...".into())
            .on_page(|page: &Page| {
                let ustr = page.url.to_string();
                if ustr.ends_with(".php") {
                    seen.insert(ustr);
                }
            })
            .depth(3)
            .build()
            .unwrap()
            .crawl("http://plugins.svn.wordpress.org/qiwi-button/")
            .await
            .unwrap();

        println!("{:?}", seen);
        println!("{:?}", errs);
        assert_eq!(seen.len(), 18);
    }
}
