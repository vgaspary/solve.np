use crate::mailer::FROM_EMAIL;
use crate::state::State;
use actix_web::Responder;
use actix_web::error::Error as HttpError;
use actix_web::error::ErrorInternalServerError;
use actix_web::web;
use lettre::AsyncTransport;
use lettre::message::Mailbox;
use lettre::message::Message;
use lettre::message::SinglePart;
use log::info;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct FeedbackForm {
    title: String,
    content: String,
    email: Option<String>,
}

pub async fn send_feedback(
    state: web::Data<State>,
    form: web::Form<FeedbackForm>,
) -> Result<impl Responder, HttpError> {
    let FeedbackForm {
        title,
        content,
        email,
    } = form.into_inner();
    let content = format!(
        "{content}\n\nFrom: {}\n",
        email.unwrap_or("anonymous".into())
    );
    let email = Message::builder()
        .from(
            FROM_EMAIL
                .parse::<Mailbox>()
                .map_err(ErrorInternalServerError)?,
        )
        .to("feedback@solvenp.now"
            .parse::<Mailbox>()
            .map_err(ErrorInternalServerError)?)
        .subject(title)
        .singlepart(SinglePart::plain(content))
        .map_err(ErrorInternalServerError)?;
    match state.mail_client.as_ref() {
        None => {
            info!(
                "Feedback received:\n{}",
                std::str::from_utf8(&email.formatted()).unwrap_or_default()
            );
        }
        Some(mail_client) => {
            mail_client
                .send(email)
                .await
                .map_err(ErrorInternalServerError)?;
        }
    };
    Ok(web::Redirect::to("/feedback").see_other())
}
