# gar-crawl
A high level HTML crawler with a concise builder.  

The goal of this crate is to accomplish crawling and scraping tasks with minimal boilerplate.
Default propagators are provided or you can make your own, and you can modify the Reqwest client used
before building a crawler.

# examples
This example will crawl from https://example.org to a depth of 3, printing all links found  
```rust
Crawler::builder()
    .add_default_propagators()
    .user_agent("Mozilla/5.0 (X11; Linux x86_64)...")
    .add_handler("*[href]", |args| {
        if let Some(href) = args.element.unwrap().value().attr("href") {
            println!("{href}");
        }
    })
    .depth(3)
    .build()?
    .crawl("https://example.org")
    .await?;
```  

See `examples/` for more examples
