use {crate::Page, std::io, tinytemplate::TinyTemplate, vial::asset};

#[derive(Serialize)]
pub struct Jump {
    pages: Vec<Page>,
}

impl Jump {
    pub fn new(pages: Vec<Page>) -> Jump {
        Jump { pages }
    }

    pub fn to_string(&self) -> Result<String, io::Error> {
        let mut tt = TinyTemplate::new();
        let index = asset::to_string("html/jump.html")?;

        tt.add_template("index", &index)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        tt.render("index", &self)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}
