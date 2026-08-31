//! Shared fail-closed framing and host validation for loopback HTTP listeners.

use std::collections::HashMap;
use std::io::Read;
use std::net::{IpAddr, SocketAddr};

use crate::naruon_http::header_is_credential;
use crate::{ApiError, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT};

/// Maximum request-line plus header bytes accepted before the body.
pub const NARUON_LIVE_HEADER_BYTE_LIMIT: usize = 8 * 1024;

/// Maximum number of HTTP header lines on one live request.
pub const NARUON_LIVE_HEADER_COUNT_LIMIT: usize = 32;

/// Read one HTTP/1.1 request, including its declared UTF-8 body.
pub(crate) fn read_http_request<R: Read>(reader: &mut R) -> Result<String, ApiError> {
    read_http_request_with_limit(reader, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)
}

/// Read one HTTP/1.1 request with a caller-selected body limit.
pub(crate) fn read_http_request_with_limit<R: Read>(
    reader: &mut R,
    maximum_body_bytes: usize,
) -> Result<String, ApiError> {
    let mut header_bytes = Vec::new();
    let mut byte = [0_u8; 1];
    while !header_bytes.ends_with(b"\r\n\r\n") {
        if header_bytes.len() >= NARUON_LIVE_HEADER_BYTE_LIMIT {
            return Err(ApiError::LimitExceeded);
        }
        let read = reader
            .read(&mut byte)
            .map_err(|error| map_io_error(&error))?;
        if read == 0 {
            return Err(ApiError::InvalidWirePayload);
        }
        header_bytes.push(byte[0]);
    }
    let header_text =
        std::str::from_utf8(&header_bytes).map_err(|_| ApiError::InvalidWirePayload)?;
    let content_length = declared_content_length(header_text)?;
    if content_length > maximum_body_bytes {
        return Err(ApiError::LimitExceeded);
    }
    let mut body = vec![0_u8; content_length];
    if content_length > 0 {
        reader
            .read_exact(&mut body)
            .map_err(|error| map_io_error(&error))?;
    }
    let body_text = std::str::from_utf8(&body).map_err(|_| ApiError::InvalidWirePayload)?;
    Ok(format!("{header_text}{body_text}"))
}

/// Parsed loopback HTTP/1.1 request already in memory.
#[derive(Debug)]
pub struct LoopbackHttpParts<'a> {
    /// HTTP method token from the request line.
    pub method: &'a str,
    /// Request-target path. Query strings are refused.
    pub path: &'a str,
    /// Lowercased unique headers.
    pub headers: HashMap<String, String>,
    /// UTF-8 body whose length matches `Content-Length`.
    pub body: &'a str,
}

/// Parse one complete loopback HTTP/1.1 request already in memory.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] for framing, version, or header
/// violations and [`ApiError::LimitExceeded`] when the header block or body
/// exceeds the configured bound.
pub fn parse_loopback_http_parts(
    request: &str,
    maximum_body_bytes: usize,
) -> Result<LoopbackHttpParts<'_>, ApiError> {
    let (header_block, body) = split_request_with_limit(request, maximum_body_bytes)?;
    let mut lines = header_block.split("\r\n");
    let (method, path) = parse_request_line(lines.next().unwrap_or(""))?;
    let headers = parse_headers(&mut lines)?;
    Ok(LoopbackHttpParts {
        method,
        path,
        headers,
        body,
    })
}
pub(crate) fn split_request(request: &str) -> Result<(&str, &str), ApiError> {
    split_request_with_limit(request, DEFAULT_ANALYSIS_RUN_BYTE_LIMIT)
}

/// Split one complete request with a caller-selected body limit.
pub(crate) fn split_request_with_limit(
    request: &str,
    maximum_body_bytes: usize,
) -> Result<(&str, &str), ApiError> {
    let Some(index) = request.find("\r\n\r\n") else {
        if request.len() >= NARUON_LIVE_HEADER_BYTE_LIMIT {
            return Err(ApiError::LimitExceeded);
        }
        return Err(ApiError::InvalidWirePayload);
    };
    if index > NARUON_LIVE_HEADER_BYTE_LIMIT {
        return Err(ApiError::LimitExceeded);
    }
    let header_block = &request[..index];
    let body = &request[index + 4..];
    let declared = declared_content_length(&format!("{header_block}\r\n\r\n"))?;
    if declared != body.len() {
        return Err(ApiError::InvalidWirePayload);
    }
    if declared > maximum_body_bytes {
        return Err(ApiError::LimitExceeded);
    }
    Ok((header_block, body))
}

/// Parse the single decimal content length from a complete header block.
pub(crate) fn declared_content_length(header_text: &str) -> Result<usize, ApiError> {
    let header_block = header_text
        .strip_suffix("\r\n\r\n")
        .ok_or(ApiError::InvalidWirePayload)?;
    let mut found = None;
    for line in header_block.split("\r\n").skip(1) {
        let (name, value) = split_header_line(line)?;
        if name.eq_ignore_ascii_case("content-length") {
            if found.is_some()
                || value.is_empty()
                || !value.bytes().all(|byte| byte.is_ascii_digit())
            {
                return Err(ApiError::InvalidWirePayload);
            }
            found = Some(value.parse().map_err(|_| ApiError::InvalidWirePayload)?);
        }
    }
    found.ok_or(ApiError::InvalidWirePayload)
}

