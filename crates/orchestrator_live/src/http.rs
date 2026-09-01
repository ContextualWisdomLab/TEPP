//! HTTP/1.1 framing helpers for the loopback listener.

use std::collections::HashMap;
use std::io::{Read, Write};

use crate::error::OrchestratorLiveError;
use crate::request::{
    host_implies_table_access, require_nonempty, DEFAULT_INTERPRETATION_BYTE_LIMIT,
};

/// Maximum request-line plus header bytes accepted before the body.
pub const LIVE_HEADER_BYTE_LIMIT: usize = 8 * 1024;

/// Maximum number of HTTP header lines on one live request.
pub const LIVE_HEADER_COUNT_LIMIT: usize = 32;

/// HTTP/1.1 response produced by the orchestrator live listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct OrchestratorLiveResponse {
    /// Numeric status code.
    pub status_code: u16,
    /// RFC 7231 reason phrase paired with [`Self::status_code`].
    pub reason_phrase: &'static str,
    /// JSON accepted-run or redacted error envelope.
    pub body: String,
}

impl OrchestratorLiveResponse {
    pub(crate) fn json(status_code: u16, reason_phrase: &'static str, body: String) -> Self {
        Self {
            status_code,
            reason_phrase,
            body,
        }
    }

    /// Render the response as an HTTP/1.1 message.
    #[must_use]
    pub fn to_http_bytes(&self) -> Vec<u8> {
        format!(
            "HTTP/1.1 {} {}\r\ncontent-type: application/json\r\ncontent-length: {}\r\n\r\n{}",
            self.status_code,
            self.reason_phrase,
            self.body.len(),
            self.body
        )
        .into_bytes()
    }
}

pub(crate) fn read_http_request<R: Read>(reader: &mut R) -> Result<String, OrchestratorLiveError> {
    let mut header_bytes = Vec::new();
    let mut byte = [0_u8; 1];
    loop {
        if header_bytes.len() >= LIVE_HEADER_BYTE_LIMIT {
            return Err(OrchestratorLiveError::LimitExceeded);
        }
        let read = reader
            .read(&mut byte)
            .map_err(|error| map_io_error(&error))?;
        if read == 0 {
            return Err(OrchestratorLiveError::InvalidWirePayload);
        }
        header_bytes.push(byte[0]);
        if header_bytes.ends_with(b"\r\n\r\n") {
            break;
        }
    }
    let header_text = std::str::from_utf8(&header_bytes)
        .map_err(|_| OrchestratorLiveError::InvalidWirePayload)?;
    let content_length = declared_content_length(header_text)?;
    if content_length > DEFAULT_INTERPRETATION_BYTE_LIMIT {
        return Err(OrchestratorLiveError::LimitExceeded);
    }
    let mut body = vec![0_u8; content_length];
    if content_length > 0 {
        reader
            .read_exact(&mut body)
            .map_err(|error| map_io_error(&error))?;
    }
    let body_text =
        std::str::from_utf8(&body).map_err(|_| OrchestratorLiveError::InvalidWirePayload)?;
    Ok(format!("{header_text}{body_text}"))
}

pub(crate) fn write_response<W: Write>(
    writer: &mut W,
    response: &OrchestratorLiveResponse,
) -> Result<(), OrchestratorLiveError> {
    writer
        .write_all(&response.to_http_bytes())
        .map_err(|error| map_io_error(&error))?;
    writer.flush().map_err(|error| map_io_error(&error))
}

