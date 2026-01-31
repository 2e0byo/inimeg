use std::io::Write;

use bytes::Bytes;

use crate::status::{Status, Success};

pub struct ErrResponse {
    pub status: Status,
    pub msg: Option<Bytes>,
}

pub struct SuccessResponse {
    pub status: Success,
    pub mime: String,
    pub body: Bytes,
}

// TODO: do we really want a trait?
// pub struct StreamingResponse<T: Read> {
//     body: T
// }

pub enum Response {
    Err(ErrResponse),
    Fixed(SuccessResponse),
    // Streaming(Box<dyn StreamingResponse>)
}

impl Response {
    /// Send this response.
    pub fn send<W: Write>(self, mut writer: W) -> std::io::Result<()> {
        match self {
            Self::Err(resp) => {
                write!(writer, "{}", resp.status)?;
                if let Some(msg) = resp.msg {
                    writer.write_all(b" ")?;
                    writer.write_all(&msg)?;
                }
                write!(writer, "\r\n")?;
            }
            Self::Fixed(SuccessResponse { status, mime, body }) => {
                write!(writer, "{} {}\r\n", Status::Success(status), mime)?;
                writer.write_all(&body)?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use anyhow::Result;

    use super::*;

    #[test]
    fn an_error_response_is_serialised_correctly() -> Result<()> {
        let mut buf = Vec::new();

        let resp = ErrResponse {
            status: Status::Redirect(crate::status::Redirect::Permanent),
            msg: Some("gemini://foo.example.com/bar".into()),
        };

        Response::Err(resp).send(&mut buf)?;

        let serialised = String::try_from(buf)?;
        assert_eq!(serialised, "31 gemini://foo.example.com/bar\r\n");
        Ok(())
    }

    #[test]
    fn a_fixed_response_is_serialised_correctly() -> Result<()> {
        let mut buf = Vec::new();

        let resp = SuccessResponse {
            status: Success::Generic,
            mime: "text/plain".into(),
            body: r#"Line 0
line1
line2

line4
"#
            .into(),
        };

        Response::Fixed(resp).send(&mut buf)?;

        let serialised = String::try_from(buf)?;
        assert_eq!(
            serialised,
            "20 text/plain\r\nLine 0\nline1\nline2\n\nline4\n"
        );
        Ok(())
    }
}
