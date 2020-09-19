//! Rendering "logic".

use {
    crate::{helper::*, markdown, render, Page},
    std::{fs, io, str},
    vial::asset,
};

/// Renders a chunk of HTML surrounded by `static/html/layout.html`.
pub fn layout<T, S>(title: T, body: S, nav: Option<&str>) -> io::Result<String>
where
    T: AsRef<str>,
    S: AsRef<str>,
{
    let title = title.as_ref();
    let body = body.as_ref();
    let mut webview_app = "";
    if cfg!(feature = "gui") {
        webview_app = "webview-app";
    }

    Ok(if asset::exists("html/layout.html") {
        asset::to_string("html/layout.html")?
            .replace("{title}", title)
            .replace("{body}", body)
            .replace("{webview-app}", webview_app)
            .replace("{nav}", nav.unwrap_or(""))
    } else {
        body.to_string()
    })
}
