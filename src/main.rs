use axum::http::HeaderMap;
use axum::Extension;
use axum::{
    extract::Form,
    http::StatusCode,
    response::IntoResponse,
    routing::{get, post},
    Json, Router,
};
use chrono::{TimeZone, Utc};
use dy_xhs::generate_xb;
use ini::Ini;
use mobc::{Connection, Pool};
use mobc_redis::redis::AsyncCommands;
use mobc_redis::RedisConnectionManager;
use regex::Regex;
use reqwest::{Client, ClientBuilder, Error};
use serde::Deserialize;
use serde_json::Value;
use std::collections::HashMap;
use std::path::Path;
use std::process::Stdio;
use std::time::Duration;
use std::{fs, io};
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::windows::named_pipe::ClientOptions;
use tokio::process::Command;
use tokio::time;
use validator::Validate;
use windows_sys::Win32::Foundation::ERROR_PIPE_BUSY;

static mut CONFIG: Option<&mut Config> = None;
static DY_VIDEO_TYPE: [i64; 7] = [0, 4, 51, 55, 58, 61, 109];
static DY_IMG_TYPE: [i64; 3] = [2, 68, 150];

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut cmd = Command::new("std");

    // Specify that we want the command's standard output piped back to us.
    // By default, standard input/output/error will be inherited from the
    // current process (for example, this means that standard input will
    // come from the keyboard and standard output/error will go directly to
    // the terminal if this process is invoked from the command line).
    cmd.stdout(Stdio::piped());
    cmd.stdin(Stdio::piped());
    let mut child = cmd.spawn().expect("failed to spawn command");
    let mut stdin = child
        .stdin
        .take()
        .expect("child did not have a handle to stdin");

    stdin
        .write("hello".as_bytes())
        .await
        .expect("could not write to stdin");

    let stdout = child
        .stdout
        .take()
        .expect("child did not have a handle to stdout");

    drop(stdin);

    let op = child.wait_with_output().await?;
    println!("stdout: {}", String::from_utf8_lossy(&op.stdout));

    // const PIPE_NAME: &str = r"\\.\pipe\dy_xhs";

    // let client = ClientOptions::new().open(PIPE_NAME).unwrap();
    // client.writable().await?;
    // client.try_write(b"Hello from Rust!").unwrap();

    // loop{
    //     client.readable().await?;
    //     let mut buf = Vec::with_capacity(4096);

    //     // Try to read data, this may still fail with `WouldBlock`
    //     // if the readiness event is a false positive.
    //     match client.try_read_buf(&mut buf) {
    //         Ok(0) => println!("read 0 bytes"),
    //         Ok(n) => {
    //             println!("read {} bytes", n);
    //             //将buf转换为字符串
    //             let s = String::from_utf8(buf).unwrap();
    //             println!("{}", s);
    //         }
    //         Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
    //             return Err(e.into());
    //         }
    //         Err(e) => {
    //             return Err(e.into());
    //         }
    //     }
    // }

    let redis_pool = create_redis_pool("redis://127.0.0.1/").await;

    let conf = Ini::load_from_file("conf.ini").unwrap();
    // let dy_download_dir = conf.general_section().get("dyDownloadDir").unwrap();
    let dy_download_dir = conf.get_from(None::<String>, "dyDownloadDir").unwrap();
    let dy_cookie = conf.get_from(None::<String>, "dyCookie").unwrap();
    let c = Box::new(Config {
        dy_download_dir: dy_download_dir.to_string(),
        dy_cookie: dy_cookie.to_string(),
    });
    unsafe {
        CONFIG = Some(Box::leak(c));
    }
    // let contents = fs::read_to_string(Path::new(dy_download_dir).join("poem.txt"))
    //     .expect("读取配置文件发生错误");

    let app = Router::new()
        .route("/", get(|| async { "Hello, World!" }))
        .route("/hook", post(hook))
        .layer(Extension(redis_pool));
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000").await.unwrap();
    axum::serve(listener, app).await.unwrap();
    Ok(())
}

#[derive(Debug, Deserialize, Validate)]
struct Monitor {
    #[validate(length(min = 1))]
    link: String,
}

struct Config {
    dy_download_dir: String,
    dy_cookie: String,
}

