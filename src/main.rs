mod md_ex;

use actix_web::{get, web, App, HttpResponse, HttpServer, Responder};
use serde::Deserialize;

use html_template_core::Root;
use html_template_macros::html;

use std::fs::File;
use std::path::PathBuf;
use tokio::fs::read_dir;
use std::io::BufReader;

use time::OffsetDateTime;

use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

mod config {
    pub const FS_ARTICLES_PATH: &str = "./articles";
    pub const FS_DATA_PATH: &str = "./data";
    pub const FS_MEDIA_PATH: &str = "./media";

    pub const IP_BIND: &str = "0.0.0.0";

    pub const HTTP_PORT: u16 = 8080;

    pub const HTTPS_PORT: u16 = 4430;

    pub const PRIVATE_KEY_FILEPATH: &str = "key.pem";

    pub const CERTIFICATE_CHAIN_FILEPATH: &str = "cert.pem";


    #[allow(clippy::assertions_on_constants)]
    const _: () = assert!(HTTP_PORT != HTTPS_PORT, "cannot use the same port for http and https");
}


#[derive(Deserialize)]
struct Page {
    #[serde(default)]
    pub p: usize,
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

fn copyright() -> String {
    let now = OffsetDateTime::now_utc();
    let year = now.year();
    html! {
        <p id="copyright">
        {[move] format!("copyright Louis Sven Goulet 2023-{}", year)}
        </p>
    }.to_string()
}

fn common_footer() -> String {
    html! {
        <div id="page_link_div"></div>
        { copyright() }
    }.to_string()
}

fn common_head(title: String, author: Option<String>, blurb: Option<String>) -> String {
    let author = author.unwrap_or_else(|| "Louis Sven Goulet".to_string());
    html! {
        <title>{title.clone()}</title>
        <meta charset="UTF-8">
        {
            blurb.iter().map(|v| html!{
                <meta name="description" content={[move] format!("\"{}\"", v)}>
            }).collect()
        }
        <base href="/" >
        <meta name="author" content={[move] format!("\"{}\"", author)}>
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
                <main>
                <h1>"Louis' imperfect blog"</h1>
                </main>
            </body>
            <footer>
            { common_footer() }
            </footer>
        </html>
    }.into();
    HttpResponse::Ok()
        .content_type(mime::TEXT_HTML)
        .body(body.to_string())
}


#[get("/articles")]
async fn articles<'a>(info: web::Query<Page>) -> impl Responder + 'a {
    const ARTICLES_PER_PAGE: usize = 15;

    let page = info.0.p;

    let mut dir = yeet_500!(read_dir(config::FS_ARTICLES_PATH).await);
    let mut articles = Vec::new();
    loop {
        // ugly but I can't flatten due to the await
        let res = dir.next_entry().await;
        let entry = match yeet_500!(res) {
            Some(v) => v,
            None => break,
        };

        let metadata = yeet_500!(entry.metadata().await);
        if metadata.is_file()
            && entry.file_name().to_string_lossy().to_lowercase().ends_with(".md") {
                // normal std::fs::File because tokio's async BufReader is really annoying
                let file = BufReader::new(yeet_500!(File::open(entry.path())));
                let article_data = match md_ex::ExtendedMd::read_header(file) {
                    Ok(v) => v,
                    Err(e) => {
                        // ignore the error
                        eprintln!("ran into an error: {e:?} for file: {}", entry.path().display());
                        continue
                    }
                };
                let date = article_data.get("Date")
                    .cloned()
                    .unwrap_or_else(|| "31005-12-01".to_string());
                articles.push((date, entry.file_name().to_string_lossy().to_string(), article_data));
        }
    }

    let last_page = (articles.len() / ARTICLES_PER_PAGE).saturating_sub(1);
    let cur_page = last_page.min(page);

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
                <main>
                <h1>Articles</h1>
                <div id="article_container" >
                {
                    trimmed_articles.iter()
                        .map(|(date, name, data)| {
                            let title = data.get("Title")
                                .unwrap_or(name);
                            let blurb = data.get("Blurb");
                            html! {
                            <article>
                                <h3 class="list_element">
                                    {[move] format!("{}", date) }
                                </h3>
                                <h3 class="list_element">
                                    <a href={[move] format!("\"article/{}\"", name)}>
                                        {[move] format!("{}", title)}
                                    </a>
                                </h3>
                                {[move] blurb.iter().map(|v| html!{
                                    <blockquote id="blurb">
                                    {v.to_string()}
                                    </blockquote>
                                }).collect()}
                            </article>
                        }
                    }).collect()
                }
                </div>
                </main>
                <div id="page_link_div">
                <a
                    href="/articles"
                    class="article_link"
                    id="link_first_page"
                    title="fist page"
                >&lt;&lt;</a>
                <a
                    href={format!{"\"articles?p={}\"", cur_page.saturating_sub(1)}}
                    class="article_link"
                    id="link_previous_page"
                    title="previous page"
                >&lt;</a>
                <a
                    href={format!{"\"articles?p={}\"", cur_page.saturating_add(1)}}
                    class="article_link"
                    id="link_next_page"
                    title="next page"
                >&gt;</a>
                <a
                    href={format!{"\"articles?p={}\"", last_page}}
                    class="article_link"
                    id="link_last_page"
                    title="last page"
                >&gt;&gt;</a>
                </div>
                <footer>
                { copyright() }
                </footer>
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
    let mut md_path = PathBuf::from(config::FS_ARTICLES_PATH);
    md_path.push(&title);

    let file = yeet_404!(File::open(md_path));

    let markdown = yeet_404!(md_ex::ExtendedMd::from_bufread(BufReader::new(file)));

    let real_title = markdown.header.get("Title").unwrap_or(&title);
    let author = markdown.header.get("Author");
    let blurb = markdown.header.get("Blurb");

    let body: Root = html!{
        <!DOCTYPE html>
        <html>
            <head>
                {common_head(real_title.clone(), author.cloned(), blurb.cloned())}
            </head>
            <body>
                <header>
                { common_header() }
                </header>
                <main>
                { markdown.to_html() }
                </main>
                <footer>
                { common_footer() }
                </footer>
            </body>
        </html>
    }.into();

    HttpResponse::Ok()
        .content_type(mime::TEXT_HTML)
        .body(body.to_string())
}

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // load TLS keys
    // to create a self-signed temporary cert for testing:
    // `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`
    let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
    builder
        .set_private_key_file(config::PRIVATE_KEY_FILEPATH, SslFiletype::PEM)
        .unwrap();
    builder.set_certificate_chain_file(config::CERTIFICATE_CHAIN_FILEPATH).unwrap();

    let new_website = ||
        App::new()
            .service(index)
            .service(article)
            .service(articles)
            .service(actix_files::Files::new("/media", config::FS_MEDIA_PATH).prefer_utf8(true))
            .service(actix_files::Files::new("/data", config::FS_DATA_PATH).prefer_utf8(true))
            .default_service(web::to(page_404));

    futures::try_join!(
        // https
        HttpServer::new(new_website)
        .bind_openssl(format!("{}:{}", config::IP_BIND, config::HTTPS_PORT), builder)?
        .run(),
        // http
        HttpServer::new(new_website)
        .bind((config::IP_BIND, config::HTTP_PORT))?
        .run(),
    )?;
    Ok(())
}