pub(crate) fn split_request(request: &str) -> Result<(&str, &str), OrchestratorLiveError> {
    let Some(index) = request.find("\r\n\r\n") else {
        if request.len() >= LIVE_HEADER_BYTE_LIMIT {
            return Err(OrchestratorLiveError::LimitExceeded);
        }
        return Err(OrchestratorLiveError::InvalidWirePayload);
    };
    if index > LIVE_HEADER_BYTE_LIMIT {
        return Err(OrchestratorLiveError::LimitExceeded);
    }
    let header_block = &request[..index];
    let body = &request[index + 4..];
    let declared = declared_content_length(&format!("{header_block}\r\n\r\n"))?;
    if declared != body.len() {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if declared > DEFAULT_INTERPRETATION_BYTE_LIMIT {
        return Err(OrchestratorLiveError::LimitExceeded);
    }
    Ok((header_block, body))
}

pub(crate) fn declared_content_length(header_text: &str) -> Result<usize, OrchestratorLiveError> {
    let header_block = header_text
        .strip_suffix("\r\n\r\n")
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    let mut found = None;
    for line in header_block.split("\r\n").skip(1) {
        let (name, value) = split_header_line(line)?;
        if name.eq_ignore_ascii_case("content-length") {
            if found.is_some() {
                return Err(OrchestratorLiveError::InvalidWirePayload);
            }
            if value.is_empty() || !value.bytes().all(|byte| byte.is_ascii_digit()) {
                return Err(OrchestratorLiveError::InvalidWirePayload);
            }
            found = Some(
                value
                    .parse()
                    .map_err(|_| OrchestratorLiveError::InvalidWirePayload)?,
            );
        }
    }
    found.ok_or(OrchestratorLiveError::InvalidWirePayload)
}

pub(crate) fn parse_request_line(line: &str) -> Result<(&str, &str), OrchestratorLiveError> {
    let mut parts = line.split(' ');
    let method = parts
        .next()
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    let path = parts
        .next()
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    let version = parts
        .next()
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    if parts.next().is_some() || version != "HTTP/1.1" {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if !path.starts_with('/') || path.contains('?') || path.contains('#') || path.contains("://") {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    Ok((method, path))
}

pub(crate) fn parse_headers<'a, I>(
    lines: I,
) -> Result<HashMap<String, String>, OrchestratorLiveError>
where
    I: Iterator<Item = &'a str>,
{
    let mut headers = HashMap::new();
    let mut count = 0_usize;
    for line in lines {
        count += 1;
        if count > LIVE_HEADER_COUNT_LIMIT {
            return Err(OrchestratorLiveError::LimitExceeded);
        }
        let (name, value) = split_header_line(line)?;
        let key = name.to_ascii_lowercase();
        if headers.contains_key(&key) {
            return Err(OrchestratorLiveError::InvalidWirePayload);
        }
        headers.insert(key, value.to_owned());
    }
    Ok(headers)
}

pub(crate) fn split_header_line(line: &str) -> Result<(&str, &str), OrchestratorLiveError> {
    let Some((name, value)) = line.split_once(':') else {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    };
    if name.is_empty() || name.chars().any(|ch| ch.is_whitespace() || ch.is_control()) {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    Ok((name, value.trim()))
}

pub(crate) fn refuse_live_headers(
    headers: &HashMap<String, String>,
) -> Result<(), OrchestratorLiveError> {
    refuse_common_live_headers(headers)?;
    let _idempotency_key = header_value(headers, "idempotency-key")?;
    Ok(())
}

/// Collection GET admits empty bodies and refuses `idempotency-key`.
pub(crate) fn refuse_collection_get_headers(
    headers: &HashMap<String, String>,
) -> Result<(), OrchestratorLiveError> {
    refuse_common_live_headers(headers)?;
    if headers.contains_key("idempotency-key") {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    Ok(())
}

/// GET-by-id admits empty bodies and refuses pagination plus `idempotency-key`.
pub(crate) fn refuse_retrieval_get_headers(
    headers: &HashMap<String, String>,
) -> Result<(), OrchestratorLiveError> {
    refuse_collection_get_headers(headers)?;
    if headers.contains_key("tepp-page-limit") || headers.contains_key("tepp-page-cursor") {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    Ok(())
}

fn refuse_common_live_headers(
    headers: &HashMap<String, String>,
) -> Result<(), OrchestratorLiveError> {
    for (name, value) in headers {
        if header_is_credential(name) || header_is_credential(value) {
            return Err(OrchestratorLiveError::AuthorizationDenied);
        }
    }
    let host = header_value(headers, "host")?;
    if host_implies_table_access(host) {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if header_value(headers, "content-type")? != "application/json" {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if header_value(headers, "tepp-consumer")? != "contextual-orchestrator" {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    if header_value(headers, "tepp-contract-version")? != "1" {
        return Err(OrchestratorLiveError::InvalidWirePayload);
    }
    Ok(())
}

pub(crate) fn header_value<'a>(
    headers: &'a HashMap<String, String>,
    name: &str,
) -> Result<&'a str, OrchestratorLiveError> {
    let value = headers
        .get(name)
        .ok_or(OrchestratorLiveError::InvalidWirePayload)?;
    require_nonempty(value)?;
    Ok(value.as_str())
}

pub(crate) fn header_is_credential(name: &str) -> bool {
    let lowered = name.to_ascii_lowercase();
    lowered == "authorization"
        || lowered == "cookie"
        || lowered == "x-api-key"
        || lowered.contains("token")
        || lowered.contains("copilot")
        || lowered.contains("github")
        || lowered.contains("nvidia_nim_api_key")
}

pub(crate) fn map_io_error(error: &std::io::Error) -> OrchestratorLiveError {
    match error.kind() {
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => {
            OrchestratorLiveError::LimitExceeded
        }
        _ => OrchestratorLiveError::InvalidWirePayload,
    }
}

pub(crate) fn status_for(error: OrchestratorLiveError) -> (u16, &'static str) {
    match error {
        OrchestratorLiveError::InvalidWirePayload => (400, "Bad Request"),
        OrchestratorLiveError::AuthorizationDenied => (403, "Forbidden"),
        OrchestratorLiveError::LimitExceeded => (413, "Payload Too Large"),
        OrchestratorLiveError::UnsupportedContractVersion
        | OrchestratorLiveError::ScientificAuthorityRefused => (422, "Unprocessable Entity"),
    }
}

#[cfg(test)]
mod tests {
    use super::{
        declared_content_length, header_is_credential, map_io_error, parse_headers,
        parse_request_line, refuse_collection_get_headers, refuse_live_headers,
        refuse_retrieval_get_headers, split_header_line, split_request, status_for,
    };
    use crate::error::OrchestratorLiveError;
    use std::collections::HashMap;
    use std::io::ErrorKind;

    #[test]
    fn helpers_cover_status_io_and_request_line_edges() {
        assert_eq!(
            status_for(OrchestratorLiveError::InvalidWirePayload),
            (400, "Bad Request")
        );
        assert_eq!(
            status_for(OrchestratorLiveError::AuthorizationDenied),
            (403, "Forbidden")
        );
        assert_eq!(
            status_for(OrchestratorLiveError::LimitExceeded),
            (413, "Payload Too Large")
        );
        assert_eq!(
            status_for(OrchestratorLiveError::UnsupportedContractVersion),
            (422, "Unprocessable Entity")
        );
        assert_eq!(
            status_for(OrchestratorLiveError::ScientificAuthorityRefused),
            (422, "Unprocessable Entity")
        );
        assert_eq!(
            map_io_error(&std::io::Error::new(ErrorKind::TimedOut, "t")),
            OrchestratorLiveError::LimitExceeded
        );
        assert_eq!(
            map_io_error(&std::io::Error::new(ErrorKind::WouldBlock, "w")),
            OrchestratorLiveError::LimitExceeded
        );
        assert_eq!(
            map_io_error(&std::io::Error::other("x")),
            OrchestratorLiveError::InvalidWirePayload
        );
        assert_eq!(
            parse_request_line("POST /v1/interpretation-runs HTTP/1.1 extra"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            parse_request_line("POST https://tepp.example/v1/interpretation-runs HTTP/1.1"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            parse_request_line("POST /v1/interpretation-runs#x HTTP/1.1"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            parse_request_line("POST"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            parse_request_line("POST /only"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            split_header_line("NoColon"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            split_header_line(": empty-name"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            split_header_line("Bad Name: v"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            split_header_line("Host: 127.0.0.1").expect("hdr"),
            ("Host", "127.0.0.1")
        );
        assert_eq!(
            declared_content_length(
                "POST /x HTTP/1.1\r\ncontent-length: 1\r\ncontent-length: 1\r\n\r\n"
            ),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            declared_content_length("POST /x HTTP/1.1\r\ncontent-length: +1\r\n\r\n"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            declared_content_length("POST /x HTTP/1.1\r\nHost: 127.0.0.1\r\n"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            split_request(&"x".repeat(super::LIVE_HEADER_BYTE_LIMIT)),
            Err(OrchestratorLiveError::LimitExceeded)
        );
        assert_eq!(
            declared_content_length(
                "POST /x HTTP/1.1\r\ncontent-length: 999999999999999999999\r\n\r\n"
            ),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert!(header_is_credential("Authorization"));
        assert!(header_is_credential("x-github-token"));
        assert!(!header_is_credential("host"));
    }

    #[test]
    fn helpers_cover_short_circuit_and_transport_edges() {
        assert_eq!(
            parse_request_line("POST relative HTTP/1.1"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            declared_content_length("POST /x HTTP/1.1\r\nmalformed\r\ncontent-length: 0\r\n\r\n"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            declared_content_length("POST /x HTTP/1.1\r\ncontent-length:\r\n\r\n"),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            parse_headers(["malformed"].into_iter()),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        assert_eq!(
            parse_headers(["Bad\u{0001}Name: value"].into_iter()),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );

        let mut missing = HashMap::new();
        assert_eq!(
            refuse_live_headers(&missing),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        missing.insert("host".into(), "127.0.0.1".into());
        assert_eq!(
            refuse_live_headers(&missing),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        missing.insert("content-type".into(), "application/json".into());
        assert_eq!(
            refuse_live_headers(&missing),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        missing.insert("tepp-consumer".into(), "contextual-orchestrator".into());
        assert_eq!(
            refuse_live_headers(&missing),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        missing.insert("tepp-contract-version".into(), "1".into());
        assert_eq!(
            refuse_live_headers(&missing),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        missing.insert("idempotency-key".into(), "idem".into());
        refuse_live_headers(&missing).expect("complete headers");

        assert!(header_is_credential("x-github"));
        assert!(header_is_credential("x-copilot"));
        assert!(header_is_credential("x-nvidia_nim_api_key"));
        assert!(!header_is_credential("x-safe-header"));
    }

    #[test]
    fn collection_get_headers_refuse_idempotency_key_and_foreign_consumers() {
        let mut headers = HashMap::new();
        headers.insert("host".into(), "127.0.0.1".into());
        headers.insert("content-type".into(), "application/json".into());
        headers.insert("tepp-consumer".into(), "contextual-orchestrator".into());
        headers.insert("tepp-contract-version".into(), "1".into());
        headers.insert("idempotency-key".into(), "idem".into());
        assert_eq!(
            refuse_collection_get_headers(&headers),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        headers.remove("idempotency-key");
        refuse_collection_get_headers(&headers).expect("collection headers");
        refuse_retrieval_get_headers(&headers).expect("retrieval headers");
        headers.insert("tepp-page-limit".into(), "1".into());
        assert_eq!(
            refuse_retrieval_get_headers(&headers),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        headers.remove("tepp-page-limit");
        headers.insert("tepp-consumer".into(), "naruon".into());
        assert_eq!(
            refuse_collection_get_headers(&headers),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        headers.insert("tepp-consumer".into(), "lineageweave".into());
        assert_eq!(
            refuse_collection_get_headers(&headers),
            Err(OrchestratorLiveError::InvalidWirePayload)
        );
        headers.insert("tepp-consumer".into(), "contextual-orchestrator".into());
        headers.insert("authorization".into(), "Bearer x".into());
        assert_eq!(
            refuse_collection_get_headers(&headers),
            Err(OrchestratorLiveError::AuthorizationDenied)
        );
    }
}
