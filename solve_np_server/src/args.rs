use crate::auth::gauth::GoogleAuthClientId;
use crate::auth::gauth::GoogleAuthClientSecret;
use crate::auth::gauth::GooglePublicKeys;
use crate::auth::github_auth::GithubAuthClientId;
use crate::auth::github_auth::GithubAuthClientSecret;
use crate::auth::session_registry::SessionRegistry;
use crate::mailer::create_mailer;
use crate::state::LazyStaticData;
use crate::state::State;
use crate::state::StaticData;
use clap::Parser;
use log::error;
use rusqlite::Connection;
use rustls::ServerConfig;
use std::fs::File;
use std::io::BufReader;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::Duration;
use std::time::SystemTime;

#[derive(Parser)]
pub struct Args {
    /// Static data path.
    #[arg(long, default_value = "static_data.yaml")]
    pub static_data_path: PathBuf,

    /// Session cookie key.
    #[arg(long)]
    pub key: String,

    /// DB path.
    #[arg(long, default_value = "solve_np.db3")]
    pub db_path: PathBuf,

    /// Address.
    #[arg(long, default_value = "http://127.0.0.1:3333")]
    pub address: String,

    /// Port.
    #[arg(long, default_value_t = 3333)]
    pub port: u16,

    /// Ranking update period.
    #[arg(long, value_parser = humantime::parse_duration, default_value = "5m")]
    pub rerank_period: Duration,

    /// Session store.
    #[arg(long, default_value = "sessions.yaml")]
    pub session_store: PathBuf,

    /// Session cleanup and saving period.
    #[arg(long, value_parser = humantime::parse_duration, default_value = "10s")]
    pub session_cleanup_period: Duration,

    /// SMTP password.
    #[arg(long)]
    pub smtp_pass: Option<String>,

    #[command(flatten)]
    pub auth_args: AuthArgs,

    #[command(flatten)]
    pub tls_args: TlsArgs,
}

#[derive(clap::Args)]
#[group(id = "auth", required = false, multiple = true)]
pub struct AuthArgs {
    /// Google Auth Client ID.
    #[arg(long, requires = "auth")]
    pub google_auth_client_id: Option<String>,

    /// Google Auth Client secret.
    #[arg(long, requires = "auth")]
    pub google_auth_client_secret: Option<String>,

    /// GitHub Auth Client ID.
    #[arg(long, requires = "auth")]
    pub github_auth_client_id: Option<String>,

    /// GitHub Auth Client secret.
    #[arg(long, requires = "auth")]
    pub github_auth_client_secret: Option<String>,
}

#[derive(clap::Args)]
#[group(id = "tls", required = false, multiple = true)]
pub struct TlsArgs {
    /// Cert file path.
    #[arg(long, requires = "tls")]
    pub tls_cert: Option<PathBuf>,

    /// Key file path.
    #[arg(long, requires = "tls")]
    pub tls_key: Option<PathBuf>,
}

impl TryFrom<&Args> for State {
    type Error = std::io::Error;

    fn try_from(args: &Args) -> Result<Self, Self::Error> {
        let start_time = SystemTime::now();
        let static_data: StaticData = serde_yaml::from_reader::<_, LazyStaticData>(BufReader::new(
            File::open(&args.static_data_path)?,
        ))
        .map_err(std::io::Error::other)?
        .into();
        let db = Mutex::new(Connection::open(&args.db_path).map_err(std::io::Error::other)?);
        let address = args.address.clone();
        let session_registry = SessionRegistry::load_from_file_or_default(&args.session_store);
        let google_auth_client_id = args
            .auth_args
            .google_auth_client_id
            .clone()
            .map(GoogleAuthClientId);
        let google_auth_client_secret = args
            .auth_args
            .google_auth_client_secret
            .clone()
            .map(GoogleAuthClientSecret);
        let github_auth_client_id = args
            .auth_args
            .github_auth_client_id
            .clone()
            .map(GithubAuthClientId);
        let github_auth_client_secret = args
            .auth_args
            .github_auth_client_secret
            .clone()
            .map(GithubAuthClientSecret);
        let mail_client = args.smtp_pass.as_ref().and_then(|smtp_pass| {
            create_mailer(&smtp_pass)
                .inspect_err(|err| error!("{err}"))
                .ok()
        });
        Ok(Self {
            start_time,
            static_data,
            db,
            address,
            session_registry,
            google_auth_client_id,
            google_auth_client_secret,
            google_public_keys: GooglePublicKeys::new(),
            github_auth_client_id,
            github_auth_client_secret,
            mail_client,
        })
    }
}

pub fn server_config_from_tls_args(tls_args: &TlsArgs) -> Option<ServerConfig> {
    let mut key_file =
        BufReader::new(File::open(tls_args.tls_key.as_ref()?).expect("Open privatekey file"));
    let pkey = rustls_pemfile::private_key(&mut key_file).expect("Private key")?;
    let mut cert_file =
        BufReader::new(File::open(tls_args.tls_cert.as_ref()?).expect("Open cert file"));
    let cert_chain = rustls_pemfile::certs(&mut cert_file)
        .collect::<Result<_, _>>()
        .expect("Extracting certificates");
    let tls_config = ServerConfig::builder()
        .with_no_client_auth()
        .with_single_cert(cert_chain, pkey)
        .inspect_err(|e| error!("{e}"))
        .unwrap();
    Some(tls_config)
}
