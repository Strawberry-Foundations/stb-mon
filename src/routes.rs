use axum::http::header::CONTENT_TYPE;
use axum::http::{HeaderMap, HeaderValue};

pub async fn favicon_route() -> (HeaderMap, Vec<u8>) {
    let hm = HeaderMap::from_iter(vec![(
        CONTENT_TYPE,
        HeaderValue::from_str("image/png").unwrap(),
    )]);
    let img = include_bytes!("../static/favicon.ico").to_vec();
    (hm, img)
}

pub async fn indexjs_route() -> (HeaderMap, String) {
    let hm = HeaderMap::from_iter(vec![(
        CONTENT_TYPE,
        HeaderValue::from_str("text/javascript").unwrap(),
    )]);
    let script = include_str!("../static/index.js").to_string();

    (hm, script)
}

pub async fn adminjs_route() -> (HeaderMap, String) {
    let hm = HeaderMap::from_iter(vec![(
        CONTENT_TYPE,
        HeaderValue::from_str("text/javascript").unwrap(),
    )]);
    let script = include_str!("../static/admin.js").to_string();
    
    (hm, script)
}
