pub mod crawler;
pub mod crawler_builder;

#[cfg(test)]
mod tests {
    // use super::crawler::*;
    // use async_channel::*;
    // use reqwest::Url;
    // use scraper::ElementRef;

    #[tokio::test]
    async fn it_works() {

        // let crawler = Crawler::builder()
        //     .add_handler("*[href]".into(), |el: ElementRef, url: Url| {
        //     println!("{:?}", el.value().attr("href"));
        //     })
        //     .add_default_propagators()
        //     .build()
        //     .unwrap();
        // crawler
        //     .crawl("https://www.vim.org/about.php")
        //     .await
        //     .unwrap();

        // assert_eq!(false, true);
    }
}
