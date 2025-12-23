use axum::{
    Router,
    extract::{Path, Query},
    http::{HeaderMap, HeaderValue, StatusCode, header},
    response::{IntoResponse, Response},
    routing::get,
};
use resvg::tiny_skia::{Pixmap, Transform};
use resvg::usvg::{Options, Tree};
use std::collections::HashMap;

const GRAVATAR_URL: &str = "https://www.gravatar.com/avatar";
// todo: don't fetch this but calculate locally
const BORING_AVATARS_URL: &str = "https://boring-avatars-api.vercel.app/api/avatar";
const DEFAULT_SIZE: u32 = 80;
const MAX_SIZE: u32 = 512;

#[derive(Clone, Copy)]
enum Style {
    Gravatar(GravatarStyle),
    Boring(BoringVariant),
}

#[derive(Clone, Copy)]
enum GravatarStyle {
    NotFound,
    Mp,
    Identicon,
    Monsterid,
    Wavatar,
    Retro,
    Robohash,
    Blank,
}

impl GravatarStyle {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "404" => Some(Self::NotFound),
            "mp" => Some(Self::Mp),
            "identicon" => Some(Self::Identicon),
            "monsterid" => Some(Self::Monsterid),
            "wavatar" => Some(Self::Wavatar),
            "retro" => Some(Self::Retro),
            "robohash" => Some(Self::Robohash),
            "blank" => Some(Self::Blank),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::NotFound => "404",
            Self::Mp => "mp",
            Self::Identicon => "identicon",
            Self::Monsterid => "monsterid",
            Self::Wavatar => "wavatar",
            Self::Retro => "retro",
            Self::Robohash => "robohash",
            Self::Blank => "blank",
        }
    }
}

#[derive(Clone, Copy)]
enum BoringVariant {
    Marble,
    Beam,
    Pixel,
    Sunset,
    Ring,
    Bauhaus,
}

impl BoringVariant {
    fn from_str(s: &str) -> Option<Self> {
        match s {
            "marble" => Some(Self::Marble),
            "beam" => Some(Self::Beam),
            "pixel" => Some(Self::Pixel),
            "sunset" => Some(Self::Sunset),
            "ring" => Some(Self::Ring),
            "bauhaus" => Some(Self::Bauhaus),
            _ => None,
        }
    }

    fn as_str(self) -> &'static str {
        match self {
            Self::Marble => "marble",
            Self::Beam => "beam",
            Self::Pixel => "pixel",
            Self::Sunset => "sunset",
            Self::Ring => "ring",
            Self::Bauhaus => "bauhaus",
        }
    }
}

impl Style {
    fn from_str(s: &str) -> Option<Self> {
        if let Some(g) = GravatarStyle::from_str(s) {
            Some(Self::Gravatar(g))
        } else {
            BoringVariant::from_str(s).map(Self::Boring)
        }
    }
}

#[tokio::main]
async fn main() {
    let app = Router::new().route("/avatar/{hash}", get(avatar_handler));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:8000").await.unwrap();
    println!("Listening on http://0.0.0.0:8000");
    axum::serve(listener, app).await.unwrap();
}

async fn avatar_handler(
    Path(hash): Path<String>,
    Query(params): Query<HashMap<String, String>>,
) -> Response {
    let style_str = params.get("d").map(|s| s.as_str()).unwrap_or("identicon");
    let size = params
        .get("s")
        .and_then(|s| s.parse::<u32>().ok())
        .unwrap_or(DEFAULT_SIZE)
        .min(MAX_SIZE);

    let style = match Style::from_str(style_str) {
        Some(s) => s,
        None => {
            return (
                StatusCode::BAD_REQUEST,
                format!("Unknown style: {}", style_str),
            )
                .into_response();
        }
    };

    let client = reqwest::Client::new();

    match style {
        Style::Gravatar(g) => {
            let url = format!("{GRAVATAR_URL}/{hash}?d={}&s={}", g.as_str(), size);
            fetch_and_relay(&client, &url).await
        }
        Style::Boring(b) => {
            // First check if user has a gravatar
            let gravatar_url = format!("{GRAVATAR_URL}/{hash}?d=404&s={size}");

            match client.get(&gravatar_url).send().await {
                Ok(resp) if resp.status().is_success() => {
                    // User has a gravatar, pass it through
                    let content_type = resp
                        .headers()
                        .get(header::CONTENT_TYPE)
                        .and_then(|v| v.to_str().ok())
                        .unwrap_or("image/png")
                        .to_string();

                    match resp.bytes().await {
                        Ok(bytes) => {
                            let mut headers = HeaderMap::new();
                            if let Ok(ct) = HeaderValue::from_str(&content_type) {
                                headers.insert(header::CONTENT_TYPE, ct);
                            }
                            (StatusCode::OK, headers, bytes.to_vec()).into_response()
                        }
                        Err(_) => StatusCode::BAD_GATEWAY.into_response(),
                    }
                }
                Ok(resp) if resp.status() == StatusCode::NOT_FOUND => {
                    // No gravatar, use boring avatar
                    fetch_boring_avatar(&client, &hash, b, size).await
                }
                Ok(resp) => {
                    eprintln!(
                        "Gravatar returned unexpected status {} for hash {}",
                        resp.status(),
                        hash
                    );
                    StatusCode::BAD_GATEWAY.into_response()
                }
                Err(e) => {
                    eprintln!("Failed to fetch from Gravatar for hash {}: {}", hash, e);
                    StatusCode::BAD_GATEWAY.into_response()
                }
            }
        }
    }
}

