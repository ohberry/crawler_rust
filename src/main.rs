use axum::{
    extract::Form,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use regex::Regex;
use serde::Deserialize;
use validator::Validate;

#[tokio::main]
async fn main() {
    // build our application with a single route
    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/hook", post(hook));

    // run our app with hyper, listening globally on port 3000
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
}

#[derive(Debug,Deserialize,Validate)]
struct Monitor {
    #[validate(length(min = 1))]
    link: String,
}

async fn hook(Form(monitor): Form<Monitor>){
    let url = monitor.link;
    if url.contains("douyin.com"){
        let r = Regex::new(r"(douyin\.com/user/)(?<author_id>[\w|-]+)").unwrap();
        let caps = r.captures(&url).unwrap();
        let author_id = &caps["author_id"];
        handle_douyin(author_id).await;
    }
}

async fn handle_douyin(author_id: &str){
    //has_more 代表是否还有更多，1代表有，0代表没有
    let has_more = 1;
    while has_more == 1 {
        
    }
}
