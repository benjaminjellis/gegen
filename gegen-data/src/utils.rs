use reqwest::header::HeaderMap;

pub(crate) fn create_header_maps() -> HeaderMap {
    let mut headers = HeaderMap::new();
    headers.insert("Accept", "application/json".parse().unwrap());
    headers.insert("Accept-Encoding", "".parse().unwrap());

    headers.insert("Connection", "keep-alive".parse().unwrap());
    headers.insert("Host", "optaplayerstats.statsperform.com".parse().unwrap());
    headers.insert("Cache-Control", "no-cache".parse().unwrap());

    headers.insert(
        "Referer",
        "https://optaplayerstats.statsperform.com/ro_RO/soccer"
            .parse()
            .unwrap(),
    );

    headers.insert(
        "User-Agent",
        "Mozilla/5.0 (Macintosh; Intel Mac OS X 10.15; rv:137.0) Gecko/20100101 Firefox/137.0"
            .parse()
            .unwrap(),
    );
    headers
}
