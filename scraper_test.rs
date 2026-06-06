fn main() {
    let client = reqwest::blocking::Client::builder()
        .user_agent("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/125.0.0.0 Safari/537.36")
        .timeout(std::time::Duration::from_secs(15))
        .build()
        .unwrap();

    let search_query = "RRQ liquipedia png logo";
    let url = format!(
        "https://www.bing.com/images/search?q={}&form=HDRSC2&first=1",
        urlencoding::encode(&search_query)
    );

    let html = client
        .get(&url)
        .header("Accept-Language", "en-US,en;q=0.9")
        .send()
        .unwrap()
        .text()
        .unwrap();

    let re_m = regex::Regex::new(r#"m="(\{[^"]+\})""#).unwrap();
    let re_murl = regex::Regex::new(r#""murl":"([^"]+)""#).unwrap();
    let re_turl = regex::Regex::new(r#""turl":"([^"]+)""#).unwrap();
    
    for cap in re_m.captures_iter(&html).take(5) {
        let json_str = cap[1].replace("&quot;", "\"").replace("&amp;", "&");
        println!("JSON: {}", json_str);
        if let Some(murl) = re_murl.captures(&json_str) {
            println!("MURL: {}", murl[1].to_string());
        }
        if let Some(turl) = re_turl.captures(&json_str) {
            println!("TURL: {}", turl[1].to_string());
        }
    }
}
