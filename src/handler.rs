use std::{
    fs::File,
    ops::Deref,
    path::{Path, PathBuf},
};

use log::debug;

use crate::{
    request::Request,
    response::{ErrResponse, FileResponse, Response},
    status::{Status, Success},
};

pub trait Handler {
    fn handle_request(&mut self, request: &Request) -> Option<Response>;
}

#[derive(Debug)]
struct Prefix(String);

impl From<String> for Prefix {
    fn from(mut value: String) -> Self {
        if !value.starts_with('/') {
            value = format!("/{value}");
        }
        if !value.ends_with('/') {
            value.push('/');
        }
        Self(value)
    }
}

impl Deref for Prefix {
    type Target = String;
    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

#[derive(Debug)]
pub struct StaticHandler {
    /// The directory containing the content
    path: PathBuf,
    /// The prefix required in the url for this handler to match.
    prefix: Prefix,
}

#[derive(Debug, thiserror::Error)]
pub enum StaticHandlerError {
    #[error("Path is relative")]
    RelativePath,
    #[error("Path does not existt")]
    MissingPath,
}

impl StaticHandler {
    pub fn new(
        path: impl Into<PathBuf>,
        prefix: impl Into<String>,
    ) -> Result<Self, StaticHandlerError> {
        let path: PathBuf = path.into();
        if !path.is_absolute() {
            Err(StaticHandlerError::RelativePath)
        } else if !path.exists() {
            Err(StaticHandlerError::MissingPath)
        } else {
            Ok(Self {
                path,
                prefix: Prefix::from(prefix.into()),
            })
        }
    }
}

fn mime_type(path: &Path) -> &'static str {
    match path.extension().and_then(|s| s.to_str()) {
        Some("gemini") => "text/gemini",
        Some("gmi") => "text/gemini",
        // Better to guess wrong for now
        _ => "text/plain",
    }
}

impl Handler for StaticHandler {
    fn handle_request(&mut self, request: &Request) -> Option<Response> {
        let url = request.url();
        (url.query().is_none())
            .then_some(url.path())
            .inspect(|path| {
                debug!(
                    "Static handler for '{:?}' is considering request for '{}'",
                    self.prefix, path
                )
            })
            .filter(|p| p.starts_with(&*self.prefix))
            .map(|p| p.get(self.prefix.len()..).unwrap_or_default())
            .map(|p| self.path.join(p))
            .filter(|p| p.starts_with(&self.path))
            .map(|p| {
                if p.is_dir() {
                    {
                        let path = p.join("index.gemini");
                        if !path.exists() {
                            p.join("index.gmi")
                        } else {
                            path
                        }
                    }
                } else {
                    p
                }
            })
            .map(|p| (mime_type(&p), p))
            .inspect(|(mime, p)| {
                     debug!(
                    "Static handler for '{:?}' is looking for '{p:?}' of mime type '{mime}' on disk",
                    self.prefix
                );
            }
            )
            .map(|(mime, p)| {
                File::open(p)
                    .map(|file| {
                        Response::Disk(FileResponse {
                            status: Success::Generic,
                            mime,
                            file,
                        })
                    })
                    .unwrap_or_else(|_| {
                        // TODO ergonomics
                        Response::Err(ErrResponse::from_status(Status::PermanentFailure(
                            crate::status::PermanentFailure::NotFound,
                        )))
                    })
            })
    }
}

#[cfg(test)]
mod test_static_handler {
    use super::*;
    use anyhow::Result;
    use rstest::rstest;
    use tempfile::TempDir;

    #[test]
    fn can_only_be_constructed_with_an_extant_absolute_path() {
        assert!(matches!(
            StaticHandler::new("../foo", ""),
            Err(StaticHandlerError::RelativePath)
        ));
        assert!(!PathBuf::from("/foo/bar/baz/blah").exists()); // sanity
        assert!(matches!(
            StaticHandler::new("/foo/bar/baz/blah", ""),
            Err(StaticHandlerError::MissingPath)
        ));
    }

    #[rstest]
    #[case::simple_noslash("foo.gemini", "static")]
    #[case::nested_noslash("posts/foo.gemini", "static")]
    #[case::simple_noslash("foo.gemini", "static/")]
    #[case::nested_noslash("posts/foo.gemini", "static/")]
    fn serves_content_from_its_content_dir(
        #[case] target: &str,
        #[case] prefix: &str,
    ) -> Result<()> {
        let dir = TempDir::new()?;
        let path = dir.path().join(target);
        let _ = path.parent().map(std::fs::create_dir_all);
        std::fs::write(&path, "hello world")?;

        let mut handler = StaticHandler::new(dir.path(), prefix)?;

        let req: Request = format!("gemini://example.com/static/{target}\r\n").parse()?;
        let resp = handler.handle_request(&req).expect("handled");

        let mut buffer = Vec::new();
        resp.send(&mut buffer)?;
        let buffer = String::try_from(buffer)?;

        assert_eq!(buffer, "20 text/gemini\r\nhello world");
        Ok(())
    }

    #[rstest]
    #[case::index_dot_gemini("index.gemini")]
    #[case::index_dot_gmi("index.gmi")]
    fn serves_index_gemini_or_index_gmi_for_slash(#[case] path: &str) -> Result<()> {
        let dir = TempDir::new()?;
        let path = dir.path().join(path);
        std::fs::write(&path, "hello world")?;

        let mut handler = StaticHandler::new(dir.path(), "static")?;

        let req: Request = "gemini://example.com/static/\r\n".parse()?;
        let resp = handler.handle_request(&req).expect("handled");

        let mut buffer = Vec::new();
        resp.send(&mut buffer)?;
        let buffer = String::try_from(buffer)?;

        assert_eq!(buffer, "20 text/gemini\r\nhello world");
        Ok(())
    }

    #[test]
    fn rejects_requests_from_outside_its_content_dir() -> Result<()> {
        let dir = TempDir::new()?;
        let mut handler = StaticHandler::new(dir.path(), "static")?;

        // let req: Request = "gemini://example.co.uk/static/../../passwd\r\n".parse()?;
        let req: Request =
            "gemini://example.co.uk/static/..%2F..%2F..%2F..%2F..%2Fetc%2Fpasswd\r\n".parse()?;
        let resp = handler.handle_request(&req).expect("handled");

        let mut buffer = Vec::new();
        resp.send(&mut buffer)?;
        let buffer = String::try_from(buffer)?;
        assert_eq!(buffer, "51 PermanentFailure(NotFound)\r\n");
        Ok(())
    }

    #[test]
    fn ignores_requests_not_starting_with_its_prefix() -> Result<()> {
        let dir = TempDir::new()?;
        let mut handler = StaticHandler::new(dir.path(), "static")?;

        let req: Request = "gemini://example.co.uk/dynamic/../../passwd\r\n".parse()?;

        assert!(handler.handle_request(&req).is_none());
        Ok(())
    }
}
