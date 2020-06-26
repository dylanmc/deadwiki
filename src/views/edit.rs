use {std::io, tinytemplate::TinyTemplate, vial::asset};

#[derive(Serialize)]
pub struct Edit {
    markdown: String,
}

impl Edit {
    pub fn new(markdown: String) -> Edit {
        Edit { markdown }
    }

    /// Render the index page which lists all wiki pages.
    pub fn to_string(&self) -> Result<String, io::Error> {
        let mut tt = TinyTemplate::new();
        let index = asset::to_string("html/edit.html")?;

        tt.add_template("edit", &index)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        tt.render("edit", &self)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}
