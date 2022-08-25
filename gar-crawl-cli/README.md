# gar-crawl-cli
Command line interface for `gar-crawl`.

# help
```
gar-crawl-cli 0.1.0

USAGE:
    gar-crawl-cli [OPTIONS]

OPTIONS:
    -c, --confine              confine crawl inside given path ( alias of whitelist(url) )
    -d, --depth <DEPTH>        crawl depth [default: 2]
    -h, --help                 Print help information
    -r, --revisit              revisit urls
    -t, --timeout <TIMEOUT>    request timeout ( seconds ) [default: 10]
    -u, --url <URL>            start url ( will read lines from stdin if not provided as a flag )
    -v, --verbose              verbose output
    -V, --version              Print version information
    -w, --workers <WORKERS>    concurrency limit [default: 40]
```