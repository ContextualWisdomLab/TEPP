//! Operator loopback CLI for analysis-run running and terminal POST.
//!
//! GAP-003A lifecycle CLI slice: operators run `tepp-lifecycle running` or
//! `tepp-lifecycle terminal` to record metric-free status from the typed
//! naruon/`LineageWeave` lifecycle exchange onto spawned `tepp-loopback` TCP.
//! Empty stdin is admitted for `running`; terminal requires typed JSON.
//! `tepp.scientific_acceptance.v1` never appears. Persistence remains GAP-003B.

use std::fmt::Write as _;
use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use crate::lineageweave_http::consumer_is_supported;
use crate::live_http::map_io_error;
use crate::naruon_http::header_is_credential;
use crate::wire::require_nonempty;
use crate::{
    ANALYSIS_RUN_ID_MAX_LEN, AnalysisRunLifecycleTransition, AnalysisRunLiveService,
    AnalysisRunStatus, AnalysisRunStatusState, ApiError, LINEAGEWEAVE_CONSUMER_CODE,
    NARUON_CONSUMER_CODE, NARUON_LIVE_IO_TIMEOUT, NaruonHttpExchange, NaruonLiveResponse,
    SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA, lineageweave_analysis_run_running_exchange,
    lineageweave_analysis_run_terminal_exchange, naruon_analysis_run_running_exchange,
    naruon_analysis_run_terminal_exchange, refuse_metrics_on_receipt,
};

/// Supported operator verbs for the loopback lifecycle CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisRunLifecycleCliVerb {
    /// `POST /v1/analysis-runs/{run_id}/running`.
    Running,
    /// `POST /v1/analysis-runs/{run_id}/terminal`.
    Terminal,
}

impl AnalysisRunLifecycleCliVerb {
    /// Parse one exact lowercase verb token.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] for an unknown token.
    pub fn parse(token: &str) -> Result<Self, ApiError> {
        match token {
            "running" => Ok(Self::Running),
            "terminal" => Ok(Self::Terminal),
            _ => Err(ApiError::InvalidWirePayload),
        }
    }

    /// Return the canonical lowercase verb token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Running => "running",
            Self::Terminal => "terminal",
        }
    }
}

/// One operator CLI invocation against a loopback lifecycle POST listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisRunLifecycleCliInvocation {
    /// CLI verb to execute.
    pub verb: AnalysisRunLifecycleCliVerb,
    /// Loopback `host:port` of `tepp-loopback`.
    pub host: String,
    /// Published HTTPS origin used to mint the typed lifecycle exchange.
    pub origin: String,
    /// Published modular consumer (`naruon` or `lineageweave`).
    pub consumer: String,
    /// Opaque server-assigned run identity.
    pub run_id: String,
    /// Exact request idempotency key.
    pub idempotency_key: String,
    /// Optional typed lifecycle JSON. Empty POST is admitted only for running.
    pub body: String,
}

impl AnalysisRunLifecycleCliInvocation {
    /// Parse argv plus stdin body into a validated loopback lifecycle invocation.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for unknown verbs, missing required flags, a
    /// non-loopback host, a non-`https` origin, an unpublished consumer,
    /// credential-shaped flags, hostile identities, metric bodies, empty
    /// terminal stdin, or a typed body that does not match the path identity
    /// and verb.
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
        let verb = AnalysisRunLifecycleCliVerb::parse(verb_token)?;
        let flags = parse_flags(rest)?;
        assemble_invocation(verb, flags, body.into())
    }

    /// Reject a non-loopback host, unpublished consumer, or hostile lifecycle body.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host and
    /// [`ApiError::InvalidWirePayload`] or [`ApiError::LimitExceeded`] for
    /// empty, unpublished, oversized, metric-bearing, or verb-mismatched fields.
    pub fn validate(&self) -> Result<(), ApiError> {
        require_loopback_host(&self.host)?;
        require_nonempty(&self.origin)?;
        if !self.origin.starts_with("https://") {
            return Err(ApiError::InvalidWirePayload);
        }
        require_nonempty(&self.consumer)?;
        if !consumer_is_supported(&self.consumer) {
            return Err(ApiError::InvalidWirePayload);
        }
        require_nonempty(&self.run_id)?;
        require_nonempty(&self.idempotency_key)?;
        if self.run_id.len() > ANALYSIS_RUN_ID_MAX_LEN {
            return Err(ApiError::LimitExceeded);
        }
        refuse_scientific_acceptance_schema(&self.body)?;
        refuse_metrics_on_receipt(&self.body)?;
        if self.body.is_empty() {
            return match self.verb {
                AnalysisRunLifecycleCliVerb::Running => Ok(()),
                AnalysisRunLifecycleCliVerb::Terminal => Err(ApiError::InvalidWirePayload),
            };
        }
        let transition = AnalysisRunLifecycleTransition::from_json(&self.body)?;
        if transition.run_id != self.run_id || transition.idempotency_key != self.idempotency_key {
            return Err(ApiError::InvalidWirePayload);
        }
        match self.verb {
            AnalysisRunLifecycleCliVerb::Running => {
                if transition.run_state == AnalysisRunStatusState::Running {
                    Ok(())
                } else {
                    Err(ApiError::InvalidWirePayload)
                }
            }
            AnalysisRunLifecycleCliVerb::Terminal => match transition.run_state {
                AnalysisRunStatusState::Succeeded | AnalysisRunStatusState::Failed => Ok(()),
                AnalysisRunStatusState::Accepted | AnalysisRunStatusState::Running => {
                    Err(ApiError::InvalidWirePayload)
                }
            },
        }
    }
}

