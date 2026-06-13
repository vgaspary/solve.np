use crate::frontend::handlers::head::HeadTemplate;
use crate::frontend::handlers::header::HeaderTemplate;
use crate::state::State;
use actix_session::SessionExt;
use actix_web::dev::ServiceResponse;
use actix_web::middleware::ErrorHandlerResponse;
use actix_web::web;
use askama::Template;

#[derive(Template)]
#[template(path = "error.html")]
struct ErrorTemplate {
    head: HeadTemplate,
    header: HeaderTemplate,
    report_link: String,
}

pub fn handle_internal_error<B>(
    res: ServiceResponse<B>,
) -> actix_web::Result<ErrorHandlerResponse<B>> {
    let title = "500: Internal Server Error";
    let error_message = res
        .response()
        .error()
        .map_or("Unknown internal error.".into(), |e| e.to_string());
    let report_link = format!("/feedback?title={title}&content={error_message}");

    let req = res.request();
    let session = req.get_session();
    let state = req.app_data::<web::Data<State>>().cloned();

    let head = HeadTemplate::with_title("Error - solve.NP");
    let header = state
        .and_then(|state| HeaderTemplate::try_from(&session, state).ok())
        .unwrap_or_default();

    let body = ErrorTemplate {
        head,
        header,
        report_link,
    }
    .render()
    .unwrap();

    let (req, res) = res.into_parts();
    let res = res.set_body(body);
    let mut res = ServiceResponse::new(req, res)
        .map_into_boxed_body()
        .map_into_right_body();

    res.response_mut().headers_mut().insert(
        actix_web::http::header::CACHE_CONTROL,
        actix_web::http::header::HeaderValue::from_static("no-store"),
    );
    res.response_mut().headers_mut().insert(
        actix_web::http::header::CONTENT_TYPE,
        actix_web::http::header::HeaderValue::from_static("text/html"),
    );

    Ok(ErrorHandlerResponse::Response(res))
}
