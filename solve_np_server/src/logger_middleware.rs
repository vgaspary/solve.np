use crate::auth::SessionId;
use actix_session::SessionExt;
use actix_web::Error;
use actix_web::body::BodySize;
use actix_web::body::MessageBody;
use actix_web::dev::Service;
use actix_web::dev::ServiceRequest;
use actix_web::dev::ServiceResponse;
use actix_web::dev::Transform;
use log::info;
use std::future::Ready;
use std::future::ready;
use std::marker::PhantomData;
use std::rc::Rc;
use std::task::Context;
use std::task::Poll;
use std::time::Instant;

type LocalBoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + 'a>>;

pub struct RequestLogger;

impl<S, B> Transform<S, ServiceRequest> for RequestLogger
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type InitError = ();
    type Transform = RequestLoggerMiddleware<S, B>;
    type Future = Ready<Result<Self::Transform, Self::InitError>>;

    fn new_transform(&self, service: S) -> Self::Future {
        ready(Ok(RequestLoggerMiddleware::<S, B> {
            service: Rc::new(service),
            _phantom: PhantomData,
        }))
    }
}

pub struct RequestLoggerMiddleware<S, B> {
    service: Rc<S>,
    _phantom: PhantomData<B>,
}

impl<S, B> Service<ServiceRequest> for RequestLoggerMiddleware<S, B>
where
    S: Service<ServiceRequest, Response = ServiceResponse<B>, Error = Error> + 'static,
    B: MessageBody + 'static,
{
    type Response = ServiceResponse<B>;
    type Error = Error;
    type Future = LocalBoxFuture<'static, Result<Self::Response, Self::Error>>;

    fn poll_ready(&self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.service.poll_ready(cx)
    }

    fn call(&self, req: ServiceRequest) -> Self::Future {
        let service = self.service.clone();
        let start_time = Instant::now();

        let session_id = req
            .get_session()
            .get::<SessionId>("session_id")
            .ok()
            .flatten()
            .map(SessionId::short)
            .map(|id| id.to_string())
            .unwrap_or_else(|| "-".to_string());
        let session_id_padded = format!("{: <10}", session_id);

        let remote_addr = req
            .peer_addr()
            .map(|addr| addr.ip().to_string())
            .unwrap_or_else(|| "-".to_string());
        let remote_addr_padded = format!("{: <15}", remote_addr);

        let method = req.method().to_string();
        let path = req.uri().path().to_string();
        let version = format!("{:?}", req.version());
        let request_line = format!("{} {} {}", method, path, version);

        let referer = req
            .headers()
            .get("Referer")
            .and_then(|h| h.to_str().ok())
            .unwrap_or_else(|| "-")
            .to_string();

        let user_agent = req
            .headers()
            .get("User-Agent")
            .and_then(|h| h.to_str().ok())
            .unwrap_or_else(|| "-")
            .to_string();

        Box::pin(async move {
            let res = service.call(req).await?;

            let status = res.status();
            let response_size = match res.response().body().size() {
                BodySize::Sized(len) => len,
                _ => 0,
            };
            let response_size_padded = format!("{: <7}", response_size);

            let elapsed = start_time.elapsed();
            let response_time = format!("{:.6}", elapsed.as_secs_f64());
            let response_time_padded = format!("{: <10}", response_time);

            info!(
                "{} {} \"{}\" {: <3} {} \"{}\" \"{}\" {}",
                session_id_padded,
                remote_addr_padded,
                request_line,
                status.as_u16(),
                response_size_padded,
                referer,
                user_agent,
                response_time_padded,
            );

            Ok(res)
        })
    }
}
