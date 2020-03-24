use crate::io::Buf;
use std::str::{self, FromStr};
use std::collections::BTreeMap;

#[derive(Debug, Copy, Clone)]
pub(crate) enum Severity {
    Panic,
    Fatal,
    Error,
    Warning,
    Notice,
    Debug,
    Info,
    Log,
}

impl Severity {
    pub(crate) fn is_error(self) -> bool {
        match self {
            Severity::Panic | Severity::Fatal | Severity::Error => true,
            _ => false,
        }
    }
}

impl FromStr for Severity {
    type Err = crate::Error;

    fn from_str(s: &str) -> crate::Result<Self> {
        Ok(match s {
            "PANIC" => Severity::Panic,
            "FATAL" => Severity::Fatal,
            "ERROR" => Severity::Error,
            "WARNING" => Severity::Warning,
            "NOTICE" => Severity::Notice,
            "DEBUG" => Severity::Debug,
            "INFO" => Severity::Info,
            "LOG" => Severity::Log,

            _ => {
                return Err(protocol_err!("unexpected response severity: {}", s).into());
            }
        })
    }
}

#[derive(Debug)]
pub(crate) struct Response {
    // always present fields:
    // S: localized Severity
    severity: Box<str>,
    // C: SQLSTATE code
    code: Box<str>,
    // M: Message
    message: Box<str>,
    // all optional fields
    fields: BTreeMap<u8, Box<str>>,
}

impl Response {
    /// Lazily parse the severity
    pub(crate) fn severity(&self) -> crate::Result<Severity> {
        // non-localized Severity
        self.fields.get(&b'V')
            .unwrap_or(&self.severity)
            .parse()
    }

    pub(crate) fn code(&self) -> &str { &self.code }

    pub(crate) fn message(&self) -> &str { &self.message }

    pub (crate) fn field(&self, tag: u8) -> Option<&str> {
        match tag {
            // support fetching these in case users expect them
            b'S' => Some(&self.severity),
            b'C' => Some(&self.code),
            b'M' => Some(&self.message),
            _ => self.fields.get(&tag)
        }
    }

    pub(crate) fn read(mut buf: &[u8]) -> crate::Result<Self> {
        let mut code = None::<Box<str>>;
        let mut message = None::<Box<str>>;
        let mut severity = None::<Box<str>>;

        let mut other = BTreeMap::new();

        loop {
            let field_type = buf.get_u8()?;

            if field_type == 0 {
                break;
            }

            let field_value = buf.get_str_nul()?;

            match field_type {
                b'S' => {
                    severity = Some(field_value.into());
                }

                b'C' => {
                    code = Some(field_value.into());
                }

                b'M' => {
                    message = Some(field_value.into());
                }

                _ => {
                    other.insert(field_type, field_value.into());
                }
            }
        }

        let severity = severity
            .ok_or(protocol_err!(
                "did not receieve field `severity` for Response"
            ))?;

        let code = code.ok_or(protocol_err!("did not receieve field `code` for Response",))?;
        let message = message.ok_or(protocol_err!(
            "did not receieve field `message` for Response"
        ))?;

        Ok(Self {
            severity,
            code,
            message,
            fields: other,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::{Response, Severity};
    use matches::assert_matches;

    const RESPONSE: &[u8] = b"SNOTICE\0VNOTICE\0C42710\0Mextension \"uuid-ossp\" already exists, \
          skipping\0Fextension.c\0L1656\0RCreateExtension\0\0";

    #[test]
    fn it_decodes_response() {
        let message = Response::read(RESPONSE).unwrap();

        assert_matches!(message.severity, Severity::Notice);
        assert_eq!(&*message.code, "42710");
        assert_eq!(&*message[b'F'], "extension.c");
        assert_eq!(&*message[b'L'], "1656");
        assert_eq!(&*message[b'R'], "CreateExtension");
        assert_eq!(
            &*message.message,
            "extension \"uuid-ossp\" already exists, skipping"
        );
    }
}
