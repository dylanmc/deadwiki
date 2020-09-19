use {crate::markdown, std::fs};

tenjin::context! {
    self: ('p) Context<'p> {
        page => self.page,
        path => self.page.path.as_str(),
        root => self.page.root.as_str(),
        name => self.page.name(),
        url => self.page.url().as_str(),
        title => self.page.title().as_str(),
        markdown => @raw &markdown(&self.page.body())[..],
    }

    self: Page {
        path => self.path.as_str(),
        root => self.root.as_str(),
        name => self.name(),
        url => self.url().as_str(),
        title => self.title().as_str(),
        body => self.body().as_str(),
    }
}

pub struct Context<'p> {
    page: &'p Page,
}

impl<'p> Context<'p> {
    pub fn new(page: &'p Page) -> Context<'p> {
        Context { page }
    }
}

/// Single Wiki Page
pub struct Page {
    path: String,
    root: String,
}

impl Page {
    pub fn new<S: AsRef<str>, T: AsRef<str>>(root: S, path: T) -> Page {
        Page {
            root: root.as_ref().into(),
            path: path.as_ref().into(),
        }
    }

    pub fn name(&self) -> &str {
        self.path_without_root().trim_end_matches(".md")
    }

    pub fn url(&self) -> String {
        format!("/{}", self.name())
    }

    pub fn path(&self) -> &str {
        &self.path
    }

    pub fn body(&self) -> String {
        fs::read_to_string(&self.path()).unwrap_or_else(|_| "".into())
    }

    pub fn path_without_root(&self) -> &str {
        self.path
            .trim_start_matches(&self.root)
            .trim_start_matches('.')
            .trim_start_matches('/')
    }

    pub fn title(&self) -> String {
        self.name()
            .split('_')
            .map(|part| {
                if part.contains('/') {
                    let mut parts = part.split('/').rev();
                    let last = parts.next().unwrap_or("?");
                    format!(
                        "{}/{}",
                        parts.rev().collect::<Vec<_>>().join("/"),
                        capitalize(last)
                    )
                } else {
                    capitalize(&part)
                }
            })
            .collect::<Vec<_>>()
            .join(" ")
    }
}

/// Capitalize the first letter of a string.
fn capitalize(s: &str) -> String {
    format!(
        "{}{}",
        s.chars().next().unwrap_or('?').to_uppercase(),
        &s.chars().skip(1).collect::<String>()
    )
}

// Shortcut
fn markdown(txt: &str) -> String {
    markdown::to_html(txt, &[])
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_name() {
        let page = Page::new("./wiki", "./wiki/info.md");
        assert_eq!(page.name(), "info");
        assert_eq!(page.title(), "Info");
        assert_eq!(page.url(), "/info");
        assert_eq!(page.path, "./wiki/info.md");

        let page = Page::new("./wiki", "./wiki/linux_laptops.md");
        assert_eq!(page.name(), "linux_laptops");
        assert_eq!(page.title(), "Linux Laptops");
        assert_eq!(page.url(), "/linux_laptops");
        assert_eq!(page.path, "./wiki/linux_laptops.md");
    }
}
