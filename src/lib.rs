pub mod auxiliary;
pub mod crawler;
pub mod fuzzer;

pub use auxiliary::absolute_url;

#[cfg(test)]
mod tests {
    use reqwest::Url;

    use crate::absolute_url;

    #[tokio::test]
    async fn abs_url() {
        let base_url = Url::parse("https://github.com/garlic0x1/").unwrap();
        let href = "gar-crawl/";
        let abs_url = absolute_url(&base_url, href).unwrap();
        assert_eq!(abs_url.as_str(), "https://github.com/garlic0x1/gar-crawl/");

        let base_url = Url::parse("https://github.com/garlic0x1").unwrap();
        let href = "gar-crawl/";
        let abs_url = absolute_url(&base_url, href).unwrap();
        assert_eq!(abs_url.as_str(), "https://github.com/gar-crawl/");

        let base_url = Url::parse("https://github.com/garlic0x1/").unwrap();
        let href = "https://google.com";
        let abs_url = absolute_url(&base_url, href).unwrap();
        assert_eq!(abs_url.as_str(), "https://google.com/");
    }
}
