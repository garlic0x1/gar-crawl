# gar-crawl
A high level HTML crawler with a concise builder.  

The goal of this crate is to accomplish crawling and scraping tasks with minimal boilerplate.
Default propagators are provided or you can make your own, and you can modify the Reqwest client used
before building a crawler.

# examples
Basic usage with default options ( crawl depth: 2, workers: 40, revisit: false )  
```rust
Crawler::builder()
    .add_default_propagators()                         // crawl to href and src links
    .add_handler("*[href]", |args| {                   // add handler
        if let Some(href) = args.element.unwrap().value().attr("href") {
            println!("{href}");
        }
    })
    .build()?                                          // construct crawler
    .crawl("https://example.org")                      // begin crawl
    .await?;
```  

Example with more features used  
```rust
Crawler::builder()
    .add_default_propagators()                         // crawl to href and src links
    .revisit(true)                                     // default false
    .whitelist("https://example.org")                  // stay on this site
    .user_agent("Mozilla/5.0 (X11; Linux x86_64)...")  // set user agent
    .proxy("127.0.0.1:8080", "/path/to/cacert.der")?   // set up https proxy
    .add_handler("*[href]", |args| {                   // add handler
        if let Some(href) = args.element.unwrap().value().attr("href") {
            println!("{href}");
        }
    })
    .on_page(|args| {
        // do stuff with page
    })
    .depth(3)                                          // default 2
    .workers(100)                                      // default 40
    .timeout(5, 0)                                     // timeout requests after 5 seconds
    .build()?                                          // construct crawler
    .crawl("https://example.org")                      // begin crawl
    .await?;
```  

See `examples/` or `gar-crawl-cli/` for more examples
