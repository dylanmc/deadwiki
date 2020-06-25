use {crate::Page, std::io, tinytemplate::TinyTemplate, vial::prelude::*};

#[derive(Serialize)]
pub struct Index {
    pages: Vec<Page>,
    hide_hint: bool,
}

impl Index {
    pub fn new(pages: Vec<Page>) -> Index {
        Index {
            hide_hint: pages.is_empty(),
            pages,
        }
    }

    /// Render the index page which lists all wiki pages.
    pub fn to_string(&self) -> Result<String, io::Error> {
        let mut tt = TinyTemplate::new();
        let index = asset::to_string("html/index.html")?;

        tt.add_template("index", &index)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        tt.render("index", &self)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}
