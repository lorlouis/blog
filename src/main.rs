mod md_ex;

use actix_web::{dev::Service, get, web, App, HttpResponse, HttpServer, Responder};
use md_ex::ExtendedMd;
use serde::Deserialize;

use html_template::{html, Root};

use std::collections::BTreeMap;
use std::fs::File;
use std::io::{self, BufReader};
use std::path::PathBuf;
use tokio::fs::read_dir;

use time::OffsetDateTime;

use openssl::ssl::{SslAcceptor, SslFiletype, SslMethod};

const BASE_URL: &str = "https://louissven.xyz";

mod config {
    use lazy_static::lazy_static;
    use std::env::vars;

    lazy_static! {
        pub static ref FS_DATA_PATH: String = {
            vars().find(|(k, _v)| k == "FS_DATA_PATH")
                .map(|(_key, value)| value)
                .unwrap_or_else(|| "./data".to_string())
        };

        pub static ref FS_MEDIA_PATH: String = {
            vars().find(|(k, _v)| k == "FS_MEDIA_PATH")
                .map(|(_key, value)| value)
                .unwrap_or_else(|| "./media".to_string())
        };

        pub static ref FS_ARTICLES_PATH: String = {
            vars().find(|(k, _v)| k == "FS_ARTICLES_PATH")
                .map(|(_key, value)| value)
                .unwrap_or_else(|| "./articles".to_string())
        };

        pub static ref IP_BIND: String = {
            vars().find(|(k, _v)| k == "IP_BIND")
                .map(|(_key, value)| value)
                .unwrap_or_else(|| "0.0.0.0".to_string())
        };

        pub static ref HTTP_PORT: u16 = {
            vars().find(|(k, _v)| k == "HTTP_PORT")
                .map(|(_key, value)| value.parse().expect("invalid HTTP_PORT value"))
                .unwrap_or(80)
        };

        pub static ref HTTPS_PORT: u16 = {
            vars().find(|(k, _v)| k == "HTTPS_PORT")
                .map(|(_key, value)| value.parse().expect("invalid HTTPS_PORT value"))
                .unwrap_or(443)
        };

        pub static ref PRIVATE_KEY_FILEPATH: Option<String> = {
            vars().find(|(k, _v)| k == "PRIVATE_KEY_FILEPATH")
                .map(|(_key, value)| value)
                .or_else(|| {
                    log::warn!("No 'PRIVATE_KEY_FILEPATH' found in env, defaulting to http");
                    None
                })
        };

        pub static ref CERTIFICATE_CHAIN_FILEPATH: Option<String> = {
            vars().find(|(k, _v)| k == "CERTIFICATE_CHAIN_FILEPATH")
                .map(|(_key, value)| value)
                .or_else(|| {
                    log::warn!("No 'CERTIFICATE_CHAIN_FILEPATH' found in env, defaulting to http");
                    None
                })
        };

        pub static ref INDEX_MD_FILEPATH: String = {
            vars().find(|(k, _v)| k == "INDEX_MD_FILEPATH")
                .map(|(_key, value)| value)
                .unwrap_or_else(|| "./data/index.md".to_string())
        };

        pub static ref RENDER_WIP: bool = {
            vars().find(|(k, _v)| k == "RENDER_WIP")
                .map(|(_key, value)| !value.is_empty())
                .unwrap_or(false)
        };

        #[allow(clippy::assertions_on_constants)]
        static ref _ASSERT: () = assert!(*HTTP_PORT != *HTTPS_PORT, "cannot use the same port for http and https");

    }
}

#[derive(Deserialize)]
struct Page {
    #[serde(default)]
    pub p: usize,
}

async fn page_404() -> HttpResponse {
    let body: Root = html! {
        <!DOCTYPE html>
        <html>
            <head>
            { common_head("Page not found".to_string(), None, None)}
            </head>
            <body>
                <header>
                { common_header() }
                </header>
                <main>
                <h1>"Page not found"</h1>
                </main>
                <footer>
                { common_footer() }
                </footer>
            </body>
        </html>
    }
    .into();
    HttpResponse::NotFound()
        .content_type(mime::TEXT_HTML)
        .body(body.to_string())
}

fn page_500(e: impl std::error::Error) -> HttpResponse {
    let error = e.to_string();
    let body: Root = html! {
        <!DOCTYPE html>
        <html>
            <head>
            { common_head("Page not found".to_string(), None, None)}
            </head>
            <body>
                <header>
                { common_header() }
                </header>
                <main>
                <h1>"Internal server error"</h1>
                <pre><code>{ error.to_string() }</code></pre>
                </main>
                <footer>
                { common_footer() }
                </footer>
            </body>
        </html>
    }
    .into();
    HttpResponse::NotFound()
        .content_type(mime::TEXT_HTML)
        .body(body.to_string())
}

