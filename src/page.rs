// Wiki Page
use crate::helper::wiki_path_to_title;

#[derive(Serialize)]
pub struct Page {
    name: String,
    title: String,
}

impl Page {
    pub fn new<S: AsRef<str>>(name: S) -> Page {
        let name = name.as_ref().to_string();
        Page {
            title: wiki_path_to_title(&name),
            name,
        }
    }
}
