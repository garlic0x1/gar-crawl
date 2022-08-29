pub mod crawler;
pub mod crawler_builder;
pub mod handler;

pub use crawler::*;
pub use crawler_builder::*;
pub use handler::*;

#[cfg(test)]
mod tests {
    use super::*;
    use std::collections::HashSet;

    #[tokio::test]
    async fn crawl_test() {
        let mut visited = HashSet::new();
        let mut links = HashSet::new();
        let mut pages_loaded = 0;

        let errs = Crawler::builder()
            .add_default_propagators()
            .whitelist("qiwi-button")
            //.revisit(true)
            .user_agent("Mozilla/5.0 (X11; Linux x86_64)...")
            .add_handler("*[href]", |args| {
                if let Some(link) = args.element.unwrap().value().attr("href") {
                    links.insert(link.to_string());
                }
            })
            .on_page(|args| {
                pages_loaded += 1;
                let ustr = args.page.url.to_string();
                if ustr.ends_with(".php") {
                    visited.insert(ustr);
                }
            })
            .depth(3)
            .build()
            .unwrap()
            .crawl("http://plugins.svn.wordpress.org/qiwi-button/")
            .await
            .unwrap();

        // show what went wrong
        println!("{:?}", visited);
        println!("{:?}", links);
        println!("{:?}", errs);

        assert_eq!(errs.len(), 0);
        assert_eq!(pages_loaded, 69);
        assert_eq!(visited.len(), 18);
        assert_eq!(links.len(), 61);
    }
}
