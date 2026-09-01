//! Operator loopback CLI for `LineageWeave` project-history cancel POST.
//!
//! GAP-003A unique slice: operators run `tepp-project-history-cancel cancel`
//! to mint `lineageweave_project_history_cancel_exchange` onto spawned
//! `tepp-loopback` TCP. Stdout is one metric-free cancelled identity with
//! `cancelled=true` and `inference_status=temporal_association_only`.
//! `tepp.scientific_acceptance.v1` never appears. Naruon is refused.
//! `NaruonLiveService` stays POST-only. Dedicated binary so it does not
//! collide with `tepp-project-histories` (#428) or `tepp-project-history-get`
//! (#431). This module does not duplicate project-history cancel HTTP (#447),
//! retrieval CLI (#431), GET-by-id (#429), collection CLI (#428), collection
//! GET (#424), POST CLI (#420), export cancel CLI (#446), interpretation-run
//! cancel CLI (#442), analysis-run cancel (#361), Leiden, or GAP-010
//! Figma/export. Persistence remains GAP-003B.

use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use crate::lineageweave_http::LINEAGEWEAVE_CONSUMER_CODE;
use crate::live_http::map_io_error;
use crate::naruon_http::header_is_credential;
use crate::project_history::validate_project_history_registry_identity;
use crate::wire::require_nonempty;
use crate::{
    lineageweave_project_history_cancel_exchange, project_history_cancel_path_id,
    refuse_metrics_on_project_history_collection_payload, AnalysisRunLiveService, ApiError,
    ProjectHistoryCancelled, PROJECT_HISTORY_CANCEL_ID_MAX_LEN, NARUON_LIVE_IO_TIMEOUT,
    NaruonHttpExchange, NaruonLiveResponse,
};

const SCIENTIFIC_ACCEPTANCE_SCHEMA: &str = "tepp.scientific_acceptance.v1";

/// Supported operator verbs for the loopback project-history cancel CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProjectHistoryCancelCliVerb {
    /// `POST /v1/project-histories/{idempotency_key}/cancel`.
    Cancel,
}

impl ProjectHistoryCancelCliVerb {
    /// Parse one exact lowercase verb token.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] for an unknown token.
    pub fn parse(token: &str) -> Result<Self, ApiError> {
        match token {
            "cancel" => Ok(Self::Cancel),
            _ => Err(ApiError::InvalidWirePayload),
        }
    }

    /// Return the canonical lowercase verb token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Cancel => "cancel",
        }
    }
}

/// One operator CLI invocation against a loopback project-history cancel POST.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectHistoryCancelCliInvocation {
    /// CLI verb to execute.
    pub verb: ProjectHistoryCancelCliVerb,
    /// Loopback `host:port` of `tepp-loopback`.
    pub host: String,
    /// Published HTTPS origin used to mint the typed cancel exchange.
    pub origin: String,
    /// Published modular consumer. Cancel POST is `LineageWeave`-only.
    pub consumer: String,
    /// Authorized tenant that minted the stored identity.
    pub tenant_workspace_id: String,
    /// Opaque idempotency key that minted the stored identity.
    pub idempotency_key: String,
    /// JSON body. Cancel POST requires empty.
    pub body: String,
}

impl ProjectHistoryCancelCliInvocation {
    /// Parse argv plus stdin body into a validated loopback cancel invocation.
    ///
    /// Empty stdin is admitted. Nonempty leftover stdin fails closed.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for unknown verbs, missing required flags, a
    /// non-loopback host, a non-`https` origin, a non-`LineageWeave` consumer,
    /// credential-shaped flags, a hostile identity, or a nonempty body.
    pub fn from_args<I, S>(args: I, body: impl Into<String>) -> Result<Self, ApiError>
    where
        I: IntoIterator<Item = S>,
        S: AsRef<str>,
    {
        let tokens: Vec<String> = args
            .into_iter()
            .map(|token| token.as_ref().to_owned())
            .collect();
        let (verb_token, rest) = tokens.split_first().ok_or(ApiError::InvalidWirePayload)?;
        let verb = ProjectHistoryCancelCliVerb::parse(verb_token)?;
        let flags = parse_flags(rest)?;
        assemble_invocation(verb, flags, body.into())
    }

