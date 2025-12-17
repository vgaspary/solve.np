use askama::Template;

#[derive(Template)]
#[template(path = "head.html")]
pub struct HeadTemplate {
    pub title: String,
}

impl HeadTemplate {
    pub fn with_title(title: &str) -> Self {
        Self {
            title: title.into(),
        }
    }
}
