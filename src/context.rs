use crate::page::Page;

pub struct Context<'p> {
    page: &'p Page,
}

tenjin::context! {
    self: ('p) Context<'p> {
        page => self.page,
        markdown => @raw crate::markdown::to_html(&self.page.body(), &[]).as_str(),
    }
}

impl<'p> Context<'p> {
    pub fn new(page: &'p Page) -> Context<'p> {
        Context { page }
    }
}
