use crate::page::Page;

tenjin::context! {
    self: Context {
        is_app => self.is_app,
        title => self.title,
    }

    self: ('p) PageContext<'p> {
        is_app => self.is_app,
        title => self.title.as_str(),
        page => self.page,
        markdown => @raw crate::markdown::to_html(&self.page.body(), &[]).as_str(),
    }
}

pub struct Context {
    title: &'static str,
    is_app: bool,
}

impl Context {
    pub fn new(title: &'static str) -> Context {
        Context {
            title,
            is_app: false,
        }
    }
}

pub struct PageContext<'p> {
    page: &'p Page,
    title: String,
    is_app: bool,
}

impl<'p> PageContext<'p> {
    pub fn new(title: String, page: &'p Page) -> PageContext<'p> {
        PageContext {
            title,
            page,
            is_app: false,
        }
    }
}
