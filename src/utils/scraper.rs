use reqwest::blocking::Client;
use reqwest::header::{HeaderMap, HeaderValue, ACCEPT, ACCEPT_LANGUAGE, USER_AGENT, REFERER};

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

    let mut results = Vec::new();
    let mut seen_urls = std::collections::HashSet::new();

    // Bing embeds image data in a JSON object inside an HTML attribute: m="{&quot;murl&quot;:&quot;https://...&quot;}"
    // We can extract murl (main image url) and t (title)
    // Because it's HTML escaped, we match the encoded quotes (&quot;)
    let re_murl = regex::Regex::new(r#"&quot;murl&quot;:&quot;(https?://[^&]+)&quot;"#)
        .map_err(|e| format!("Regex error: {}", e))?;
    let re_turl = regex::Regex::new(r#"&quot;turl&quot;:&quot;(https?://[^&]+)&quot;"#)
        .map_err(|e| format!("Regex error: {}", e))?;
    let re_title = regex::Regex::new(r#"&quot;t&quot;:&quot;([^&]+)&quot;"#)
        .map_err(|e| format!("Regex error: {}", e))?;

    let imgs: Vec<_> = re_murl.captures_iter(&html).map(|c| c[1].to_string()).collect();
    let thumbs: Vec<_> = re_turl.captures_iter(&html).map(|c| c[1].to_string()).collect();
    let titles: Vec<_> = re_title.captures_iter(&html).map(|c| c[1].to_string()).collect();

    for i in 0..imgs.len() {
        if i >= 20 { break; }
        
        let mut img_url = imgs[i].clone();
        img_url = img_url.replace("\\/", "/");
        
        let mut thumb_url = thumbs.get(i).unwrap_or(&img_url).clone();
        thumb_url = thumb_url.replace("\\/", "/").replace("&amp;", "&");

        if seen_urls.contains(&img_url) { continue; }
        seen_urls.insert(img_url.clone());

        let mut title = titles.get(i).unwrap_or(&"Image".to_string()).clone();
        title = title.replace("\\/", "/");

        results.push(ScrapedImage {
            url: img_url.clone(),
            thumbnail: thumb_url,
            title,
        });
    }

    // Filter to prioritize liquipedia, wikipedia, and fandom
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

    if !wiki_results.is_empty() {
        Ok(wiki_results)
    } else {
        Err("No Wikipedia or Liquipedia images found. Try a different query.".to_string())
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
