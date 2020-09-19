use {
    crate::db::DB,
    std::{
        io::{self, BufWriter},
        sync::{Arc, RwLock},
    },
    tenjin::Tenjin,
    vial::asset,
};

/// Shared state for web requests.
pub struct State {
    db: DB,
    tpl: Arc<RwLock<Tenjin>>,
}

/// Shortcut trait.
pub trait ReqState {
    fn db(&self) -> &DB;
    fn tpl(&self) -> Arc<RwLock<Tenjin>>;
    fn render<C>(&self, name: &str, data: &C) -> io::Result<String>
    where
        C: tenjin::Context<BufWriter<Vec<u8>>>;
}

impl ReqState for vial::Request {
    fn db(&self) -> &DB {
        self.state::<State>().db()
    }
    fn tpl(&self) -> Arc<RwLock<Tenjin>> {
        self.state::<State>().tpl()
    }
    fn render<C>(&self, name: &str, data: &C) -> io::Result<String>
    where
        C: tenjin::Context<BufWriter<Vec<u8>>>,
    {
        self.state::<State>().render(name, data)
    }
}

pub fn new(db: DB) -> State {
    State::new(db)
}

impl State {
    pub fn new(db: DB) -> State {
        State {
            db,
            tpl: Arc::new(RwLock::new(Tenjin::empty())),
        }
    }

    pub fn db(&self) -> &DB {
        &self.db
    }

    pub fn tpl(&self) -> Arc<RwLock<Tenjin>> {
        self.tpl.clone()
    }

    pub fn render<C>(&self, name: &str, data: &C) -> io::Result<String>
    where
        C: tenjin::Context<BufWriter<Vec<u8>>>,
    {
        self.render_with_layout("layout", name, data)
    }

    pub fn render_with_layout<C>(&self, layout: &str, name: &str, data: &C) -> io::Result<String>
    where
        C: tenjin::Context<BufWriter<Vec<u8>>>,
    {
        let combined = &asset::to_string(&format!("html/{}.html", layout))?
            .replace("{body}", &asset::to_string(&format!("html/{}.html", name))?);
        let mut tpl = self.tpl.write().unwrap();
        tpl.register(name, tenjin::Template::compile(combined).unwrap());
        let template = tpl.get(name).unwrap();
        let mut out = BufWriter::new(Vec::new());
        tpl.render(template, &data, &mut out).unwrap();
        let bytes = out.into_inner()?;
        Ok(String::from_utf8(bytes).unwrap())
    }

    pub fn render_without_layout<C>(&self, name: &str, data: &C) -> io::Result<String>
    where
        C: tenjin::Context<BufWriter<Vec<u8>>>,
    {
        let mut tpl = self.tpl.write().unwrap();
        tpl.register(
            name,
            tenjin::Template::compile(&asset::to_string(&format!("html/{}.html", name))?).unwrap(),
        );
        let template = tpl.get(name).unwrap();
        let mut out = BufWriter::new(Vec::new());
        tpl.render(template, &data, &mut out).unwrap();
        let bytes = out.into_inner()?;
        Ok(String::from_utf8(bytes).unwrap())
    }
}