    /// Reject a non-loopback host, unpublished consumer, or hostile POST body.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host and
    /// [`ApiError::InvalidWirePayload`] or [`ApiError::LimitExceeded`] for
    /// empty, unpublished, nonempty-body, or hostile identity fields.
    pub fn validate(&self) -> Result<(), ApiError> {
        require_loopback_host(&self.host)?;
        require_nonempty(&self.origin)?;
        if !self.origin.starts_with("https://") {
            return Err(ApiError::InvalidWirePayload);
        }
        require_nonempty(&self.consumer)?;
        if self.consumer != LINEAGEWEAVE_CONSUMER_CODE {
            return Err(ApiError::InvalidWirePayload);
        }
        validate_project_history_registry_identity(&self.tenant_workspace_id)?;
        validate_project_history_registry_identity(&self.idempotency_key)?;
        if self.idempotency_key.contains('/') || self.idempotency_key.contains('\0') {
            return Err(ApiError::InvalidWirePayload);
        }
        if self.idempotency_key.len() > PROJECT_HISTORY_CANCEL_ID_MAX_LEN {
            return Err(ApiError::LimitExceeded);
        }
        if !self.body.is_empty() {
            return Err(ApiError::InvalidWirePayload);
        }
        refuse_scientific_acceptance_schema(&self.body)?;
        refuse_metrics_on_project_history_collection_payload(&self.body)?;
        Ok(())
    }
}

struct ParsedFlags {
    host: Option<String>,
    origin: Option<String>,
    consumer: Option<String>,
    tenant_workspace_id: Option<String>,
    idempotency_key: Option<String>,
}

fn parse_flags(rest: &[String]) -> Result<ParsedFlags, ApiError> {
    let mut flags = ParsedFlags {
        host: None,
        origin: None,
        consumer: None,
        tenant_workspace_id: None,
        idempotency_key: None,
    };
    let mut index = 0;
    while index < rest.len() {
        let flag = rest[index].as_str();
        if !flag.starts_with("--") {
            return Err(ApiError::InvalidWirePayload);
        }
        let name = &flag[2..];
        if header_is_credential(name) {
            return Err(ApiError::AuthorizationDenied);
        }
        let slot = match name {
            "host" => &mut flags.host,
            "origin" => &mut flags.origin,
            "consumer" => &mut flags.consumer,
            "tenant-workspace-id" => &mut flags.tenant_workspace_id,
            "idempotency-key" => &mut flags.idempotency_key,
            _ => return Err(ApiError::InvalidWirePayload),
        };
        if slot.is_some() || index + 1 >= rest.len() {
            return Err(ApiError::InvalidWirePayload);
        }
        let value = rest[index + 1].as_str();
        require_nonempty(value)?;
        *slot = Some(value.to_owned());
        index += 2;
    }
    Ok(flags)
}

fn assemble_invocation(
    verb: ProjectHistoryCancelCliVerb,
    flags: ParsedFlags,
    body: String,
) -> Result<ProjectHistoryCancelCliInvocation, ApiError> {
    let invocation = ProjectHistoryCancelCliInvocation {
        verb,
        host: flags.host.ok_or(ApiError::InvalidWirePayload)?,
        origin: flags.origin.ok_or(ApiError::InvalidWirePayload)?,
        consumer: flags
            .consumer
            .unwrap_or_else(|| LINEAGEWEAVE_CONSUMER_CODE.to_owned()),
        tenant_workspace_id: flags
            .tenant_workspace_id
            .ok_or(ApiError::InvalidWirePayload)?,
        idempotency_key: flags.idempotency_key.ok_or(ApiError::InvalidWirePayload)?,
        body,
    };
    invocation.validate()?;
    Ok(invocation)
}

fn require_loopback_host(host: &str) -> Result<SocketAddr, ApiError> {
    let addr: SocketAddr = host.parse().map_err(|_| ApiError::InvalidWirePayload)?;
    if addr.ip().is_loopback() {
        Ok(addr)
    } else {
        Err(ApiError::AuthorizationDenied)
    }
}

