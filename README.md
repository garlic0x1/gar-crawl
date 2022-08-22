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
    .whitelist(&args.url)
    .user_agent("Mozilla/5.0 (X11; Linux x86_64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.5112.79 Safari/537.36".into())
    .add_handler("*[href]", move |el: ElementRef, page: &Page| {
        if let Some(href) = el.value().attr("href") {
            println!("{href}");
        }
    })
    .depth(args.depth)
    .build()?
    .crawl(&args.url)
    .await?;
```  

See `examples/` for more examples
