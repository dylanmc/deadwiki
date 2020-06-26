use {std::io, tinytemplate::TinyTemplate, vial::asset};

#[derive(Serialize)]
pub struct Layout<'env> {
    title: &'env str,
    body: &'env str,
    webview_app: &'env str,
    pages_json: String,
    nav: &'env str,
}

impl<'env> Layout<'env> {
    pub fn new(
        title: &'env str,
        body: &'env str,
        webview_app: &'env str,
        pages_json: String,
        nav: &'env str,
    ) -> Layout<'env> {
        Layout {
            title,
            body,
            webview_app,
            pages_json,
            nav,
        }
    }

    /// Render the index page which lists all wiki pages.
    pub fn to_string(&self) -> Result<String, io::Error> {
        let mut tt = TinyTemplate::new();
        let index = asset::to_string("html/layout.html")?;

        tt.add_template("layout", &index)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))?;

        tt.render("layout", &self)
            .map_err(|e| io::Error::new(io::ErrorKind::Other, e))
    }
}