/// Parse a strict HTTP/1.1 request line into method and path.
pub(crate) fn parse_request_line(line: &str) -> Result<(&str, &str), ApiError> {
    let mut parts = line.split(' ');
    let method = parts.next().ok_or(ApiError::InvalidWirePayload)?;
    let path = parts.next().ok_or(ApiError::InvalidWirePayload)?;
    let version = parts.next().ok_or(ApiError::InvalidWirePayload)?;
    if parts.next().is_some() || version != "HTTP/1.1" {
        return Err(ApiError::InvalidWirePayload);
    }
    if !path.starts_with('/') || path.contains('?') || path.contains('#') || path.contains("://") {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok((method, path))
}

/// Parse and normalize bounded, unique HTTP headers.
pub(crate) fn parse_headers<'a, I>(lines: I) -> Result<HashMap<String, String>, ApiError>
where
    I: Iterator<Item = &'a str>,
{
    let mut headers = HashMap::new();
    let mut count = 0_usize;
    for line in lines {
        count += 1;
        if count > NARUON_LIVE_HEADER_COUNT_LIMIT {
            return Err(ApiError::LimitExceeded);
        }
        let (name, value) = split_header_line(line)?;
        let key = name.to_ascii_lowercase();
        if headers.contains_key(&key) {
            return Err(ApiError::InvalidWirePayload);
        }
        headers.insert(key, value.to_owned());
    }
    Ok(headers)
}

/// Split one header line while rejecting malformed names.
pub(crate) fn split_header_line(line: &str) -> Result<(&str, &str), ApiError> {
    let Some((name, value)) = line.split_once(':') else {
        return Err(ApiError::InvalidWirePayload);
    };
    if name.is_empty() || name.chars().any(|ch| ch.is_whitespace() || ch.is_control()) {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok((name, value.trim()))
}

/// Return one required non-empty normalized header value.
pub(crate) fn header_value<'a>(
    headers: &'a HashMap<String, String>,
    name: &str,
) -> Result<&'a str, ApiError> {
    let value = headers.get(name).ok_or(ApiError::InvalidWirePayload)?;
    if value.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(value.as_str())
}

/// Validate common credential, framing, content-type, and loopback boundaries.
pub(crate) fn validate_common_headers(
    headers: &HashMap<String, String>,
    bound_addr: Option<SocketAddr>,
) -> Result<(), ApiError> {
    for name in headers.keys() {
        if header_is_credential(name) {
            return Err(ApiError::AuthorizationDenied);
        }
    }
    if headers.contains_key("transfer-encoding") {
        return Err(ApiError::InvalidWirePayload);
    }
    let host = header_value(headers, "host")?;
    if host_implies_table_access(host) {
        return Err(ApiError::InvalidWirePayload);
    }
    if !host_is_loopback(host, bound_addr) {
        return Err(ApiError::AuthorizationDenied);
    }
    if header_value(headers, "content-type")? != "application/json" {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(())
}

/// Return whether a host value implies direct database or table access.
pub(crate) fn host_implies_table_access(host: &str) -> bool {
    let lowered = host.to_ascii_lowercase();
    lowered.contains("postgres")
        || lowered.contains("jdbc")
        || lowered.contains("/sql")
        || lowered.contains("/tables/")
        || lowered.contains('\'')
        || lowered.contains(';')
        || lowered.contains('\\')
        || lowered.contains(' ')
        || lowered.chars().any(char::is_control)
}

/// Return whether a host resolves to loopback or the bound loopback socket.
pub(crate) fn host_is_loopback(host: &str, bound_addr: Option<SocketAddr>) -> bool {
    if let Some(bound) = bound_addr
        && (host == bound.to_string() || host == bound.ip().to_string())
    {
        return true;
    }
    let lowered = host.to_ascii_lowercase();
    if lowered == "localhost"
        || lowered
            .strip_prefix("localhost:")
            .is_some_and(|port| !port.is_empty() && port.parse::<u16>().is_ok())
    {
        return true;
    }
    if let Ok(addr) = host.parse::<SocketAddr>() {
        return addr.ip().is_loopback();
    }
    if let Ok(ip) = host.parse::<IpAddr>() {
        return ip.is_loopback();
    }
    false
}

/// Map socket timeout and transport failures to redacted API errors.
pub(crate) fn map_io_error(error: &std::io::Error) -> ApiError {
    match error.kind() {
        std::io::ErrorKind::TimedOut | std::io::ErrorKind::WouldBlock => ApiError::LimitExceeded,
        _ => ApiError::InvalidWirePayload,
    }
}
