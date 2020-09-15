//! (Method, URL) => Code

use {
    crate::{db::ReqWithDB, markdown},
    std::{collections::HashMap, io, ops},
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

fn search(req: Request) -> Result<impl Responder, io::Error> {
    if let Some(tag) = req.query("tag") {
        let mut env = Env::new();
        env.set("tag", tag);
        env.set("pages", req.db().find_pages_with_tag(tag)?);
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
fn index(req: Request) -> Result<impl Responder, io::Error> {
    let mut env = Env::new();
    env.set("pages", req.db().pages()?);
    render("deadwiki", env.render("html/index.hat")?)
}

// POST new page
fn create(req: Request) -> Result<impl Responder, io::Error> {
    let name = req.form("name").unwrap_or("note.md");
    let page = req.db().create(name, req.form("markdown").unwrap_or(""))?;
    Ok(Response::redirect_to(page.url()))
}

// Recently modified wiki pages.
fn recent(req: Request) -> Result<impl Responder, io::Error> {
    let mut env = Env::new();
    env.set("pages", req.db().recent()?);
    render("Recently Modified Pages", env.render("html/list.hat")?)
}

fn jump(req: Request) -> Result<impl Responder, io::Error> {
    let mut env = Env::new();

    let pages = req.db().pages()?;
    let pages = pages.iter().enumerate().map(|(i, p)| {
        let mut map: HashMap<String, hatter::Value> = HashMap::new();
        map.insert("id".into(), i.into());
        map.insert("name".into(), p.title().into());
        map.insert("url".into(), p.url().into());
        map
    });

    let mut idx = pages.len();
    let tags = req.db().tags()?;
    let tags = tags.iter().enumerate().map(|(i, tag)| {
        let mut map: HashMap<String, hatter::Value> = HashMap::new();
        map.insert("id".into(), (idx + i).into());
        map.insert("name".into(), tag.into());
        map.insert("url".into(), tag.into());
        idx += 1;
        map
    });

    env.set("pages", pages.chain(tags).collect::<Vec<_>>());
    render("Jump to Wiki Page", env.render("html/jump.hat")?)
}

fn update(req: Request) -> Result<impl Responder, io::Error> {
    if let Some(name) = req.arg("name") {
        let page = req.db().update(name, req.form("markdown").unwrap_or(""))?;
        Ok(Response::redirect_to(page.url()))
    } else {
        Ok(Response::from(404))
    }
}

fn edit(req: Request) -> Result<impl Responder, io::Error> {
    let mut env = Env::new();
    if let Some(name) = req.arg("name") {
        if let Some(page) = req.db().find(name) {
            env.set("page", page);
            return render("Edit", env.render("html/edit.hat")?);
        }
    }
    Ok(response_404())
}

fn show(req: Request) -> Result<impl Responder, io::Error> {
    if let Some(name) = req.arg("name") {
        // let raw = name.ends_with(".md");
        if let Some(page) = req.db().find(name.trim_end_matches(".md")) {
            let mut env = Env::new();
            let title = page.title().clone();
            let names = req.db().names()?;
            env.set("page", page);
            env.helper("markdown", move |_, args| {
                let src = args.get(0).map(|a| a.to_str()).unwrap_or("");
                markdown::to_html(&src, &names).into()
            });
            return render(&title, env.render("html/show.hat")?);
        }
    }
    Ok(response_404())
}

fn response_404() -> Response {
    Response::from(404).with_asset("html/404.hat")
}

fn render<S: AsRef<str>>(title: &str, body: S) -> Result<Response, io::Error> {
    let mut env = Env::new();
    env.set("title", title);
    env.set("body", body.as_ref());
    env.set("webview-app?", false);
    #[cfg(feature = "gui")]
    env.set("webview-app?", true);
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
