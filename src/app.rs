//! (Method, URL) => Code

use {
    crate::{helper::*, wiki_root},
    atomicwrites::{AllowOverwrite, AtomicFile},
    std::{
        collections::HashMap,
        fs,
        io::{self, Write},
        ops,
        path::Path,
    },
    vial::prelude::*,
};

routes! {
    GET "/" => index;

    GET "/jump" => jump;
    GET "/recent" => recent;

    GET "/new" => new;
    POST "/new" => create;

    GET "/search" => search;

    GET "/edit/*name" => edit;
    POST "/edit/*name" => update;
    GET "/*name" => show;
}

// Don't include the '#' when you search. Pass in "hashtag" to
// search for #hashtag.
fn pages_with_tag(tag: &str) -> Result<Vec<String>, io::Error> {
    let tag = if tag.starts_with('#') {
        tag.to_string()
    } else {
        format!("#{}", tag)
    };

    let out = shell!("grep --exclude-dir .git -l -r '{}' {}", tag, wiki_root())?;
    Ok(out
        .split("\n")
        .filter_map(|line| {
            if !line.is_empty() {
                Some(
                    line.split(':')
                        .next()
                        .unwrap_or("?")
                        .trim_end_matches(".md")
                        .trim_start_matches(&wiki_root())
                        .trim_start_matches('/')
                        .to_string(),
                )
            } else {
                None
            }
        })
        .collect::<Vec<_>>())
}

fn search(req: Request) -> Result<impl Responder, io::Error> {
    if let Some(tag) = req.query("tag") {
        let mut env = Env::new();
        env.set("tag", tag);
        env.set("pages", pages_with_tag(tag)?);
        render("Search", env.render("html/search.hat")?)
    } else {
        Ok(Response::from(404))
    }
}

fn new(req: Request) -> Result<impl Responder, io::Error> {
    let mut env = Env::new();
    env.set("name", req.query("name"));
    render("New Page", env.render("html/new.hat")?)
}

/// Render the index page which lists all wiki pages.
pub fn index(_req: Request) -> Result<impl Responder, io::Error> {
    let mut env = Env::new();
    env.helper("page_url", |_, args| format!("/{}", args[0]).into());
    env.helper("page_title", |_, args| {
        wiki_path_to_title(&args[0].to_string()).into()
    });

    env.set("pages", page_names());
    render("deadwiki", env.render("html/index.hat")?)
}

fn create(req: Request) -> Result<impl Responder, io::Error> {
    let path = pathify(&req.form("name").unwrap_or(""));
    if !page_names().contains(&path) {
        if let Some(disk_path) = new_page_path(&path) {
            if disk_path.contains('/') {
                if let Some(dir) = Path::new(&disk_path).parent() {
                    fs::create_dir_all(&dir.display().to_string())?;
                }
            }
            let mut file = fs::File::create(disk_path)?;
            return if let Some(mdown) = req.form("markdown") {
                write!(file, "{}", mdown)?;
                Ok(Response::redirect_to(path))
            } else {
                Ok(Response::redirect_to("/new"))
            };
        }
    }
    Ok(response_404())
}

// Recently modified wiki pages.
fn recent(_: Request) -> Result<impl Responder, io::Error> {
    let out = shell!(
        r#"git --git-dir={}/.git log --pretty=format: --name-only -n 30 | grep "\.md\$""#,
        wiki_root()
    )?;
    let mut pages = vec![];
    let mut seen = HashMap::new();
    for page in out.split("\n") {
        if seen.get(page).is_some() || page == ".md" || page.is_empty() {
            // TODO: .md hack
            continue;
        } else {
            pages.push(page);
            seen.insert(page, true);
        }
    }

    let mut env = Env::new();
    env.set("pages", pages);
    render("Recently Modified Pages", env.render("html/list.hat")?)
}

fn jump(_: Request) -> Result<impl Responder, io::Error> {
    let mut env = Env::new();
    env.set(
        "pages",
        page_names()
            .iter()
            .chain(tag_names().iter())
            .collect::<Vec<_>>(),
    );
    render("Jump to Wiki Page", env.render("html/jump.hat")?)
}

fn update(req: Request) -> Result<impl Responder, io::Error> {
    if let Some(name) = req.arg("name") {
        if let Some(disk_path) = page_path(name) {
            let mdown = req.form("markdown").unwrap_or("");
            let af = AtomicFile::new(disk_path, AllowOverwrite);
            af.write(|f| f.write_all(mdown.as_bytes()))?;
            return Ok(Response::redirect_to(format!(
                "/{}",
                pathify(name).replace("edit/", "")
            )));
        }
    }
    Ok(response_404())
}

fn edit(req: Request) -> Result<impl Responder, io::Error> {
    let mut env = Env::new();
    if let Some(name) = req.arg("name") {
        if let Some(disk_path) = page_path(name) {
            env.set("name", name);
            env.set("markdown", &fs::read_to_string(disk_path)?);
            return render("Edit", env.render("html/edit.hat")?);
        }
    }
    Ok(response_404())
}

fn show(req: Request) -> Result<impl Responder, io::Error> {
    if let Some(name) = req.arg("name") {
        if let Some(_disk_path) = page_path(name) {
            let mut env = Env::new();
            env.set("page", wiki_page(name));
            return render(name, env.render("html/show.hat")?);
        }
    }
    Ok(response_404())
}

fn response_404() -> Response {
    Response::from(404).with_asset("html/404.hat")
}

fn wiki_page(name: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    map.insert("title".into(), name.into());
    map.insert("body".into(), "coming soon".into());
    map
}

fn render<S: AsRef<str>>(title: &str, body: S) -> Result<Response, io::Error> {
    let mut env = Env::new();
    env.set("title", title);
    env.set("body", body.as_ref());
    Ok(Response::from(env.render("html/layout.hat")?))
}

struct Env {
    vm: hatter::VM,
}
impl ops::Deref for Env {
    type Target = hatter::VM;
    fn deref(&self) -> &Self::Target {
        &self.vm
    }
}
impl ops::DerefMut for Env {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.vm
    }
}
impl Env {
    fn new() -> Env {
        Env {
            vm: hatter::VM::new(false),
        }
    }
    fn render(&mut self, path: &str) -> Result<String, io::Error> {
        Ok(self.vm.render(asset::to_string(path)?).unwrap())
    }
}
