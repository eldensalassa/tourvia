#![allow(dead_code)]

use reqwest::blocking::Client;

#[derive(Clone, Debug)]
pub struct ScrapedImage {
    pub url: String,
    pub thumbnail: String,
    pub title: String,
}

/// Scrape image results from Bing Images (lightweight, doesn't block as aggressively as Google).
pub fn fetch_images_bing(query: &str) -> Result<Vec<ScrapedImage>, String> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .map_err(|e| format!("HTTP client error: {}", e))?;

    let mut search_query = query.to_string();
    if !search_query.to_lowercase().contains("png") {
        search_query.push_str(" png logo");
    }

    let url = format!(
        "https://www.bing.com/images/search?q={}&form=HDRSC2&first=1",
        urlencoding::encode(&search_query)
    );

    let html = client
        .get(&url)
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .map_err(|e| format!("Failed to reach Bing: {}", e))?
        .text()
        .map_err(|e| format!("Failed to read response: {}", e))?;

    // Bing embeds image data in a JSON object inside an HTML attribute: m="{&quot;murl&quot;:&quot;https://...&quot;}"
    let re_m = regex::Regex::new(r#"m="([^"]+)""#).map_err(|e| format!("Regex error: {}", e))?;
    let re_murl = regex::Regex::new(r#"&quot;murl&quot;:&quot;(https?://[^&]+)&quot;"#).map_err(|e| format!("Regex error: {}", e))?;
    let re_turl = regex::Regex::new(r#"&quot;turl&quot;:&quot;(https?://[^&]+)&quot;"#).map_err(|e| format!("Regex error: {}", e))?;
    let re_title = regex::Regex::new(r#"&quot;t&quot;:&quot;([^&]+)&quot;"#).map_err(|e| format!("Regex error: {}", e))?;

    let mut results: Vec<ScrapedImage> = Vec::new();
    let mut seen_urls: std::collections::HashSet<String> = std::collections::HashSet::new();

    for cap in re_m.captures_iter(&html) {
        let m_attr = &cap[1];
        
        let murl = match re_murl.captures(m_attr) {
            Some(c) => c[1].to_string(),
            None => continue,
        };
        
        let turl = match re_turl.captures(m_attr) {
            Some(c) => c[1].to_string(),
            None => murl.clone(),
        };
        
        let title = match re_title.captures(m_attr) {
            Some(c) => c[1].to_string(),
            None => "Image".to_string(),
        };
        
        let img_url = murl.replace("\\/", "/");
        let thumb_url = turl.replace("\\/", "/").replace("&amp;", "&");
        let title_clean = title.replace("\\/", "/");
        
        if seen_urls.contains(&img_url) { continue; }
        seen_urls.insert(img_url.clone());
        
        results.push(ScrapedImage {
            url: img_url,
            thumbnail: thumb_url,
            title: title_clean,
        });
        
        if results.len() >= 20 { break; }
    }

    // Sort to prioritize liquipedia, wikipedia, and fandom
    let mut wiki_results: Vec<ScrapedImage> = Vec::new();
    let mut other_results: Vec<ScrapedImage> = Vec::new();

    for r in results {
        let is_wiki = r.url.contains("liquipedia.net") 
            || r.url.contains("wikipedia.org") 
            || r.url.contains("wikimedia.org")
            || r.url.contains("fandom.com");
            
        if is_wiki {
            wiki_results.push(r);
        } else {
            other_results.push(r);
        }
    }

    wiki_results.extend(other_results);

    if !wiki_results.is_empty() {
        Ok(wiki_results)
    } else {
        Err("No images found. Try a different query.".to_string())
    }
}

/// Download an image from a URL and return the raw bytes.
pub fn download_image(url: &str) -> Result<Vec<u8>, String> {
    let client = Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(20))
        .build()
        .map_err(|e| format!("HTTP Client error: {}", e))?;

    let bytes = client
        .get(url)
        .send()
        .map_err(|e| format!("Failed to download image: {}", e))?
        .bytes()
        .map_err(|e| format!("Failed to read image bytes: {}", e))?;

    if bytes.len() < 100 {
        return Err("Downloaded file is too small — likely not a valid image.".to_string());
    }

    Ok(bytes.to_vec())
}
