use {std::io, tinytemplate::TinyTemplate, vial::asset};

#[derive(Serialize)]
pub struct New<'name> {
    name: &'name str,
}

impl<'name> New<'name> {
    pub fn new(name: &'name str) -> New<'name> {
        New { name }
    }

    /// Render the index page which lists all wiki pages.
    pub fn to_string(&self) -> Result<String, io::Error> {
        let mut tt = TinyTemplate::new();
        let index = asset::to_string("html/new.html")?;

        tt.add_template("new", &index)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        tt.render("new", &self)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}
