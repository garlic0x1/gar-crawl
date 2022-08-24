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

This example will crawl from https://example.org to a depth of 3, revisiting urls and printing all links found  
```rust
Crawler::builder()
    .add_default_propagators()                         // crawl to href and src links
    .revisit(true)                                     // default false
    .whitelist("https://example.org")                  // stay on this site
    .user_agent("Mozilla/5.0 (X11; Linux x86_64)...")  // set user agent
    .proxy("127.0.0.1:8080", "/path/to/cacert.der")    // set up https proxy
    .add_handler("*[href]", |args| {                   // add handler
        if let Some(href) = args.element.unwrap().value().attr("href") {
            println!("{href}");
        }
    })
    .depth(3)                                          // default 2
    .build()?                                          // construct crawler
    .crawl("https://example.org")                      // begin crawl
    .await?;
```  

See `examples/` for more examples
