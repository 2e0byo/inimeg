/// A Gemini status code.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatusCode(u8);

#[derive(Debug, Clone, thiserror::Error)]
#[error("Failed to parse status code: {0}")]
pub struct StatusCodeParseError(u8);

impl TryFrom<u8> for StatusCode {
    type Error = StatusCodeParseError;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            0..10 => Err(StatusCodeParseError(value)),
            10..70 => Ok(StatusCode(value)),
            70.. => Err(StatusCodeParseError(value)),
        }
    }
}

/// The server expects more of the client.
#[derive(Debug, Clone, Copy)]
pub enum InputExpected {
    /// Do better next time.
    Generic,
    /// Sensitive input.
    Whisper,
}

/// The server has succeeded.
#[derive(Debug, Clone, Copy)]
pub enum Success {
    Generic,
}

/// The server wishes the client to try looking in the correct place next time.
#[derive(Debug, Clone, Copy)]
pub enum Redirect {
    Temporary,
    Permanent,
}

/// The server isn't feeling so well.
#[derive(Debug, Clone, Copy)]
pub enum TemporaryFailure {
    Generic,
    Unavailable,
    CGIError,
    ProxyError,
    SlowDown,
}

/// The server has given up.
#[derive(Debug, Clone, Copy)]
pub enum PermanentFailure {
    Generic,
    NotFound,
    Gone,
    ProxyRequestRefused,
    BadRequest,
}

/// The client should get its paperwork in order.
#[derive(Debug, Clone, Copy)]
pub enum CertificateRequired {
    Generic,
    /// That cert is no good for this resource.
    NotAuthorised,
    /// That cert is no good.
    Invalid,
}

/// A status, according to its logical kind
#[derive(Debug, Clone, Copy)]
pub enum Status {
    InputExpected(InputExpected),
    Success(Success),
    Redirect(Redirect),
    TemporaryFailure(TemporaryFailure),
    PermanentFailure(PermanentFailure),
    CertificateRequired(CertificateRequired),
}

/// The powers that be have not yet decided what this means
#[derive(Debug, Clone, Copy, thiserror::Error)]
#[error("Undefined (but valid) status code: `{0:?}`")]
pub struct UndefinedStatus(StatusCode);

impl TryFrom<StatusCode> for Status {
    type Error = UndefinedStatus;
    fn try_from(value: StatusCode) -> Result<Self, Self::Error> {
        match value.0 {
            10 => Ok(Status::InputExpected(InputExpected::Generic)),
            11 => Ok(Status::InputExpected(InputExpected::Whisper)),
            20 => Ok(Status::Success(Success::Generic)),
            30 => Ok(Status::Redirect(Redirect::Temporary)),
            31 => Ok(Status::Redirect(Redirect::Permanent)),
            40 => Ok(Status::TemporaryFailure(TemporaryFailure::Generic)),
            41 => Ok(Status::TemporaryFailure(TemporaryFailure::Unavailable)),
            42 => Ok(Status::TemporaryFailure(TemporaryFailure::CGIError)),
            43 => Ok(Status::TemporaryFailure(TemporaryFailure::ProxyError)),
            44 => Ok(Status::TemporaryFailure(TemporaryFailure::SlowDown)),
            50 => Ok(Status::PermanentFailure(PermanentFailure::Generic)),
            51 => Ok(Status::PermanentFailure(PermanentFailure::NotFound)),
            52 => Ok(Status::PermanentFailure(PermanentFailure::Gone)),
            53 => Ok(Status::PermanentFailure(
                PermanentFailure::ProxyRequestRefused,
            )),
            59 => Ok(Status::PermanentFailure(PermanentFailure::BadRequest)),
            60 => Ok(Status::CertificateRequired(CertificateRequired::Generic)),
            61 => Ok(Status::CertificateRequired(
                CertificateRequired::NotAuthorised,
            )),
            62 => Ok(Status::CertificateRequired(CertificateRequired::Invalid)),
            _ => Err(UndefinedStatus(value)),
        }
    }
}

#[rustfmt::skip]
impl From<&Status> for StatusCode {
    fn from(value: &Status) -> Self {
        match value {
            Status::InputExpected(InputExpected::Generic)                    => StatusCode(10),
            Status::InputExpected(InputExpected::Whisper)                    => StatusCode(11),
            Status::Success(Success::Generic)                                => StatusCode(20),
            Status::Redirect(Redirect::Temporary)                            => StatusCode(30),
            Status::Redirect(Redirect::Permanent)                            => StatusCode(31),
            Status::TemporaryFailure(TemporaryFailure::Generic)              => StatusCode(40),
            Status::TemporaryFailure(TemporaryFailure::Unavailable)          => StatusCode(41),
            Status::TemporaryFailure(TemporaryFailure::CGIError)             => StatusCode(42),
            Status::TemporaryFailure(TemporaryFailure::ProxyError)           => StatusCode(43),
            Status::TemporaryFailure(TemporaryFailure::SlowDown)             => StatusCode(44),
            Status::PermanentFailure(PermanentFailure::Generic)              => StatusCode(50),
            Status::PermanentFailure(PermanentFailure::NotFound)             => StatusCode(51),
            Status::PermanentFailure(PermanentFailure::Gone)                 => StatusCode(52),
            Status::PermanentFailure(PermanentFailure::ProxyRequestRefused)  => StatusCode(53),
            Status::PermanentFailure(PermanentFailure::BadRequest)           => StatusCode(59),
            Status::CertificateRequired(CertificateRequired::Generic)        => StatusCode(60),
            Status::CertificateRequired(CertificateRequired::NotAuthorised)  => StatusCode(61),
            Status::CertificateRequired(CertificateRequired::Invalid)        => StatusCode(62),
        }
    }
}

impl std::fmt::Display for Status {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", StatusCode::from(self).0)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use anyhow::Result;
    use rstest::rstest;

    #[rstest]
    #[case(10)]
    #[case(11)]
    #[case(20)]
    #[case(30)]
    #[case(31)]
    #[case(40)]
    #[case(41)]
    #[case(42)]
    #[case(43)]
    #[case(44)]
    #[case(50)]
    #[case(50)]
    #[case(51)]
    #[case(52)]
    #[case(53)]
    #[case(59)]
    #[case(60)]
    #[case(61)]
    #[case(62)]
    fn valid_status_codes_round_tripped(#[case] code: u8) -> Result<()> {
        let code = StatusCode::try_from(code)?;
        let status = Status::try_from(code)?;
        let recoded = StatusCode::from(&status);

        assert_eq!(code, recoded);
        Ok(())
    }

    #[rstest]
    #[case::too_small(9)]
    #[case::too_large(70)]
    fn invalid_status_code_rejected(#[case] code: u8) {
        assert!(StatusCode::try_from(code).is_err())
    }

    #[test]
    fn an_undefined_status_code_is_not_a_valid_status() -> Result<()> {
        let code = StatusCode::try_from(55)?;
        assert!(Status::try_from(code).is_err());
        Ok(())
    }
}
