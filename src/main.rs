mod md_ex;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

use html_template_core::Root;
use html_template_macros::html;

use std::path::PathBuf;
use std::fs::File;
use std::io::BufReader;

const BASE_ARTICLE_PATH: &str = "./articles/";

#[derive(Deserialize)]
struct Page {
    #[serde(default)]
    pub p: u32,
}

async fn page_404() -> HttpResponse {
    let body: Root = html!{
        <!DOCTYPE html>
        <html>
            <body>
                <h1>"Page not found"</h1>
            </body>
        </html>
    }.into();
    HttpResponse::NotFound()
        .content_type(mime::TEXT_HTML)
        .body(body.to_string())
}

macro_rules! yeet_404 {
    ($v:expr) => { match $v {
        Ok(v) => v,
        Err(_) => return page_404().await
    }};
}

fn common_head(title: String, author: Option<String>, blurb: Option<String>) -> String {
    let author = author.unwrap_or_else(|| "Louis Sven Goulet".to_string());
    html! {
        <title>{title.clone()}</title>
        <meta charset="UTF-8">
        {
            blurb.clone().into_iter().map(|v| html!{
                <meta name="description" content={v.to_string()}>
            }).collect()
        }
        <base href="/" >
        <meta name="author" content={author.clone()}>
        <link rel="stylesheet" href="data/site.css">
    }.to_string()
}

#[get("/")]
async fn index() -> impl Responder {
    let body: Root = html!{
        <!DOCTYPE html>
        <html>
            <head>
            { common_head("LSG".to_string(), None, None)}
            </head>
            <body>

                <h1>"Hello world"</h1>

            </body>
        </html>
    }.into();
    HttpResponse::Ok()
        .content_type(mime::TEXT_HTML)
        .body(body.to_string())
}


#[get("/articles")]
async fn articles<'a>(info: web::Query<Page>) -> impl Responder + 'a {
    let page = info.0.p;
    let body: Root = html!{
        <!DOCTYPE html>
        <html>
            <head>
            { common_head("articles".to_string(), None, None) }
            </head>
            <body>
                <h1>{ page.to_string() }</h1>
            </body>
        </html>
    }.into();

    HttpResponse::Ok()
        .content_type(mime::TEXT_HTML)
        .body(body.to_string())
}

#[get("/article/{title}")]
async fn article<'a>(title: web::Path<String>) -> impl Responder + 'a {

    let title = title.into_inner();
    let mut md_path = PathBuf::from(BASE_ARTICLE_PATH);
    md_path.push(&title);

    let file = yeet_404!(File::open(md_path));

    let markdown = yeet_404!(md_ex::ExtendedMd::from_bufread(BufReader::new(file)));

    let real_title = markdown.header.get("Title").unwrap_or(&title).clone();
    let author = markdown.header.get("Author").cloned();
    let blurb = markdown.header.get("Blurb").cloned();

    let body: Root = html!{
        <!DOCTYPE html>
        <html>
            <head>
                {common_head(real_title.clone(), author.clone(), blurb.clone())}
            </head>
            <body>
            { markdown.to_html() }
            </body>
        </html>
    }.into();

    HttpResponse::Ok()
        .content_type(mime::TEXT_HTML)
        .body(body.to_string())
}


#[actix_web::main]
async fn main() -> std::io::Result<()> {
    HttpServer::new(|| {
        App::new()
            .service(index)
            .service(article)
            .service(articles)
            .service(actix_files::Files::new("/media", "./media").prefer_utf8(true))
            .service(actix_files::Files::new("/data", "./data").prefer_utf8(true))
            .default_service(web::to(page_404))
    })
    .bind(("0.0.0.0", 8080))?
    .run()
    .await
}
