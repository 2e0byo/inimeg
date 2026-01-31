use crate::{
    cli,
    request::{Request, RequestError},
    response::{ErrResponse, Response, SuccessResponse},
    status::*,
};
use rustls::{
    ServerConfig, ServerConnection, Stream,
    pki_types::{CertificateDer, PrivateKeyDer, pem::PemObject},
};
use std::{
    io::{BufRead, Read},
    net::{TcpListener, TcpStream},
    str::FromStr,
    sync::Arc,
};

use log::warn;

type TlsStream<'a> = Stream<'a, ServerConnection, TcpStream>;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("IO failed: `{0:?}`")]
    IO(#[from] std::io::Error),
    #[error("Invalid request: `{0:?}`")]
    Request(#[from] RequestError),
    #[error("Request not utf8: `{0:?}`")]
    Utf8(#[from] std::string::FromUtf8Error),
}

type Result<T> = std::result::Result<T, Error>;

pub struct Server {
    config: Arc<ServerConfig>,
    listener: TcpListener,
}

fn parse_raw_request<'a>(stream: &mut TlsStream<'a>) -> Result<String> {
    let mut buf = Vec::with_capacity(1026);
    stream.take(1026).read_until(b'\n', &mut buf)?;
    Ok(String::try_from(buf)?)
}

impl Server {
    pub fn new(
        cert: CertificateDer<'static>,
        key: PrivateKeyDer<'static>,
        port: usize,
    ) -> anyhow::Result<Self> {
        let config = ServerConfig::builder()
            .with_no_client_auth() // TODO
            .with_single_cert(vec![cert], key)?;

        let listener = TcpListener::bind(format!("[::]:{port}"))?;

        Ok(Self {
            config: config.into(),
            listener,
        })
    }

    fn handle_request(&self, request: &str) -> Result<Response> {
        let request = Request::from_str(request)?;
        dbg!(&request);
        Ok(Response::Fixed(SuccessResponse {
            status: Success::Generic,
            mime: "text/plain".into(),
            body: "Hello world".into(),
        }))
    }

    fn handle<'a>(&self, mut stream: Stream<'a, ServerConnection, TcpStream>) {
        let resp: Response = parse_raw_request(&mut stream)
            .and_then(|raw| self.handle_request(raw.as_ref()))
            .or_else(|e| -> std::result::Result<Response, ()> {
                match e {
                    Error::IO(_) => Ok(Response::Err(ErrResponse::from_status(
                        Status::TemporaryFailure(TemporaryFailure::Generic),
                    ))),
                    Error::Utf8(e) => Ok(Response::Err(ErrResponse {
                        status: Status::PermanentFailure(PermanentFailure::BadRequest),
                        msg: Some(format!("{e:?}").replace("\n", " ").into()), // HACK
                    })),
                    Error::Request(ref e) => Ok(Response::Err(ErrResponse::from_status(e.into()))),
                }
            })
            .unwrap(); // E is (), so this is safe
        let _ = resp
            .send(stream)
            .inspect_err(|e| warn!("Failed to send response: {e:?}"));
    }

    pub fn run(self) -> anyhow::Result<()> {
        loop {
            let (mut tcp_stream, _) = self.listener.accept()?;
            let mut conn = ServerConnection::new(self.config.clone())?;
            let tls_stream = Stream::new(&mut conn, &mut tcp_stream);
            self.handle(tls_stream);
        }
    }
}

impl TryFrom<cli::Serve> for Server {
    type Error = anyhow::Error;
    fn try_from(value: cli::Serve) -> std::result::Result<Self, Self::Error> {
        Self::new(
            CertificateDer::from_pem_file(value.certificate).expect("certificate"),
            PrivateKeyDer::from_pem_file(value.private_key).expect("certificate"),
            value.port,
        )
    }
}