/// Render a typed project-history cancel exchange as HTTP/1.1 for loopback.
///
/// # Errors
///
/// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host or a
/// credential-bearing header, and [`ApiError::InvalidWirePayload`] when the
/// exchange is not a POST `/v1/project-histories/{key}/cancel` with an empty
/// body.
pub fn loopback_http1_from_project_history_cancel_exchange(
    exchange: &NaruonHttpExchange,
    loopback_host: &str,
) -> Result<String, ApiError> {
    let _addr = require_loopback_host(loopback_host)?;
    let host = loopback_host.trim();
    if exchange.method != "POST" {
        return Err(ApiError::InvalidWirePayload);
    }
    if !exchange.body.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    let rest = exchange
        .target_url
        .strip_prefix("https://")
        .ok_or(ApiError::InvalidWirePayload)?;
    let path = rest
        .find('/')
        .map(|index| &rest[index..])
        .ok_or(ApiError::InvalidWirePayload)?;
    project_history_cancel_path_id(path)?;
    for (name, _) in &exchange.headers {
        if header_is_credential(name) {
            return Err(ApiError::AuthorizationDenied);
        }
        if name.eq_ignore_ascii_case("idempotency-key") {
            return Err(ApiError::InvalidWirePayload);
        }
    }
    let mut request = String::new();
    write!(
        request,
        "{} {path} HTTP/1.1\r\nHost: {host}\r\n",
        exchange.method
    )
    .map_err(|_| ApiError::InvalidWirePayload)?;
    for (name, value) in &exchange.headers {
        if name.eq_ignore_ascii_case("host") || name.eq_ignore_ascii_case("content-length") {
            continue;
        }
        write!(request, "{name}: {value}\r\n").map_err(|_| ApiError::InvalidWirePayload)?;
    }
    write!(request, "content-length: 0\r\n\r\n").map_err(|_| ApiError::InvalidWirePayload)?;
    Ok(request)
}

/// Compose one HTTP/1.1 cancel POST from the typed `LineageWeave` exchange.
///
/// # Errors
///
/// Returns the same fail-closed errors as
/// [`ProjectHistoryCancelCliInvocation::validate`].
pub fn compose_project_history_cancel_cli_http(
    invocation: &ProjectHistoryCancelCliInvocation,
) -> Result<String, ApiError> {
    invocation.validate()?;
    let exchange = lineageweave_project_history_cancel_exchange(
        &invocation.origin,
        &invocation.tenant_workspace_id,
        &invocation.idempotency_key,
    )?;
    loopback_http1_from_project_history_cancel_exchange(&exchange, &invocation.host)
}

/// Dispatch one cancel CLI invocation against an in-process listener.
///
/// # Errors
///
/// Returns fail-closed validation errors before the HTTP handler runs.
pub fn dispatch_project_history_cancel_cli(
    service: &mut AnalysisRunLiveService,
    invocation: &ProjectHistoryCancelCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let request = compose_project_history_cancel_cli_http(invocation)?;
    Ok(service.handle_http_request(&request))
}

/// Execute one cancel CLI invocation over loopback TCP.
///
/// # Errors
///
/// Returns fail-closed validation, transport, or response-framing errors.
pub fn execute_project_history_cancel_cli(
    invocation: &ProjectHistoryCancelCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let addr = require_loopback_host(&invocation.host)?;
    let request = compose_project_history_cancel_cli_http(invocation)?;
    let mut stream = TcpStream::connect(addr).map_err(|error| map_io_error(&error))?;
    stream
        .set_read_timeout(Some(NARUON_LIVE_IO_TIMEOUT))
        .map_err(|error| map_io_error(&error))?;
    stream
        .set_write_timeout(Some(NARUON_LIVE_IO_TIMEOUT))
        .map_err(|error| map_io_error(&error))?;
    stream
        .write_all(request.as_bytes())
        .map_err(|error| map_io_error(&error))?;
    stream.flush().map_err(|error| map_io_error(&error))?;
    let mut bytes = Vec::new();
    stream
        .read_to_end(&mut bytes)
        .map_err(|error| map_io_error(&error))?;
    parse_http_response(&bytes)
}

