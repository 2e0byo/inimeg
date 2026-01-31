use std::str::FromStr;

use url::Url;

use crate::status::{PermanentFailure, Status};

#[derive(Debug, PartialEq)]
pub struct Request(Url);

#[derive(Debug, Clone, thiserror::Error, PartialEq)]
pub enum RequestError {
    #[error("The input exceeded 1024 bytes.")]
    TooLong,
    #[error("This is a gemini:// server.")]
    WrongScheme,
    #[error("URI parse error: `{0:?}`.")]
    URIParseError(#[from] url::ParseError),
    #[error("A request line must be terminated by CRLF.")]
    UnterminatedLine,
    #[error("Gemini URIs may not contain a fragment.")]
    URIContainsFragment,
    #[error("Gemini URIs may not contain userinfo.")]
    URIContainsUserInfo,
    #[error("Gemini URIs must be absolute.")]
    RelativeURI,
}

impl From<&RequestError> for Status {
    fn from(_value: &RequestError) -> Self {
        Status::PermanentFailure(PermanentFailure::BadRequest)
    }
}

impl FromStr for Request {
    type Err = RequestError;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s.len() >= 1026 {
            return Err(RequestError::TooLong);
        }
        match s.get(s.len() - 2..) {
            None => return Err(RequestError::UnterminatedLine),
            Some(terminator) if terminator != "\r\n" => return Err(RequestError::UnterminatedLine),
            _ => {}
        }

        let mut url = Url::parse(s).map_err(|e| match e {
            url::ParseError::RelativeUrlWithoutBase => RequestError::RelativeURI,
            _ => RequestError::URIParseError(e),
        })?;
        if url.scheme() != "gemini" {
            return Err(RequestError::WrongScheme);
        } else if !url.username().is_empty() || url.password().is_some() {
            return Err(RequestError::URIContainsUserInfo);
        } else if url.fragment().is_some() {
            return Err(RequestError::URIContainsFragment);
        }
        if url.path().is_empty() {
            url.set_path("/");
        }
        Ok(Request(url))
    }
}

#[cfg(test)]
mod an_invalid_request_is_rejected {
    use super::*;
    use rstest::rstest;

    #[test]
    fn when_over_1024_bytes() {
        let url = format!("gemini://{}\r\n", "a".repeat(1024 - 11));
        assert!(Request::from_str(&url).is_ok());

        let url = format!("gemini://{}\r\n", "a".repeat(1025 - 10));
        let parsed = Request::from_str(&url);

        assert!(matches!(parsed, Err(RequestError::TooLong)));
    }

    #[rstest]
    #[case("gemini://ends-with-CR\r")]
    #[case("gemini://ends-with-LF\n")]
    #[case("gemini://example.com")]
    fn when_not_terminated_by_crlf(#[case] request: &str) {
        let parsed = Request::from_str(request);
        assert!(matches!(parsed, Err(RequestError::UnterminatedLine)));
    }

    #[test]
    fn when_it_contains_a_fragment() {
        let parsed = Request::from_str("gemini://example.com#frag\r\n");
        assert!(matches!(parsed, Err(RequestError::URIContainsFragment)));
    }

    #[test]
    fn when_it_contains_userinfo() {
        let parsed = Request::from_str("gemini://john@example.com\r\n");
        assert!(matches!(parsed, Err(RequestError::URIContainsUserInfo)));

        let parsed = Request::from_str("gemini://john:secret@example.com\r\n");
        assert!(matches!(parsed, Err(RequestError::URIContainsUserInfo)));
    }

    #[test]
    fn when_its_uri_is_relative() {
        let parsed = Request::from_str("foo/bar\r\n");
        assert!(matches!(parsed, Err(RequestError::RelativeURI)));
    }

    #[test]
    fn when_it_has_the_wrong_scheme() {
        let parsed = Request::from_str("https://example.com\r\n");
        assert!(matches!(parsed, Err(RequestError::WrongScheme)));
    }

    #[test]
    fn when_it_simply_isnt_a_url() {
        let parsed = Request::from_str("---!r\n"); // this is actually a valid relative url...
        assert!(parsed.is_err());
    }
}

#[cfg(test)]
mod a_valid_request {
    use super::*;

    #[test]
    fn is_parsed() {
        let url = Request::from_str("gemini://example.com/foo/bar/baz\r\n")
            .unwrap()
            .0;
        assert_eq!(
            url.to_string(),
            String::from("gemini://example.com/foo/bar/baz")
        );
    }

    #[test]
    fn has_a_missing_path_added() {
        let url = Request::from_str("gemini://example.com\r\n").unwrap().0;
        assert_eq!(url.to_string(), String::from("gemini://example.com/"));
    }
}