async fn hook(
    Extension(pool): Extension<Pool<RedisConnectionManager>>,
    Form(monitor): Form<Monitor>,
) {
    let url = monitor.link;
    if url.contains("douyin.com") {
        let r = Regex::new(r"(douyin\.com/user/)(?<author_id>[\w|-]+)").unwrap();
        let caps = r.captures(&url).unwrap();
        let author_id = &caps["author_id"];

        let mut conn = pool.get().await.unwrap();
        handle_douyin(author_id, 0, &mut conn).await.unwrap();
    }
}

async fn create_redis_pool(redis_url: &str) -> Pool<RedisConnectionManager> {
    let client = mobc_redis::redis::Client::open(redis_url).unwrap();
    let manager = RedisConnectionManager::new(client);
    Pool::builder()
        .max_open(100)
        .get_timeout(Some(Duration::from_secs(5)))
        .build(manager)
}

async fn handle_douyin(
    author_id: &str,
    mut max_cursor: u64,
    conn: &mut Connection<RedisConnectionManager>,
) -> Result<(), Box<dyn std::error::Error>> {
    //用于区分第一次和后续
    let while_count = 0;
    //has_more 代表是否还有更多，1代表有，0代表没有
    let mut has_more = 1;
    // let builder = ClientBuilder::new();
    /**
     * Client内部会维护一个连接池，对于相同的主机和端口的连接可复用，与请求参数，请求头等无关
     */
    let client = Client::new();
    while has_more == 1 {
        let mut params = generate_xb(format!(
            "aid=6383&sec_user_id={author_id}&count=20&max_cursor={max_cursor}&cookie_enabled=true&platform=PC&downlink=10"
        ).as_str());
        let xb = generate_xb(&params);
        params.push_str(format!("&X-Bogus={xb}").as_str());

        let mut map = HeaderMap::new();
        map.insert("Referer", "https://www.douyin.com/".parse()?);
        map.insert("User-Agent", "Mozilla/5.0 (Windows NT 10.0; Win64; x64) AppleWebKit/537.36 (KHTML, like Gecko) Chrome/104.0.0.0 Safari/537.36".parse()?);
        map.insert("Cookie", unsafe {
            CONFIG.as_ref().unwrap().dy_cookie.parse()?
        });
        let resp = client
            .get(format!(
                "https://www.douyin.com/aweme/v1/web/aweme/post/?{params}"
            ))
            .headers(map)
            .send()
            .await?;
        if !resp.status().is_success() {
            return Ok(());
        }
        let awemes_info: Value = resp.json().await?;
        let status_code = awemes_info["status_code"].as_i64().unwrap_or(-1);

        //status_code 不为 0 代表这个请求不符合要求
        if status_code != 0 {
            return Err("没有获取到数据".into());
        }
        has_more = awemes_info["has_more"].as_i64().unwrap_or(0);
        max_cursor = awemes_info["max_cursor"].as_u64().unwrap();
        let awemes = awemes_info["aweme_list"].as_array();
        if awemes.is_none() || awemes.unwrap().is_empty() {
            return Err("没有获取到数据".into());
        }

        let aweme = awemes.unwrap().get(0).unwrap();
        let author = aweme["author"].as_object().unwrap();
        let uid = aweme["uid"].as_str().unwrap();
        let nickname = author["nickname"].as_str().unwrap();
        let short_sec_uid = &author_id[author_id.len() - 6..];
        let base_path = Path::new(unsafe { CONFIG.as_ref().unwrap().dy_download_dir.as_str() })
            .join(format!("{uid}@{nickname}[{short_sec_uid}]"));
        //只在第一次循环时检查路径是否存在
        if while_count == 0 {
            //查看文件夹是否存在
            if !base_path.exists() {
                fs::create_dir_all(&base_path).unwrap();
            }
        }

        //迭代每一个 aweme
        for aweme in awemes.unwrap() {
            let aweme_id = aweme["aweme_id"].as_str().unwrap();
            let is_top = aweme["is_top"].as_i64().unwrap();
            let create_time = aweme["create_time"].as_i64().unwrap();
            //如果key不存在就将key的值设置为create_time
            let mut last_time = 0;
            if !conn.exists(aweme_id).await? {
                last_time = create_time;
                conn.set(aweme_id, create_time).await?;
            }
            if create_time <= last_time {
                if is_top == 1 {
                    continue;
                }
                break;
            }
            let aweme_type = aweme["aweme_type"].as_i64().unwrap();
            let time_format = Utc
                .timestamp_opt(create_time, 0)
                .unwrap()
                .format("%Y%m%d%H%M%S")
                .to_string();
            if DY_VIDEO_TYPE.contains(&aweme_type) {}
        }
        has_more = 0;
    }
    Ok(())
}
