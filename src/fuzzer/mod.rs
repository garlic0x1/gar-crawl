pub mod fuzzer;
pub mod fuzzer_builder;
pub mod handler;

pub use fuzzer::*;
pub use fuzzer_builder::*;
pub use handler::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn fuzz_test() {
        let urls = "https://google.com\nhttps://microsoft.com\nhttps://apple.com\nhttps://notrealwebsiteithink.com";
        let mut iter = urls.lines();
        let mut responses = 0;
        let errs = Fuzzer::builder()
            .add_handler(|args| {
                responses += 1;
                println!("{}", args.url.as_str());
                println!("{:?}", args.response.status())
            })
            .build()
            .unwrap()
            .fuzz_get(&mut iter)
            .await
            .unwrap();

        println!("{:?}", errs);
        assert_eq!(responses, 3);
        assert_eq!(errs.len(), 1);
    }
}