/// Filter CLI stdout so cancel never prints scientific acceptance.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a receipt carries metric keys,
/// `tepp.scientific_acceptance.v1`, or a success body that is not a metric-free
/// cancelled identity.
pub fn render_project_history_cancel_cli_stdout(
    invocation: &ProjectHistoryCancelCliInvocation,
    response: &NaruonLiveResponse,
) -> Result<String, ApiError> {
    invocation.validate()?;
    if response.body.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    refuse_scientific_acceptance_schema(&response.body)?;
    refuse_metrics_on_project_history_collection_payload(&response.body)?;
    if response.status_code != 200 {
        return Err(ApiError::InvalidWirePayload);
    }
    let parsed = ProjectHistoryCancelled::from_json(&response.body)?;
    if !parsed.cancelled {
        return Err(ApiError::InvalidWirePayload);
    }
    parsed.to_json()
}

fn refuse_scientific_acceptance_schema(body: &str) -> Result<(), ApiError> {
    if body.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA) {
        Err(ApiError::InvalidWirePayload)
    } else {
        Ok(())
    }
}

fn parse_http_response(bytes: &[u8]) -> Result<NaruonLiveResponse, ApiError> {
    let text = std::str::from_utf8(bytes).map_err(|_| ApiError::InvalidWirePayload)?;
    let (header_block, body) = text
        .split_once("\r\n\r\n")
        .ok_or(ApiError::InvalidWirePayload)?;
    let mut lines = header_block.split("\r\n");
    let status_line = lines.next().ok_or(ApiError::InvalidWirePayload)?;
    let mut parts = status_line.split(' ');
    if parts.next() != Some("HTTP/1.1") {
        return Err(ApiError::InvalidWirePayload);
    }
    let code = parts
        .next()
        .ok_or(ApiError::InvalidWirePayload)?
        .parse::<u16>()
        .map_err(|_| ApiError::InvalidWirePayload)?;
    let reason_phrase = match code {
        200 => "OK",
        202 => "Accepted",
        400 => "Bad Request",
        403 => "Forbidden",
        413 => "Payload Too Large",
        422 => "Unprocessable Entity",
        _ => return Err(ApiError::InvalidWirePayload),
    };
    let mut content_length = None;
    for line in lines {
        let (name, value) = line.split_once(':').ok_or(ApiError::InvalidWirePayload)?;
        if name.eq_ignore_ascii_case("content-length") {
            if content_length.is_some() {
                return Err(ApiError::InvalidWirePayload);
            }
            content_length = Some(
                value
                    .trim()
                    .parse::<usize>()
                    .map_err(|_| ApiError::InvalidWirePayload)?,
            );
        }
    }
    let declared = content_length.ok_or(ApiError::InvalidWirePayload)?;
    if declared != body.len() {
        return Err(ApiError::InvalidWirePayload);
    }
    Ok(NaruonLiveResponse {
        status_code: code,
        reason_phrase,
        body: body.to_owned(),
    })
}

/// Read stdin leftover bytes on a non-terminal; cancel POST admits empty.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when stdin cannot be read.
pub fn read_project_history_cancel_cli_stdin(
    stdin_is_terminal: bool,
    mut stdin: impl Read,
) -> Result<String, ApiError> {
    if stdin_is_terminal {
        Ok(String::new())
    } else {
        let mut body = String::new();
        stdin
            .read_to_string(&mut body)
            .map_err(|_| ApiError::InvalidWirePayload)?;
        Ok(body)
    }
}

#[cfg(test)]
mod tests {
    use super::{
        compose_project_history_cancel_cli_http, loopback_http1_from_project_history_cancel_exchange,
        read_project_history_cancel_cli_stdin, ProjectHistoryCancelCliInvocation,
        ProjectHistoryCancelCliVerb,
    };
    use crate::lineageweave_http::LINEAGEWEAVE_CONSUMER_CODE;
    use crate::{lineageweave_project_history_cancel_exchange, ApiError, NaruonHttpExchange};

    const ORIGIN: &str = "https://tepp.example.test";

