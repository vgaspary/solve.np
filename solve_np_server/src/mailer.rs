use lettre::Tokio1Executor;
use lettre::transport::smtp::AsyncSmtpTransport;
use lettre::transport::smtp::authentication::Credentials;
use lettre::transport::smtp::client::Tls;
use lettre::transport::smtp::client::TlsParameters;

const SMTP_HOST: &str = "smtp.gmail.com";
const SMTP_PORT: u16 = 587;
const SMTP_USER: &str = "server_for_feedback@solvenp.now";

pub const FROM_EMAIL: &str = "server_for_feedback@solvenp.now";
pub const TO_EMAIL: &str = "feedback@solvenp.now";

pub fn create_mailer(smtp_pass: &str) -> anyhow::Result<AsyncSmtpTransport<Tokio1Executor>> {
    let creds = Credentials::new(SMTP_USER.into(), smtp_pass.into());
    let tls_parameters = TlsParameters::new(SMTP_HOST.to_string())?;
    Ok(AsyncSmtpTransport::<Tokio1Executor>::relay(SMTP_HOST)?
        .port(SMTP_PORT)
        .tls(Tls::Required(tls_parameters))
        .credentials(creds)
        .build())
}