macro_rules! yeet_404 {
    ($v:expr) => {
        match $v {
            Ok(v) => v,
            Err(_) => return page_404().await,
        }
    };
}

macro_rules! yeet_500 {
    ($v:expr) => {
        match $v {
            Ok(v) => v,
            Err(e) => return page_500(e),
        }
    };
}

fn common_header() -> String {
    html! {
        <div id="header_top_div">
            <a href="/" class="header_element">home</a>
            <a href="/articles" class="header_element">articles</a>
            <a href="/data-policy" class="header_element">"data policy"</a>
            <a href="https://github.com/lorlouis" class="header_element">github</a>
            <a href="/rss" class="header_element">"rss"</a>
        </div>
    }
    .to_string()
}

fn copyright_str() -> String {
    let now = OffsetDateTime::now_utc();
    let year = now.year();
    format!("copyright Louis Sven Goulet 2023-{}", year)
}

fn copyright() -> String {
    html! {
        <p id="copyright">
        "Found a typo?"
        <a href="https://www.github.com/lorlouis/blog">" open a pr!"</a>
        <br>
        { copyright_str() }
        </p>
    }
    .to_string()
}

fn common_footer() -> String {
    html! {
        <div id="page_link_div"></div>
        { copyright() }
    }
    .to_string()
}

fn common_head(title: String, author: Option<String>, blurb: Option<String>) -> String {
    let author = author.unwrap_or_else(|| "Louis Sven Goulet".to_string());
    html! {
        <base href="/" >
        <link rel="stylesheet" href="data/site.css">
        <!-- <meta name="viewport" content="width=device-width"> -->
        <link rel="apple-touch-icon" sizes="180x180" href="/data/favicon_io/apple-touch-icon.png">
        <link rel="icon" type="image/png" sizes="32x32" href="/data/favicon_io/favicon-32x32.png">
        <link rel="icon" type="image/png" sizes="16x16" href="/data/favicon_io/favicon-16x16.png">
        <link rel="manifest" href="/data/favicon_io/site.webmanifest">

        <title>{title.clone()}</title>
        <meta charset="UTF-8">
        {
            blurb.iter().map(|v| html!{
                <meta name="description" content={[move] format!("\"{}\"", v)}>
            }).collect()
        }
        <meta name="author" content={[move] format!("\"{}\"", author)}>
        <link rel="stylesheet" href="/data/highlight/styles/nord.min.css">
        <script src="/data/highlight/highlight.min.js"></script>
        <script>hljs.highlightAll();</script>
    }
    .to_string()
}

async fn basic_md_page(path: &str) -> impl Responder {
    let md_file = yeet_500!(File::open(path));

    let markdown = yeet_500!(md_ex::ExtendedMd::from_bufread(BufReader::new(md_file)));
    let title = markdown.header.get("Title").cloned().unwrap_or_default();
    let body: Root = html! {
        <!DOCTYPE html>
        <html>
            <head>
            { common_head(title.to_string(), None, None)}
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
    }
    .into();
    HttpResponse::Ok()
        .content_type(mime::TEXT_HTML)
        .body(body.to_string())
}

#[get("/")]
async fn index() -> impl Responder {
    let index_file = yeet_500!(File::open(config::INDEX_MD_FILEPATH.as_str()));

    let markdown = yeet_500!(md_ex::ExtendedMd::from_bufread(BufReader::new(index_file)));

    let posts = yeet_500!(get_articles().await);

    let posts_ref = posts.as_slice();

    let body: Root = html! {
        <!DOCTYPE html>
        <html>
            <head>
            { common_head("Louis' imperfect blog".to_string(), None, None)}
            </head>
            <body>
                <header>
                { common_header() }
                </header>
                <main>
                { markdown.to_html() }
                <h3>"Recent Articles"</h3>
                { build_articles_html_list(posts_ref, 5, 0) }
                </main>
                <footer>
                { common_footer() }
                </footer>
            </body>
        </html>
    }
    .into();
    HttpResponse::Ok()
        .content_type(mime::TEXT_HTML)
        .body(body.to_string())
}

fn build_articles_html_list(
    posts: &[(String, String, BTreeMap<String, String>)],
    count: usize,
    skip: usize,
) -> String {
    let trimmed_articles: Vec<_> = posts.iter().skip(skip).take(count).collect();
    html! {
        <div id="article_container" >
        { [move]
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
                            <blockquote class="blurb">
                            {v.to_string()}
                            </blockquote>
                        }).collect()}
                    </article>
                }
            }).collect()
        }
        </div>
    }
    .to_string()
}

