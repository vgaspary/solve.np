use actix_files::NamedFile;
use actix_web::Responder;

pub async fn robots_txt() -> impl Responder {
    NamedFile::open_async("./static/robots.txt").await
}

pub async fn sitemap_xml() -> impl Responder {
    NamedFile::open_async("./static/sitemap.xml").await
}
