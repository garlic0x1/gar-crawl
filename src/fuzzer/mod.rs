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
        let urls = "https://google.com\nhttps://microsoft.com\nhttps://apple.com";
        let mut iter = urls.lines();
        let errs = Fuzzer::builder()
            .add_handler(|args| {
                println!("{}", args.url.as_str());
            })
            .build()
            .unwrap()
            .fuzz(&mut iter)
            .await
            .unwrap();

        assert!(false);
    }
}