#[get("/rss")]
async fn rss<'a>() -> impl Responder + 'a {
    let articles_ = yeet_500!(get_articles().await);

    HttpResponse::Ok()
        .content_type(mime::TEXT_XML)
        .body(build_rss(&articles_))
}

fn build_rss(posts: &[(String, String, BTreeMap<String, String>)]) -> String {
    let trimmed_articles = posts;

    // just a little lie
    html! {
        <?xml version="1.0" encoding="UTF-8" ?>
        <rss version="2.0">
        <channel>
        <title>"<![CDATA[Louis' imperfect blog]]>"</title>
        <description>"<![CDATA[Louis' imperfect blog's RSS feed]]>"</description>
        <link>{BASE_URL.into()}</link>
        <copyright>{ copyright_str() }</copyright>
        {
            posts.first().map(|post| {
                let date = &post.0;
                html!{<pubDate>{date.to_string()}</pubDate>}.to_string()
            }).unwrap_or_default()
        }
        <ttl>1800</ttl>

        { [move]
            trimmed_articles.iter()
                .map(|(date, name, data)| -> io::Result<html_template::Node> {
                    let title = data.get("Title")
                        .unwrap_or(name);
                    let full_title = format!(
                        "{} – {}",
                        title,
                        data.get("Blurb")
                        .map(|v| v.as_str())
                        .unwrap_or("")
                    );

                    let mut md_path = PathBuf::from(config::FS_ARTICLES_PATH.as_str());
                    md_path.push(&name);

                    let file = File::open(md_path)?;

                    let markdown = md_ex::ExtendedMd::from_bufread(BufReader::new(file))
                        .map_err(|e| io::Error::other(e))?;

                    Ok(html! {
                    <item>
                        <title>{[move] format!("<![CDATA[{}]]>", full_title) }</title>
                        <link>{[move] format!("{}/article/{}", BASE_URL, name)}</link>
                        <description>{[move] format!("<![CDATA[{}]]>", article_page(&markdown, name)) }</description>
                    </item>
                    }
                )
            })
            .filter_map(Result::ok)
            .collect()
        }
        </channel>
        </rss>
    }.to_string()
}

async fn get_articles() -> io::Result<Vec<(String, String, BTreeMap<String, String>)>> {
    let mut dir = read_dir(config::FS_ARTICLES_PATH.as_str()).await?;
    let mut posts = Vec::new();
    loop {
        // ugly but I can't flatten due to the await
        let res = dir.next_entry().await;
        let entry = match res? {
            Some(v) => v,
            None => break,
        };

        let metadata = entry.metadata().await?;
        let entry_name = entry.file_name().to_string_lossy().to_lowercase();

        let is_markdown =
            entry_name.ends_with(".md") || (*config::RENDER_WIP && entry_name.ends_with(".md.wip"));

        if metadata.is_file() && is_markdown {
            // normal std::fs::File because tokio's async BufReader is really annoying
            let file = BufReader::new(File::open(entry.path())?);
            let article_data = match md_ex::ExtendedMd::read_header(file) {
                Ok(v) => v,
                Err(e) => {
                    // ignore the error
                    log::error!(
                        "Ran into error:{e:?}; for file: '{}'",
                        entry.path().display()
                    );
                    continue;
                }
            };
            let date = article_data
                .get("Date")
                .cloned()
                .unwrap_or_else(|| "31005-12-01".to_string());
            posts.push((
                date,
                entry.file_name().to_string_lossy().to_string(),
                article_data,
            ));
        }
    }
    posts.sort_unstable_by(|s, o| {
        s.0.cmp(&o.0)
            // make the oldest articles appear at the end
            .reverse()
    });
    Ok(posts)
}

#[get("/articles")]
async fn articles<'a>(info: web::Query<Page>) -> impl Responder + 'a {
    const ARTICLES_PER_PAGE: usize = 8;

    let page = info.0.p;

    let articles = yeet_500!(get_articles().await);

    let last_page = articles.len() / ARTICLES_PER_PAGE;
    let cur_page = last_page.min(page);

    let articles_ref = articles.as_slice();

    let body: Root = html! {
        <!DOCTYPE html>
        <html>
            <head>
            { common_head("articles".to_string(), None, None) }
            </head>
            <body>
                { common_header() }
                <main>
                <h1>Articles</h1>
                {[move] build_articles_html_list(
                            articles_ref,
                            ARTICLES_PER_PAGE,
                            ARTICLES_PER_PAGE * cur_page)
                }
                </main>
                <footer>
                <div id="page_link_div">
                <a
                    href="/articles"
                    class="article_link"
                    id="link_first_page"
                    title="fist page"
                >&lt;&lt;</a>
                <a
                    href={format!{"\"articles?p={}\" ", cur_page.saturating_sub(1)}}
                    class="article_link"
                    id="link_previous_page"
                    title="previous page"
                >&lt;</a>
                <a
                    href={format!{"\"articles?p={}\" ", cur_page.saturating_add(1)}}
                    class="article_link"
                    id="link_next_page"
                    title="next page"
                >&gt;</a>
                <a
                    href={format!{"\"articles?p={}\" ", last_page}}
                    class="article_link"
                    id="link_last_page"
                    title="last page"
                >&gt;&gt;</a>
                </div>
                { copyright() }
                </footer>
            </body>
        </html>
    }
    .into();

    HttpResponse::Ok()
        .content_type(mime::TEXT_HTML)
        .body(body.to_string())
}

