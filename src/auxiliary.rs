use anyhow::{bail, Result};
use reqwest::Url;
use std::collections::HashSet;

/// absolute url relative to base
pub fn absolute_url(base_url: &Url, href: &str) -> Result<Url> {
    if let Ok(abs_url) = Url::parse(href) {
        Ok(abs_url)
    } else if let Ok(abs_url) = base_url.join(href) {
        Ok(abs_url)
    } else {
        bail!("Invalid link: {href}")
    }
}

/// match whitelist/blacklist rules
pub fn is_allowed(url: &Url, wl: &Vec<String>, bl: &Vec<String>) -> bool {
    let surl = url.to_string();
    if wl
        .iter()
        .filter(|expr| surl.contains(expr.as_str()))
        .take(1)
        .count()
        == 0
        && wl.len() > 0
    {
        false
    } else if bl
        .iter()
        .filter(|expr| surl.contains(expr.as_str()))
        .take(1)
        .count()
        != 0
    {
        false
    } else {
        true
    }
}

/// see if this url has been visited yet
pub fn is_visited(url: &Url, visited: &mut HashSet<String>) -> bool {
    let surl = url.to_string();
    !visited.insert(surl)
}
