use axum::http::header::CONTENT_TYPE;
use axum::extract::Path;
use axum::http::{HeaderMap, HeaderValue};

pub async fn static_route(path: Path<String>) -> Result<(HeaderMap, Vec<u8>), axum::http::StatusCode> {
    let content = match path.as_str() {
        "favicon.png" => include_bytes!("../static/favicon.png").to_vec(),
        "index.js" => include_str!("../static/index.js").as_bytes().to_vec(),
        "admin.js" => include_str!("../static/admin.js").as_bytes().to_vec(),
        "logo.png" => include_bytes!("../static/logo.png").to_vec(),
        _ => return Err(axum::http::StatusCode::NOT_FOUND),
    };

    let content_type = match path.as_str() {
        "favicon.png" | "logo.png" => "image/png",
        "index.js" | "admin.js" => "text/javascript",
        _ => "application/octet-stream",
    };

    let hm = HeaderMap::from_iter(vec![(
        CONTENT_TYPE,
        HeaderValue::from_str(content_type).unwrap(),
    )]);

    Ok((hm, content))
}