async fn fetch_boring_avatar(
    client: &reqwest::Client,
    hash: &str,
    variant: BoringVariant,
    size: u32,
) -> Response {
    // Fetch SVG at a larger size for better quality when scaling
    let svg_size = size.max(128);
    let url = format!(
        "{}?name={}&size={}&variant={}",
        BORING_AVATARS_URL,
        hash,
        svg_size,
        variant.as_str()
    );

    let svg_data = match client.get(&url).send().await {
        Ok(resp) if resp.status().is_success() => match resp.bytes().await {
            Ok(bytes) => bytes,
            Err(e) => {
                eprintln!("Failed to read SVG response: {}", e);
                return StatusCode::BAD_GATEWAY.into_response();
            }
        },
        Ok(resp) => {
            eprintln!("Boring Avatars returned status {}", resp.status());
            return StatusCode::BAD_GATEWAY.into_response();
        }
        Err(e) => {
            eprintln!("Failed to fetch from Boring Avatars: {}", e);
            return StatusCode::BAD_GATEWAY.into_response();
        }
    };

    // Convert SVG to PNG
    match svg_to_png(&svg_data, size) {
        Ok(png_data) => {
            let mut headers = HeaderMap::new();
            headers.insert(header::CONTENT_TYPE, HeaderValue::from_static("image/png"));
            (StatusCode::OK, headers, png_data).into_response()
        }
        Err(e) => {
            eprintln!("Failed to convert SVG to PNG: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR.into_response()
        }
    }
}

fn svg_to_png(svg_data: &[u8], size: u32) -> Result<Vec<u8>, String> {
    let tree = Tree::from_data(svg_data, &Options::default()).map_err(|e| e.to_string())?;

    let mut pixmap = Pixmap::new(size, size).ok_or("Failed to create pixmap")?;

    let svg_size = tree.size();
    let scale_x = size as f32 / svg_size.width();
    let scale_y = size as f32 / svg_size.height();
    let scale = scale_x.min(scale_y);

    let transform = Transform::from_scale(scale, scale);

    resvg::render(&tree, transform, &mut pixmap.as_mut());

    pixmap.encode_png().map_err(|e| e.to_string())
}

async fn fetch_and_relay(client: &reqwest::Client, url: &str) -> Response {
    match client.get(url).send().await {
        Ok(resp) if resp.status().is_success() => {
            let content_type = resp
                .headers()
                .get(header::CONTENT_TYPE)
                .and_then(|v| v.to_str().ok())
                .unwrap_or("image/png")
                .to_string();

            match resp.bytes().await {
                Ok(bytes) => {
                    let mut headers = HeaderMap::new();
                    if let Ok(ct) = HeaderValue::from_str(&content_type) {
                        headers.insert(header::CONTENT_TYPE, ct);
                    }
                    (StatusCode::OK, headers, bytes.to_vec()).into_response()
                }
                Err(e) => {
                    eprintln!("Failed to read response body from {}: {}", url, e);
                    StatusCode::BAD_GATEWAY.into_response()
                }
            }
        }
        Ok(resp) => {
            eprintln!("Upstream returned status {} for {}", resp.status(), url);
            StatusCode::BAD_GATEWAY.into_response()
        }
        Err(e) => {
            eprintln!("Failed to fetch {}: {}", url, e);
            StatusCode::BAD_GATEWAY.into_response()
        }
    }
}
