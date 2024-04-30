use axum::http::HeaderMap;
use axum::{
    extract::Form,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use dy_xhs::generate_xb;
use ini::Ini;
use regex::Regex;
use reqwest::ClientBuilder;
use serde::Deserialize;
use std::fs;
use std::path::Path;
use validator::Validate;

#[tokio::main]
async fn main() {
    let conf = Ini::load_from_file("conf.ini").unwrap();
    // let dy_download_dir = conf.general_section().get("dyDownloadDir").unwrap();
    let dy_download_dir = conf.get_from(None::<String>, "dyDownloadDir").unwrap();
    // let contents = fs::read_to_string(Path::new(dy_download_dir).join("poem.txt"))
    //     .expect("读取配置文件发生错误");

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/hook", post(hook));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug, Deserialize, Validate)]
struct Monitor {
    #[validate(length(min = 1))]
    link: String,
}

async fn hook(Form(monitor): Form<Monitor>) {
    let url = monitor.link;
    if url.contains("douyin.com") {
        let r = Regex::new(r"(douyin\.com/user/)(?<author_id>[\w|-]+)").unwrap();
        let caps = r.captures(&url).unwrap();
        let author_id = &caps["author_id"];
        handle_douyin(author_id, 0).await;
    }
}

async fn handle_douyin(author_id: &str, max_cursor: u64) {
    //has_more 代表是否还有更多，1代表有，0代表没有
    let has_more = 1;
    while has_more == 1 {
        let params = generate_xb(format!(
            "aid=6383&sec_user_id={author_id}&count=20&max_cursor={max_cursor}&cookie_enabled=true&platform=PC&downlink=10"
        ).as_str());
        let builder = ClientBuilder::new();
        let mut map = HeaderMap::new();
        map.insert("Referer", "https://www.douyin.com/".parse().unwrap());
        map.insert("User-Agent", ("Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36".parse().unwrap()));
        // let resp = reqwest::get("https://httpbin.org/ip")
        //     .await?
        //     .json::<HashMap<String, String>>()
        //     .await?;
        // println!("{resp:#?}");
    }
}
