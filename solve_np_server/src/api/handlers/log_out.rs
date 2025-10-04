use actix_session::Session;
use actix_web::Responder;
use actix_web::web;

pub async fn log_out(session: Session) -> impl Responder {
    session.clear();
    web::Redirect::to("/").see_other()
}
