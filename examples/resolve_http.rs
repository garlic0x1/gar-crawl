use anyhow::Result;
use gar_crawl::fuzzer::*;

#[tokio::main]
async fn main() -> Result<()> {
    let errs = Fuzzer::builder()
        .add_handler(|args| {
            println!(
                "status: {} url: {}",
                args.response.status(),
                args.url.as_str()
            )
        })
        .build()?
        .fuzz_get(&mut std::io::stdin().lines().flatten())
        .await;

    println!("Errors: {:?}", errs);
    Ok(())
}