    fn cancel_args() -> [&'static str; 11] {
        [
            "cancel",
            "--host",
            "127.0.0.1:18081",
            "--origin",
            ORIGIN,
            "--consumer",
            LINEAGEWEAVE_CONSUMER_CODE,
            "--tenant-workspace-id",
            "history-tenant",
            "--idempotency-key",
            "idem-a",
        ]
    }

    #[test]
    fn from_args_mints_cancel_and_refuses_fail_closed_inputs() {
        assert_eq!(
            ProjectHistoryCancelCliVerb::parse("cancel").expect("cancel"),
            ProjectHistoryCancelCliVerb::Cancel
        );
        assert_eq!(ProjectHistoryCancelCliVerb::Cancel.as_str(), "cancel");
        assert_eq!(
            ProjectHistoryCancelCliVerb::parse("list"),
            Err(ApiError::InvalidWirePayload)
        );
        let cancel =
            ProjectHistoryCancelCliInvocation::from_args(cancel_args(), "").expect("cancel");
        assert_eq!(cancel.verb, ProjectHistoryCancelCliVerb::Cancel);
        let http = compose_project_history_cancel_cli_http(&cancel).expect("http");
        assert!(http.starts_with("POST /v1/project-histories/idem-a/cancel HTTP/1.1"));
        assert!(http.contains("tepp-consumer: lineageweave"));
        assert!(http.contains("tepp-tenant-workspace-id: history-tenant"));
        assert!(http.contains("content-length: 0"));
        assert!(!http.contains("idempotency-key:"));
        assert!(!http.contains("authorization"));
        assert_eq!(
            ProjectHistoryCancelCliInvocation::from_args(
                [
                    "cancel",
                    "--host",
                    "8.8.8.8:80",
                    "--origin",
                    ORIGIN,
                    "--tenant-workspace-id",
                    "history-tenant",
                    "--idempotency-key",
                    "idem-a"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            ProjectHistoryCancelCliInvocation::from_args(
                [
                    "cancel",
                    "--host",
                    "localhost:18081",
                    "--origin",
                    ORIGIN,
                    "--tenant-workspace-id",
                    "history-tenant",
                    "--idempotency-key",
                    "idem-a"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ProjectHistoryCancelCliInvocation::from_args(
                [
                    "cancel",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    "http://tepp.example.test",
                    "--tenant-workspace-id",
                    "history-tenant",
                    "--idempotency-key",
                    "idem-a"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ProjectHistoryCancelCliInvocation::from_args(
                [
                    "cancel",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    ORIGIN,
                    "--tenant-workspace-id",
                    "history-tenant",
                    "--idempotency-key",
                    "idem-a",
                    "--authorization",
                    "secret"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
    }

    #[test]
    fn from_args_refuses_naruon_body_slash_and_non_post() {
        assert_eq!(
            ProjectHistoryCancelCliInvocation::from_args(
                [
                    "cancel",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    ORIGIN,
                    "--consumer",
                    "naruon",
                    "--tenant-workspace-id",
                    "history-tenant",
                    "--idempotency-key",
                    "idem-a"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ProjectHistoryCancelCliInvocation::from_args(cancel_args(), "{}").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            ProjectHistoryCancelCliInvocation::from_args(
                [
                    "cancel",
                    "--host",
                    "127.0.0.1:18081",
                    "--origin",
                    ORIGIN,
                    "--tenant-workspace-id",
                    "history-tenant",
                    "--idempotency-key",
                    "a/b"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let leftover = read_project_history_cancel_cli_stdin(false, std::io::Cursor::new(b"leftover"))
            .expect("leftover");
        assert_eq!(leftover, "leftover");
        assert!(read_project_history_cancel_cli_stdin(true, std::io::empty())
            .expect("tty")
            .is_empty());
        let exchange = lineageweave_project_history_cancel_exchange(
            ORIGIN,
            "history-tenant",
            "idem-a",
        )
        .expect("exchange");
        let gotten = NaruonHttpExchange {
            method: "GET",
            target_url: exchange.target_url,
            headers: exchange.headers,
            body: exchange.body,
        };
        assert_eq!(
            loopback_http1_from_project_history_cancel_exchange(&gotten, "127.0.0.1:18081")
                .unwrap_err(),
            ApiError::InvalidWirePayload
        );
    }
}
