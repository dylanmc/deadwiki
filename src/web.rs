use ascii::AsciiString;
use percent_encoding::percent_decode;
use pulldown_cmark as markdown;
use threadpool::ThreadPool;
use tiny_http::{
    Method::{Get, Post},
    Request, Response, Server,
};

use std::{
    fs,
    io::{self, Write},
    os::unix::fs::PermissionsExt,
    path::Path,
    process::Command,
    str,
};

/// How many threads to run. Keep it low, this is for personal use!
const MAX_WORKERS: usize = 10;

/// Run the web server.
pub fn server(host: &str, port: usize) -> Result<(), io::Error> {
    let pool = ThreadPool::new(MAX_WORKERS);
    let addr = format!("{}:{}", host, port);
    let server = Server::http(&addr).expect("Server Error: ");
    println!("-> running at http://{}", addr);

    for request in server.incoming_requests() {
        pool.execute(move || {
            if let Err(e) = handle(request) {
                eprintln!("!> {}", e);
            }
        });
    }

    Ok(())
}

/// Handle a single request.
fn handle(mut req: Request) -> Result<(), io::Error> {
    let (status, body, content_type) = match route(&mut req) {
        Ok(res) => res,
        Err(e) => {
            eprintln!("{}", e);
            (
                500,
                format!("<h1>500 Internal Error</h1><pre>{}</pre>", e),
                "text/html",
            )
        }
    };

    let response = if status == 302 {
        Response::from_data(format!("Redirected to {}", body))
            .with_status_code(status)
            .with_header(header("Location", &body))
    } else {
        Response::from_data(body)
            .with_status_code(status)
            .with_header(header("Content-Type", content_type))
    };

    println!("-> {} {} {}", status, req.method(), req.url());
    req.respond(response)
}

/// Generate a Header for tiny_http.
fn header(field: &str, value: &str) -> tiny_http::Header {
    tiny_http::Header {
        field: field.parse().unwrap(),
        value: AsciiString::from_ascii(value).unwrap_or_else(|_| AsciiString::new()),
    }
}

