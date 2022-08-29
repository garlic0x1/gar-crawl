pub mod auxiliary;
pub mod crawler;
pub mod fuzzer;

pub use auxiliary::absolute_url;

#[cfg(test)]
mod tests {

    #[tokio::test]
    async fn crawl_test() {}
}
