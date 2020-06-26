use {crate::Page, std::io, tinytemplate::TinyTemplate, vial::asset};

#[derive(Serialize)]
pub struct Search<'env> {
    tag: &'env str,
    pages: Vec<Page>,
}

impl<'env> Search<'env> {
    pub fn new(tag: &'env str, pages: Vec<Page>) -> Search<'env> {
        Search { tag, pages }
    }

    /// Render the index page which lists all wiki pages.
    pub fn to_string(&self) -> Result<String, io::Error> {
        let mut tt = TinyTemplate::new();
        let index = asset::to_string("html/search.html")?;

        tt.add_template("search", &index)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        tt.render("search", &self)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}