/// Route a request.
fn route(req: &mut Request) -> Result<(i32, String, &'static str), io::Error> {
    let mut status = 404;
    let mut body = "404 Not Found".to_string();
    let mut content_type = "text/html; charset=utf8";

    let full_url = req.url().to_string();
    let mut parts = full_url.splitn(2, "?");
    let (url, query) = (parts.next().unwrap_or("/"), parts.next().unwrap_or(""));

    match (req.method(), url) {
        (Get, "/") => {
            status = 200;
            body = render_with_layout(
                "deadwiki",
                &format!(
                    "<ul>{}</ul>",
                    wiki_page_names()
                        .iter()
                        .map(|name| format!(r#"<li><a href="{}">{}</a></li>"#, name, name))
                        .collect::<String>()
                ),
                None,
            );
        }
        (Get, "/new") => {
            status = 200;
            body = render_with_layout(
                "new page",
                &fs::read_to_string(web_path("new.html").unwrap_or_else(|| "?".into()))?,
                None,
            );
        }
        (Get, "/sleep") => {
            status = 200;
            body = "Zzzzz...".into();
            std::thread::sleep(std::time::Duration::from_secs(5));
        }
        (Get, "/404") => {
            status = 404;
            body = fs::read_to_string("web/404.html")?;
        }
        (Post, "/new") => {
            let mut content = String::new();
            req.as_reader().read_to_string(&mut content)?;
            let mut path = String::new();
            let mut mdown = String::new();
            for pair in content.split('&') {
                let mut parts = pair.splitn(2, '=');
                let (field, value) = (
                    parts.next().unwrap_or_default(),
                    parts.next().unwrap_or_default(),
                );
                match field.as_ref() {
                    "name" => path = decode_form_value(value),
                    "markdown" => mdown = decode_form_value(value),
                    _ => {}
                }
            }
            if !wiki_page_names().contains(&path.to_lowercase()) {
                if let Some(disk_path) = new_wiki_path(&path) {
                    let mut file = fs::File::create(disk_path)?;
                    write!(file, "{}", mdown)?;
                    status = 302;
                    body = path.to_string();
                }
            }
        }
        (Post, path) => {
            if query.is_empty() {
                status = 404;
                body = fs::read_to_string("web/404.html")?;
            } else {
                if let Some(disk_path) = wiki_path(path) {
                    let mut content = String::new();
                    req.as_reader().read_to_string(&mut content)?;
                    let mdown = content.split("markdown=").last().unwrap_or("");
                    let mut file = fs::File::create(disk_path)?;
                    write!(file, "{}", decode_form_value(mdown))?;
                    status = 302;
                    body = path.to_string();
                } else {
                    status = 404;
                    body = fs::read_to_string("web/404.html")?;
                }
            }
        }

        (Get, path) => {
            if let Some(disk_path) = wiki_path(path) {
                status = 200;
                if query.is_empty() {
                    body = render_wiki(path).unwrap_or_else(|| "".into());
                } else if query == "edit" {
                    body = render_with_layout(
                        "Edit",
                        &web_to_string("edit.html")
                            .unwrap_or_else(|| "".into())
                            .replace("{markdown}", &fs::read_to_string(disk_path)?),
                        None,
                    )
                }
            } else if let Some(web_path) = web_path(path) {
                status = 200;
                body = fs::read_to_string(web_path)?;
                content_type = get_content_type(path).unwrap_or("text/plain");
            } else {
                status = 404;
                body = fs::read_to_string("web/404.html")?;
            }
        }

        (x, y) => println!("x: {:?}, y: {:?}", x, y),
    }

    Ok((status, body, content_type))
}

/// some_page -> Some Page
fn path_to_title(path: &str) -> String {
    path.trim_start_matches('/')
        .split("_")
        .map(|part| {
            format!(
                "{}{}",
                part.chars().next().unwrap_or('?').to_uppercase(),
                &part.chars().skip(1).collect::<String>()
            )
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Render a wiki page to a fully loaded HTML string.
/// Wiki pages are stored in the `wiki/` directory as `.md` files.
fn render_wiki(path: &str) -> Option<String> {
    let raw = path.ends_with(".md");
    let path = if raw {
        path.trim_end_matches(".md")
    } else {
        path
    };
    let title = path_to_title(path);
    if let Some(path) = wiki_path(path) {
        let html = if is_executable(&path) {
            shell(&path, &[]).unwrap_or_else(|e| e.to_string())
        } else {
            fs::read_to_string(path).unwrap_or_else(|_| "".into())
        };
        Some(if raw {
            format!("<pre>{}</pre>", html)
        } else {
            render_with_layout(&title, &markdown_to_html(&html), Some(&nav()))
        })
    } else {
        None
    }
}

/// Return the <nav> for a page
fn nav() -> String {
    fs::read_to_string("./web/nav.html").unwrap_or("".into())
}

/// Renders a chunk of HTML surrounded by `web/layout.html`.
fn render_with_layout(title: &str, body: &str, nav: Option<&str>) -> String {
    if let Some(layout) = web_path("layout.html") {
        fs::read_to_string(layout)
            .unwrap_or_else(|_| "".into())
            .replace("{title}", title)
            .replace("{body}", body)
            .replace("{nav}", nav.unwrap_or(""))
    } else {
        body.to_string()
    }
}

/// Lowercase names of all the wiki pages.
fn wiki_page_names() -> Vec<String> {
    let mut dirs = vec![];

    if let Ok(entries) = fs::read_dir("./wiki") {
        for dir in entries {
            if let Ok(dir) = dir {
                dirs.push(
                    dir.path()
                        .to_str()
                        .unwrap_or_else(|| "?")
                        .trim_start_matches("./wiki/")
                        .trim_end_matches(".md")
                        .to_string(),
                )
            }
        }
    }

    dirs
}

/// Convert raw Markdown into HTML.
fn markdown_to_html(md: &str) -> String {
    let mut options = markdown::Options::empty();
    options.insert(markdown::Options::ENABLE_TASKLISTS);
    options.insert(markdown::Options::ENABLE_FOOTNOTES);

    // are we parsing a wiki link like [Help] or [Solar Power]?
    let mut wiki_link = false;
    // if we are, store the text between [ and ]
    let mut wiki_link_text = String::new();

    let parser = markdown::Parser::new_ext(&md, options).map(|event| match event {
        markdown::Event::Text(text) => {
            if text.as_ref() == "[" && !wiki_link {
                wiki_link = true;
                markdown::Event::Text("".into())
            } else if text.as_ref() == "]" && wiki_link {
                wiki_link = false;
                let page_name = wiki_link_text.to_lowercase().replace(" ", "_");
                let link_text = wiki_link_text.clone();
                wiki_link_text.clear();
                let link_class = if wiki_page_names().contains(&page_name) {
                    ""
                } else {
                    "new"
                };
                markdown::Event::Html(
                    format!(
                        r#"<a href="/{}" class="{}">{}</a>"#,
                        page_name, link_class, link_text
                    )
                    .into(),
                )
            } else if wiki_link {
                wiki_link_text.push_str(&text);
                markdown::Event::Text("".into())
            } else {
                let linked = autolink::auto_link(&text, &[]);
                if linked.len() == text.len() {
                    markdown::Event::Text(text)
                } else {
                    markdown::Event::Html(linked.into())
                }
            }
        }
        _ => event,
    });

    let mut html_output = String::with_capacity(md.len() * 3 / 2);
    markdown::html::push_html(&mut html_output, parser);
    html_output
}

/// Returns a path on disk to a new wiki page.
/// Nothing if the page already exists.
fn new_wiki_path(path: &str) -> Option<String> {
    if wiki_path(path).is_none() {
        Some(wiki_disk_path(path))
    } else {
        None
    }
}

/// Path of wiki page on disk, if it exists.
/// Always in the `wiki/` directory.
/// Eg. wiki_path("Welcome") -> "wiki/welcome.md"
fn wiki_path(path: &str) -> Option<String> {
    let path = wiki_disk_path(path);
    if Path::new(&path).exists() {
        Some(path)
    } else {
        None
    }
}

/// Returns a wiki path on disk, regardless of whether it exists or
/// not already.
fn wiki_disk_path(path: &str) -> String {
    format!(
        "./wiki/{}.md",
        path.to_lowercase()
            .trim_start_matches('/')
            .replace("..", ".")
    )
}
/// Is the file at the given path `chmod +x`?
fn is_executable(path: &str) -> bool {
    if let Ok(meta) = fs::metadata(path) {
        meta.permissions().mode() & 0o111 != 0
    } else {
        false
    }
}

/// Run a script and return its output.
fn shell(path: &str, args: &[&str]) -> Result<String, io::Error> {
    let output = Command::new(path).args(args).output()?;
    let out = if output.status.success() {
        output.stdout
    } else {
        output.stderr
    };
    match str::from_utf8(&out) {
        Ok(s) => Ok(s.to_string()),
        Err(e) => Err(io::Error::new(io::ErrorKind::Other, e.to_string())),
    }
}

/// Path of asset on disk, if it exists.
/// Always in the `web/` directory.
/// Eg web_path("style.css") -> "web/style.css"
fn web_path(path: &str) -> Option<String> {
    let path = format!("./web/{}", path.trim_start_matches('/').replace("..", "."));
    if Path::new(&path).exists() {
        Some(path)
    } else {
        None
    }
}

fn web_to_string(path: &str) -> Option<String> {
    if let Some(disk_path) = web_path(path) {
        Some(fs::read_to_string(disk_path).unwrap_or_else(|e| e.to_string()))
    } else {
        None
    }
}

/// Content type for a file on disk. We only look in `web/`.
fn get_content_type(path: &str) -> Option<&'static str> {
    let disk_path = web_path(path);
    if disk_path.is_none() {
        return None;
    }
    let disk_path = disk_path.unwrap();
    let path = Path::new(&disk_path);
    let extension = match path.extension() {
        None => return Some("text/plain"),
        Some(e) => e,
    };

    Some(match extension.to_str().unwrap() {
        "gif" => "image/gif",
        "jpg" => "image/jpeg",
        "jpeg" => "image/jpeg",
        "png" => "image/png",
        "pdf" => "application/pdf",
        "css" => "text/css; charset=utf8",
        "htm" => "text/html; charset=utf8",
        "html" => "text/html; charset=utf8",
        "txt" => "text/plain; charset=utf8",
        _ => "text/plain; charset=utf8",
    })
}

/// Does what it says.
fn decode_form_value(post: &str) -> String {
    percent_decode(post.as_bytes())
        .decode_utf8_lossy()
        .replace('+', " ")
}