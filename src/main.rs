mod md_ex;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

use html_template_core::Root;
use html_template_macros::html;

use std::path::PathBuf;
use std::fs::{File, read_dir};
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

fn page_500(e: impl std::error::Error) -> HttpResponse {
    let error = e.to_string();
    let body: Root = html!{
        <!DOCTYPE html>
        <html>
            <body>
                <h1>{ format!("Internal error: {}", error) }</h1>
            </body>
        </html>
    }.into();
    HttpResponse::InternalServerError()
        .content_type(mime::TEXT_HTML)
        .body(body.to_string())
}

macro_rules! yeet_404 {
    ($v:expr) => { match $v {
        Ok(v) => v,
        Err(_) => return page_404().await
    }};
}

macro_rules! yeet_500 {
    ($v:expr) => { match $v {
        Ok(v) => v,
        Err(e) => return page_500(e)
    }};
}


fn common_header() -> String {
    html! {
        <div id="header_top_div">
            <a href="/" class="header_element">home</a>
            <a href="/articles" class="header_element">articles</a>
            <a href="https://github.com/lorlouis" class="header_element">github</a>
        </div>
    }.to_string()
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
                <header>
                { common_header() }
                </header>

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
    const ARTICLES_PER_PAGE: usize = 1;

    let page = info.0.p;

    let find_articles = || -> Result<_, std::io::Error> {
        let dir = read_dir("articles")?;
        let mut articles = Vec::new();
        for entry in dir.flatten() {
            let metadata = entry.metadata()?;
            if metadata.is_file()
                && entry.file_name().to_string_lossy().to_lowercase().ends_with(".md") {
                    let file = File::open(entry.path())?;
                    let article_data = match md_ex::ExtendedMd::read_header(BufReader::new(file)) {
                        Ok(v) => v,
                        Err(e) => {
                            // ignore the error
                            eprintln!("ran into an error: {e:?} for file: {}", entry.path().display());
                            continue
                        }
                    };
                    let date = article_data.get("Date")
                        .cloned()
                        .unwrap_or_else(|| "4096-03-23".to_string());
                    articles.push((date, entry.file_name().to_string_lossy().to_string(), article_data));
            }
        }
        Ok(articles)
    };

    let mut articles = yeet_500!(find_articles());
    let last_page = (articles.len() / ARTICLES_PER_PAGE).saturating_sub(1);
    let cur_page = last_page.min(page as usize);

    articles.sort_unstable_by(|s, o| s.0.cmp(&o.0));

    let trimmed_articles: Vec<_> = articles.into_iter()
                        .skip(cur_page * ARTICLES_PER_PAGE)
                        .take(ARTICLES_PER_PAGE)
                        .collect();

    let body: Root = html!{
        <!DOCTYPE html>
        <html>
            <head>
            { common_head("articles".to_string(), None, None) }
            </head>
            <body>
                { common_header() }

                <h1>Articles</h1>
                <div>
                {
                    trimmed_articles.clone().into_iter()
                        .map(|(date, name, data)| {
                            let title = data.get("Title")
                                .cloned()
                                .unwrap_or_else(|| name.clone());
                            html! {
                            <div>
                                <h3 class="list_element">
                                    { format!("{}", date) }
                                </h3>
                                <h3 class="list_element">
                                    <a href={format!("article/{}", name)}>
                                        {format!("{}", title)}
                                    </a>
                                </h3>
                            </div>
                        }
                    }).collect()
                }
                <div>
                <a href="/articles" id="link_first_page">&lt;&lt;</a>
                <a href={format!{"articles?p={}", cur_page.saturating_sub(1)}} id="link_last_page">&lt;</a>
                <a href={format!{"articles?p={}", cur_page.saturating_add(1)}} id="link_next_page">&gt;</a>
                <a href={format!{"articles?p={}", last_page}} id="link_last_page">&gt;&gt;</a>
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
                <header>
                { common_header() }
                </header>
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