struct ParsedFlags {
    host: Option<String>,
    origin: Option<String>,
    consumer: Option<String>,
    run_id: Option<String>,
    idempotency_key: Option<String>,
}

fn parse_flags(rest: &[String]) -> Result<ParsedFlags, ApiError> {
    let mut flags = ParsedFlags {
        host: None,
        origin: None,
        consumer: None,
        run_id: None,
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
            "run-id" => &mut flags.run_id,
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
    verb: AnalysisRunLifecycleCliVerb,
    flags: ParsedFlags,
    body: String,
) -> Result<AnalysisRunLifecycleCliInvocation, ApiError> {
    let invocation = AnalysisRunLifecycleCliInvocation {
        verb,
        host: flags.host.ok_or(ApiError::InvalidWirePayload)?,
        origin: flags.origin.ok_or(ApiError::InvalidWirePayload)?,
        consumer: flags
            .consumer
            .unwrap_or_else(|| NARUON_CONSUMER_CODE.to_owned()),
        run_id: flags.run_id.ok_or(ApiError::InvalidWirePayload)?,
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

fn lifecycle_transition(
    invocation: &AnalysisRunLifecycleCliInvocation,
) -> Result<AnalysisRunLifecycleTransition, ApiError> {
    if invocation.body.is_empty() {
        AnalysisRunLifecycleTransition::running(&invocation.run_id, &invocation.idempotency_key)
    } else {
        AnalysisRunLifecycleTransition::from_json(&invocation.body)
    }
}

fn lifecycle_exchange(
    invocation: &AnalysisRunLifecycleCliInvocation,
) -> Result<NaruonHttpExchange, ApiError> {
    let transition = lifecycle_transition(invocation)?;
    let lineage = invocation.consumer == LINEAGEWEAVE_CONSUMER_CODE;
    let naruon = invocation.consumer == NARUON_CONSUMER_CODE;
    match (invocation.verb, lineage, naruon) {
        (AnalysisRunLifecycleCliVerb::Running, true, false) => {
            lineageweave_analysis_run_running_exchange(&invocation.origin, &transition)
        }
        (AnalysisRunLifecycleCliVerb::Terminal, true, false) => {
            lineageweave_analysis_run_terminal_exchange(&invocation.origin, &transition)
        }
        (AnalysisRunLifecycleCliVerb::Running, false, true) => {
            naruon_analysis_run_running_exchange(&invocation.origin, &transition)
        }
        (AnalysisRunLifecycleCliVerb::Terminal, false, true) => {
            naruon_analysis_run_terminal_exchange(&invocation.origin, &transition)
        }
        _ => Err(ApiError::InvalidWirePayload),
    }
}

/// Render a typed lifecycle exchange as HTTP/1.1 for a bound loopback listener.
///
/// The exchange keeps its HTTPS origin contract. Only the HTTP/1.1 `Host` is
/// the loopback bind address. Public bind hosts fail closed.
///
/// # Errors
///
/// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host or a
/// credential-bearing header, and [`ApiError::InvalidWirePayload`] when the
/// exchange is not a POST `/running` or `/terminal`.
pub fn loopback_http1_from_lifecycle_exchange(
    exchange: &NaruonHttpExchange,
    loopback_host: &str,
) -> Result<String, ApiError> {
    let _addr = require_loopback_host(loopback_host)?;
    let host = loopback_host.trim();
    if exchange.method != "POST" {
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
    let suffix = path.rsplit('/').next();
    if suffix != Some("running") && suffix != Some("terminal") {
        return Err(ApiError::InvalidWirePayload);
    }
    for (name, _) in &exchange.headers {
        if header_is_credential(name) {
            return Err(ApiError::AuthorizationDenied);
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
    write!(
        request,
        "content-length: {}\r\n\r\n{}",
        exchange.body.len(),
        exchange.body
    )
    .map_err(|_| ApiError::InvalidWirePayload)?;
    Ok(request)
}

/// Compose one HTTP/1.1 lifecycle POST from the typed consumer exchange.
///
/// # Errors
///
/// Returns the same fail-closed errors as
/// [`AnalysisRunLifecycleCliInvocation::validate`].
pub fn compose_analysis_run_lifecycle_cli_http(
    invocation: &AnalysisRunLifecycleCliInvocation,
) -> Result<String, ApiError> {
    invocation.validate()?;
    let exchange = lifecycle_exchange(invocation)?;
    loopback_http1_from_lifecycle_exchange(&exchange, &invocation.host)
}

/// Dispatch one lifecycle CLI invocation against an in-process loopback service.
///
/// # Errors
///
/// Returns fail-closed validation errors before the HTTP handler runs.
pub fn dispatch_analysis_run_lifecycle_cli(
    service: &mut AnalysisRunLiveService,
    invocation: &AnalysisRunLifecycleCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let request = compose_analysis_run_lifecycle_cli_http(invocation)?;
    Ok(service.handle_http_request(&request))
}

/// Execute one lifecycle CLI invocation over loopback TCP against `tepp-loopback`.
///
/// # Errors
///
/// Returns fail-closed validation, transport, or response-framing errors.
pub fn execute_analysis_run_lifecycle_cli(
    invocation: &AnalysisRunLifecycleCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let addr = require_loopback_host(&invocation.host)?;
    let request = compose_analysis_run_lifecycle_cli_http(invocation)?;
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

/// Filter CLI stdout so lifecycle receipts never print scientific acceptance.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a receipt carries metric keys,
/// `tepp.scientific_acceptance.v1`, or a success body that is not a metric-free
/// `200` status matching the verb.
pub fn render_analysis_run_lifecycle_cli_stdout(
    invocation: &AnalysisRunLifecycleCliInvocation,
    response: &NaruonLiveResponse,
) -> Result<String, ApiError> {
    invocation.validate()?;
    if response.body.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    refuse_scientific_acceptance_schema(&response.body)?;
    refuse_metrics_on_receipt(&response.body)?;
    if !(200..300).contains(&response.status_code) {
        return Ok(response.body.clone());
    }
    if response.status_code != 200 {
        return Err(ApiError::InvalidWirePayload);
    }
    let status = AnalysisRunStatus::from_json(&response.body)?;
    if status.run_id != invocation.run_id || status.idempotency_key != invocation.idempotency_key {
        return Err(ApiError::InvalidWirePayload);
    }
    match invocation.verb {
        AnalysisRunLifecycleCliVerb::Running => {
            if status.run_state != AnalysisRunStatusState::Running {
                return Err(ApiError::InvalidWirePayload);
            }
        }
        AnalysisRunLifecycleCliVerb::Terminal => match status.run_state {
            AnalysisRunStatusState::Succeeded | AnalysisRunStatusState::Failed => {}
            AnalysisRunStatusState::Accepted | AnalysisRunStatusState::Running => {
                return Err(ApiError::InvalidWirePayload);
            }
        },
    }
    status.to_json()
}

fn refuse_scientific_acceptance_schema(body: &str) -> Result<(), ApiError> {
    if body.contains(SCIENTIFIC_ACCEPTANCE_HTTP_SCHEMA) {
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

/// Read stdin leftover bytes on a non-terminal; empty running POST is admitted.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when stdin cannot be read.
pub fn read_analysis_run_lifecycle_cli_stdin(
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
