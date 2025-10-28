use askama::Template;

#[derive(Template)]
#[template(path = "head.html")]
pub struct HeadTemplate {
    pub title: String,
}
