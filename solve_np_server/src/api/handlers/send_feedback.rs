use crate::mailer::FROM_EMAIL;
use crate::mailer::TO_EMAIL;
use crate::state::State;
use actix_web::Responder;
use actix_web::web;
use lettre::AsyncTransport;
use lettre::message::Mailbox;
use lettre::message::Message;
use lettre::message::SinglePart;
use log::error;
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
) -> impl Responder {
    match send_feedback_helper(state, form).await {
        Ok(()) => web::Redirect::to("/feedback?status=success").see_other(),
        Err(msg) => web::Redirect::to(format!("/feedback?status={msg}")).see_other(),
    }
}

async fn send_feedback_helper(
    state: web::Data<State>,
    form: web::Form<FeedbackForm>,
) -> Result<(), String> {
    let FeedbackForm {
        title,
        content,
        email,
    } = form.into_inner();
    let content = format!(
        "{content}\n\nFrom: {}\n",
        email
            .map(|maybe_empty| if maybe_empty.is_empty() {
                String::from("anonymous")
            } else {
                maybe_empty
            })
            .unwrap_or("anonymous".into())
    );
    let email = Message::builder()
        .from(
            FROM_EMAIL
                .parse::<Mailbox>()
                .expect("Failed parsing the FROM_EMAIL"),
        )
        .to(TO_EMAIL
            .parse::<Mailbox>()
            .expect("Failed parsing the TO_EMAIL"))
        .subject(title)
        .singlepart(SinglePart::plain(content))
        .map_err(|_| "bad content".to_string())?;
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
                .inspect_err(|err| error!("Failed sending feedback due to: {err}",))
                .map_err(|_| "failed sending the feedback to feedback@solvenp.now".to_string())?;
        }
    };
    Ok(())
}