fn article_page(markdown: &ExtendedMd, title: &String) -> String {
    let real_title = markdown.header.get("Title").unwrap_or(title);
    let author = markdown.header.get("Author");
    let blurb = markdown.header.get("Blurb");

    let body: Root = html! {
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
    }
    .into();

    body.to_string()
}

#[get("/article/{title}")]
async fn article<'a>(title: web::Path<String>) -> impl Responder + 'a {
    let title = title.into_inner();
    let mut md_path = PathBuf::from(config::FS_ARTICLES_PATH.as_str());
    md_path.push(&title);

    let file = yeet_404!(File::open(md_path));

    let markdown = yeet_404!(md_ex::ExtendedMd::from_bufread(BufReader::new(file)));

    HttpResponse::Ok()
        .content_type(mime::TEXT_HTML)
        .body(article_page(&markdown, &title))
}

#[actix_web::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // configure logging
    env_logger::init_from_env(env_logger::Env::default().default_filter_or("info"));

    let new_website = || {
        App::new()
            .wrap_fn(|req, srv| {
                let connection_info = req.connection_info().clone();
                let target: String = req.uri().to_string().escape_debug().collect();
                let remote_addr = connection_info
                    .realip_remote_addr()
                    .map(|v| v.escape_debug().collect())
                    .unwrap_or_else(|| "UNKNOWN".to_string());
                let agent = req
                    .headers()
                    .get("User-Agent")
                    .map(|v| String::from_utf8_lossy(v.as_bytes()))
                    .map(|v| v.escape_debug().collect())
                    .unwrap_or("UNKNOWN".to_string());

                log::info!(
                    "Connection from: '{}'; With agent: '{}'; For target: '{}'",
                    remote_addr,
                    agent,
                    target
                );
                srv.call(req)
            })
            .service(index)
            .service(article)
            .service(articles)
            .service(rss)
            .route(
                "/data-policy",
                web::get().to(|| basic_md_page("./data/data_policy.md")),
            )
            .service(
                actix_files::Files::new("/media", config::FS_MEDIA_PATH.as_str()).prefer_utf8(true),
            )
            .service(
                actix_files::Files::new("/data", config::FS_DATA_PATH.as_str()).prefer_utf8(true),
            )
            .default_service(web::to(page_404))
    };

    if let (Some(private_key), Some(cert)) = (
        config::PRIVATE_KEY_FILEPATH.as_deref(),
        config::CERTIFICATE_CHAIN_FILEPATH.as_deref(),
    ) {
        // load TLS keys
        // to create a self-signed temporary cert for testing:
        // `openssl req -x509 -newkey rsa:4096 -nodes -keyout key.pem -out cert.pem -days 365 -subj '/CN=localhost'`
        let mut builder = SslAcceptor::mozilla_intermediate(SslMethod::tls()).unwrap();
        builder
            .set_private_key_file(private_key, SslFiletype::PEM)
            .unwrap();
        builder.set_certificate_chain_file(cert).unwrap();

        futures::try_join!(
            // https
            HttpServer::new(new_website)
                .bind_openssl(
                    format!("{}:{}", config::IP_BIND.as_str(), *config::HTTPS_PORT),
                    builder
                )
                .map_err(|e| format!(
                    "unable to bind on https port: {} error: {}",
                    *config::HTTPS_PORT,
                    e
                ))?
                .run(),
            // http
            HttpServer::new(new_website)
                .bind((config::IP_BIND.as_str(), *config::HTTP_PORT))
                .map_err(|e| format!(
                    "unable to bind on http port: {} error: {}",
                    *config::HTTP_PORT,
                    e
                ))?
                .run(),
        )?;
    } else {
        // http only
        HttpServer::new(new_website)
            .bind((config::IP_BIND.as_str(), *config::HTTP_PORT))
            .map_err(|e| {
                format!(
                    "unable to bind on http port: {} error: {}",
                    *config::HTTP_PORT,
                    e
                )
            })?
            .run()
            .await?;
    }

    Ok(())
}
