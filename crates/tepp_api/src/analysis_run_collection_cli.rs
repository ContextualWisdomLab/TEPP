//! Operator loopback CLI for analysis-run collection GET.
//!
//! GAP-003A eighth slice: operators run `tepp-analysis-runs list` to enumerate
//! accepted, running, cancelled, and terminal runs without writing raw HTTP or
//! guessing run identities. Stdout stays metric-free. `tepp.scientific_acceptance.v1`
//! never appears. This module does not duplicate GET-by-id (#359), lifecycle
//! POST (#360), cancel HTTP (#361), scientific-acceptance CLI (#362), or the
//! collection GET listener (#368). Persistence remains GAP-003B.

use std::io::{Read, Write};
use std::net::{SocketAddr, TcpStream};

use crate::analysis_run_collection_http::{
    parse_collection_page_cursor, parse_collection_page_limit, refuse_metrics_on_collection_payload,
};
use crate::lineageweave_http::consumer_is_supported;
use crate::live_http::map_io_error;
use crate::naruon_http::{NARUON_ANALYSIS_RUN_PATH, header_is_credential};
use crate::wire::require_nonempty;
use crate::{
    AnalysisRunCollection, AnalysisRunLiveService, ApiError, NARUON_LIVE_IO_TIMEOUT,
    NaruonLiveResponse,
};

const SCIENTIFIC_ACCEPTANCE_SCHEMA: &str = "tepp.scientific_acceptance.v1";

/// Supported operator verbs for the loopback collection CLI.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AnalysisRunCollectionCliVerb {
    /// `GET /v1/analysis-runs`.
    List,
}

impl AnalysisRunCollectionCliVerb {
    /// Parse one exact lowercase verb token.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::InvalidWirePayload`] for an unknown token.
    pub fn parse(token: &str) -> Result<Self, ApiError> {
        match token {
            "list" => Ok(Self::List),
            _ => Err(ApiError::InvalidWirePayload),
        }
    }

    /// Return the canonical lowercase verb token.
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::List => "list",
        }
    }
}

/// One operator CLI invocation against a loopback collection GET listener.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AnalysisRunCollectionCliInvocation {
    /// CLI verb to execute.
    pub verb: AnalysisRunCollectionCliVerb,
    /// Loopback `host:port` of `tepp-loopback`.
    pub host: String,
    /// Published modular consumer (`naruon` or `lineageweave`).
    pub consumer: String,
    /// Optional exclusive page cursor (`tepp-page-cursor`).
    pub page_cursor: Option<String>,
    /// Optional page limit (`tepp-page-limit`).
    pub page_limit: Option<String>,
    /// JSON body. Collection GET requires empty.
    pub body: String,
}

impl AnalysisRunCollectionCliInvocation {
    /// Parse argv plus stdin body into a validated loopback collection invocation.
    ///
    /// # Errors
    ///
    /// Returns a fail-closed error for unknown verbs, missing required flags, a
    /// non-loopback host, an unpublished consumer, credential-shaped flags,
    /// hostile pagination, or a nonempty body.
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
        let verb = AnalysisRunCollectionCliVerb::parse(verb_token)?;
        let flags = parse_flags(rest)?;
        assemble_invocation(verb, flags, body.into())
    }

    /// Reject a non-loopback host, unpublished consumer, or hostile page flags.
    ///
    /// # Errors
    ///
    /// Returns [`ApiError::AuthorizationDenied`] for a non-loopback host and
    /// [`ApiError::InvalidWirePayload`] or [`ApiError::LimitExceeded`] for
    /// empty, unpublished, or out-of-bounds fields.
    pub fn validate(&self) -> Result<(), ApiError> {
        require_loopback_host(&self.host)?;
        require_nonempty(&self.consumer)?;
        if !consumer_is_supported(&self.consumer) {
            return Err(ApiError::InvalidWirePayload);
        }
        if !self.body.is_empty() {
            return Err(ApiError::InvalidWirePayload);
        }
        refuse_metrics_on_collection_payload(&self.body)?;
        parse_collection_page_limit(self.page_limit.as_deref())?;
        parse_collection_page_cursor(self.page_cursor.as_deref())?;
        Ok(())
    }
}

struct ParsedFlags {
    host: Option<String>,
    consumer: Option<String>,
    page_cursor: Option<String>,
    page_limit: Option<String>,
}

fn parse_flags(rest: &[String]) -> Result<ParsedFlags, ApiError> {
    let mut flags = ParsedFlags {
        host: None,
        consumer: None,
        page_cursor: None,
        page_limit: None,
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
            "consumer" => &mut flags.consumer,
            "page-cursor" => &mut flags.page_cursor,
            "page-limit" => &mut flags.page_limit,
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
    verb: AnalysisRunCollectionCliVerb,
    flags: ParsedFlags,
    body: String,
) -> Result<AnalysisRunCollectionCliInvocation, ApiError> {
    let invocation = AnalysisRunCollectionCliInvocation {
        verb,
        host: flags.host.ok_or(ApiError::InvalidWirePayload)?,
        consumer: flags
            .consumer
            .unwrap_or_else(|| crate::NARUON_CONSUMER_CODE.to_owned()),
        page_cursor: flags.page_cursor,
        page_limit: flags.page_limit,
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

/// Compose one HTTP/1.1 collection GET for a validated CLI invocation.
///
/// # Errors
///
/// Returns the same fail-closed errors as
/// [`AnalysisRunCollectionCliInvocation::validate`].
pub fn compose_analysis_run_collection_cli_http(
    invocation: &AnalysisRunCollectionCliInvocation,
) -> Result<String, ApiError> {
    invocation.validate()?;
    let mut request = format!(
        "GET {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\nHost: {}\r\ncontent-type: application/json\r\ntepp-consumer: {}\r\ntepp-contract-version: 1\r\n",
        invocation.host, invocation.consumer
    );
    if let Some(cursor) = &invocation.page_cursor {
        request.push_str("tepp-page-cursor: ");
        request.push_str(cursor);
        request.push_str("\r\n");
    }
    if let Some(limit) = &invocation.page_limit {
        request.push_str("tepp-page-limit: ");
        request.push_str(limit);
        request.push_str("\r\n");
    }
    request.push_str("content-length: 0\r\n\r\n");
    Ok(request)
}

/// Dispatch one collection CLI invocation against an in-process loopback service.
///
/// # Errors
///
/// Returns fail-closed validation errors before the HTTP handler runs.
pub fn dispatch_analysis_run_collection_cli(
    service: &mut AnalysisRunLiveService,
    invocation: &AnalysisRunCollectionCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let request = compose_analysis_run_collection_cli_http(invocation)?;
    Ok(service.handle_http_request(&request))
}

/// Execute one collection CLI invocation over loopback TCP against `tepp-loopback`.
///
/// # Errors
///
/// Returns fail-closed validation, transport, or response-framing errors.
pub fn execute_analysis_run_collection_cli(
    invocation: &AnalysisRunCollectionCliInvocation,
) -> Result<NaruonLiveResponse, ApiError> {
    let addr = require_loopback_host(&invocation.host)?;
    let request = compose_analysis_run_collection_cli_http(invocation)?;
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

/// Filter CLI stdout so collection pages never print scientific acceptance.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when a receipt carries metric keys
/// or `tepp.scientific_acceptance.v1`.
pub fn render_analysis_run_collection_cli_stdout(
    invocation: &AnalysisRunCollectionCliInvocation,
    response: &NaruonLiveResponse,
) -> Result<String, ApiError> {
    invocation.validate()?;
    if response.body.is_empty() {
        return Err(ApiError::InvalidWirePayload);
    }
    refuse_scientific_acceptance_schema(&response.body)?;
    if !(200..300).contains(&response.status_code) {
        refuse_metrics_on_collection_payload(&response.body)?;
        return Ok(response.body.clone());
    }
    let collection = AnalysisRunCollection::from_json(&response.body)?;
    collection.to_json()
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
    let reason_phrase = static_reason(code)?;
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

fn static_reason(code: u16) -> Result<&'static str, ApiError> {
    match code {
        200 => Ok("OK"),
        202 => Ok("Accepted"),
        400 => Ok("Bad Request"),
        403 => Ok("Forbidden"),
        413 => Ok("Payload Too Large"),
        422 => Ok("Unprocessable Entity"),
        _ => Err(ApiError::InvalidWirePayload),
    }
}

/// Read stdin leftover bytes on a non-terminal; collection GET refuses a body.
///
/// # Errors
///
/// Returns [`ApiError::InvalidWirePayload`] when stdin cannot be read.
pub fn read_analysis_run_collection_cli_stdin(
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
#[allow(clippy::too_many_lines)]
mod tests {
    use super::{
        AnalysisRunCollectionCliInvocation, AnalysisRunCollectionCliVerb,
        SCIENTIFIC_ACCEPTANCE_SCHEMA, compose_analysis_run_collection_cli_http,
        dispatch_analysis_run_collection_cli, execute_analysis_run_collection_cli,
        parse_http_response, read_analysis_run_collection_cli_stdin,
        render_analysis_run_collection_cli_stdout, static_reason,
    };
    use crate::{
        ANALYSIS_RUN_COLLECTION_MAX_LIMIT, ANALYSIS_RUN_CONTRACT_VERSION, AnalysisRunAccepted,
        AnalysisRunCollection, AnalysisRunLiveService, AnalysisRunRequest, ApiError,
        LINEAGEWEAVE_CONSUMER_CODE, NARUON_ANALYSIS_RUN_PATH, NARUON_CONSUMER_CODE,
        NaruonLiveResponse,
    };

    fn request(idempotency_key: &str) -> AnalysisRunRequest {
        AnalysisRunRequest {
            contract_version: ANALYSIS_RUN_CONTRACT_VERSION,
            idempotency_key: idempotency_key.into(),
            tenant_workspace_id: "cli-collection-tenant".into(),
            snapshot_id: "cli-collection-snapshot".into(),
            knowledge_cutoff: "2026-08-01T00:00:00Z".into(),
            model_contract_version: "tepp-analysis-run-v1".into(),
            output_profile: "calibrated_event_measurement".into(),
        }
    }

    fn create_http(run: &AnalysisRunRequest, consumer: &str, host: &str) -> String {
        let body = run.to_json().expect("json");
        format!(
            "POST {NARUON_ANALYSIS_RUN_PATH} HTTP/1.1\r\nHost: {host}\r\ncontent-type: application/json\r\ntepp-consumer: {consumer}\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: {}\r\n\r\n{body}",
            run.idempotency_key,
            body.len()
        )
    }

    fn list_invocation() -> AnalysisRunCollectionCliInvocation {
        AnalysisRunCollectionCliInvocation::from_args(["list", "--host", "127.0.0.1:18081"], "")
            .expect("list")
    }

    #[test]
    fn verbs_parse_and_reject_unknown_tokens() {
        assert_eq!(
            AnalysisRunCollectionCliVerb::parse("list").expect("list"),
            AnalysisRunCollectionCliVerb::List
        );
        assert_eq!(AnalysisRunCollectionCliVerb::List.as_str(), "list");
        assert_eq!(
            AnalysisRunCollectionCliVerb::parse("LIST"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunCollectionCliVerb::parse("create"),
            Err(ApiError::InvalidWirePayload)
        );
        assert_eq!(
            AnalysisRunCollectionCliVerb::parse("status"),
            Err(ApiError::InvalidWirePayload)
        );
    }

    #[test]
    fn from_args_refuses_empty_unknown_host_and_credential_flags() {
        assert_eq!(
            AnalysisRunCollectionCliInvocation::from_args(Vec::<String>::new(), "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCollectionCliInvocation::from_args(["nope"], "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCollectionCliInvocation::from_args(["list"], "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCollectionCliInvocation::from_args(["list", "--host"], "").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCollectionCliInvocation::from_args(["list", "--host", "8.8.8.8:80"], "")
                .unwrap_err(),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            AnalysisRunCollectionCliInvocation::from_args(["list", "--host", "not-a-socket"], "")
                .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCollectionCliInvocation::from_args(
                [
                    "list",
                    "--host",
                    "127.0.0.1:18081",
                    "--authorization",
                    "secret"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::AuthorizationDenied
        );
        assert_eq!(
            AnalysisRunCollectionCliInvocation::from_args(
                ["list", "--host", "127.0.0.1:18081", "--pretty"],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCollectionCliInvocation::from_args(
                ["list", "--host", "127.0.0.1:18081", "extra"],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCollectionCliInvocation::from_args(
                [
                    "list",
                    "--host",
                    "127.0.0.1:18081",
                    "--run-id",
                    "tepp-run-1"
                ],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCollectionCliInvocation::from_args(
                ["list", "--host", "127.0.0.1:18081"],
                "{}"
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCollectionCliInvocation::from_args(
                ["list", "--host", "127.0.0.1:18081", "--host", "127.0.0.1:9"],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCollectionCliInvocation::from_args(
                ["list", "--host", "127.0.0.1:18081", "--consumer", "other"],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCollectionCliInvocation::from_args(
                ["list", "--host", "127.0.0.1:18081", "--page-limit", "0"],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCollectionCliInvocation::from_args(
                [
                    "list",
                    "--host",
                    "127.0.0.1:18081",
                    "--page-limit",
                    &(ANALYSIS_RUN_COLLECTION_MAX_LIMIT + 1).to_string()
                ],
                ""
            )
            .unwrap_err(),
            ApiError::LimitExceeded
        );
        assert_eq!(
            AnalysisRunCollectionCliInvocation::from_args(
                ["list", "--host", "127.0.0.1:18081", "--page-limit", "two"],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            AnalysisRunCollectionCliInvocation::from_args(
                ["list", "--host", "127.0.0.1:18081", "--page-cursor", ""],
                ""
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
    }

    #[test]
    fn list_assembles_default_consumer_and_optional_page_headers() {
        let list = list_invocation();
        assert_eq!(list.verb, AnalysisRunCollectionCliVerb::List);
        assert_eq!(list.consumer, NARUON_CONSUMER_CODE);
        assert!(list.page_cursor.is_none());
        assert!(list.page_limit.is_none());
        let http = compose_analysis_run_collection_cli_http(&list).expect("http");
        assert!(http.starts_with("GET /v1/analysis-runs HTTP/1.1"));
        assert!(http.contains("tepp-consumer: naruon"));
        assert!(!http.contains("idempotency-key"));
        assert!(!http.contains("tepp-page-cursor"));
        assert!(!http.contains("tepp-page-limit"));

        let paged = AnalysisRunCollectionCliInvocation::from_args(
            [
                "list",
                "--host",
                "127.0.0.1:18081",
                "--consumer",
                LINEAGEWEAVE_CONSUMER_CODE,
                "--page-cursor",
                "tepp-run-1",
                "--page-limit",
                "8",
            ],
            "",
        )
        .expect("paged");
        assert_eq!(paged.consumer, LINEAGEWEAVE_CONSUMER_CODE);
        let paged_http = compose_analysis_run_collection_cli_http(&paged).expect("paged http");
        assert!(paged_http.contains("tepp-page-cursor: tepp-run-1"));
        assert!(paged_http.contains("tepp-page-limit: 8"));
        assert!(paged_http.contains("tepp-consumer: lineageweave"));
    }

    #[test]
    fn dispatch_lists_created_and_cancelled_runs_without_scientific_acceptance() {
        let mut service = AnalysisRunLiveService::new();
        let first = request("cli-collection-idem-1");
        let created = service.handle_http_request(&create_http(
            &first,
            NARUON_CONSUMER_CODE,
            "127.0.0.1:18081",
        ));
        assert_eq!(created.status_code, 202);
        let accepted = AnalysisRunAccepted::from_json(&created.body).expect("accepted");

        let empty_before_second =
            dispatch_analysis_run_collection_cli(&mut service, &list_invocation())
                .expect("list one");
        assert_eq!(empty_before_second.status_code, 200);
        let listed =
            render_analysis_run_collection_cli_stdout(&list_invocation(), &empty_before_second)
                .expect("stdout");
        assert!(!listed.contains(SCIENTIFIC_ACCEPTANCE_SCHEMA));
        assert!(!listed.contains("rmse"));
        assert!(!listed.contains("terminal_result"));
        let page = AnalysisRunCollection::from_json(&listed).expect("page");
        assert_eq!(page.runs.len(), 1);
        assert_eq!(page.runs[0].run_id, accepted.run_id);
        assert_eq!(
            page.runs[0].run_state,
            crate::AnalysisRunStatusState::Accepted
        );

        let second = request("cli-collection-idem-2");
        let created_second = service.handle_http_request(&create_http(
            &second,
            NARUON_CONSUMER_CODE,
            "127.0.0.1:18081",
        ));
        let accepted_second =
            AnalysisRunAccepted::from_json(&created_second.body).expect("accepted second");
        let cancel = format!(
            "POST {NARUON_ANALYSIS_RUN_PATH}/{}/cancel HTTP/1.1\r\nHost: 127.0.0.1:18081\r\ncontent-type: application/json\r\ntepp-consumer: {NARUON_CONSUMER_CODE}\r\ntepp-contract-version: 1\r\nidempotency-key: {}\r\ncontent-length: 0\r\n\r\n",
            accepted_second.run_id, second.idempotency_key
        );
        let cancelled = service.handle_http_request(&cancel);
        assert_eq!(cancelled.status_code, 200);

        let paged = AnalysisRunCollectionCliInvocation::from_args(
            ["list", "--host", "127.0.0.1:18081", "--page-limit", "1"],
            "",
        )
        .expect("limit 1");
        let first_page =
            dispatch_analysis_run_collection_cli(&mut service, &paged).expect("page 1");
        let first_json =
            render_analysis_run_collection_cli_stdout(&paged, &first_page).expect("page 1 stdout");
        let first_collection = AnalysisRunCollection::from_json(&first_json).expect("first page");
        assert_eq!(first_collection.runs.len(), 1);
        let cursor = first_collection.next_cursor.expect("cursor");
        let second_page_invocation = AnalysisRunCollectionCliInvocation::from_args(
            [
                "list",
                "--host",
                "127.0.0.1:18081",
                "--page-cursor",
                cursor.as_str(),
                "--page-limit",
                "1",
            ],
            "",
        )
        .expect("page 2");
        let second_page =
            dispatch_analysis_run_collection_cli(&mut service, &second_page_invocation)
                .expect("page 2");
        let second_json =
            render_analysis_run_collection_cli_stdout(&second_page_invocation, &second_page)
                .expect("page 2 stdout");
        let second_collection =
            AnalysisRunCollection::from_json(&second_json).expect("second page");
        assert_eq!(second_collection.runs.len(), 1);
        assert!(
            second_collection
                .runs
                .iter()
                .any(|row| row.run_state == crate::AnalysisRunStatusState::Cancelled)
                || first_collection
                    .runs
                    .iter()
                    .any(|row| row.run_state == crate::AnalysisRunStatusState::Cancelled)
        );

        let other = AnalysisRunCollectionCliInvocation::from_args(
            [
                "list",
                "--host",
                "127.0.0.1:18081",
                "--consumer",
                LINEAGEWEAVE_CONSUMER_CODE,
            ],
            "",
        )
        .expect("other consumer");
        let isolated =
            dispatch_analysis_run_collection_cli(&mut service, &other).expect("isolated");
        let isolated_stdout =
            render_analysis_run_collection_cli_stdout(&other, &isolated).expect("isolated stdout");
        let isolated_page = AnalysisRunCollection::from_json(&isolated_stdout).expect("empty");
        assert!(isolated_page.runs.is_empty());
    }

    #[test]
    fn render_refuses_metrics_scientific_acceptance_and_empty_bodies() {
        let list = list_invocation();
        assert_eq!(
            render_analysis_run_collection_cli_stdout(
                &list,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: String::new(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_analysis_run_collection_cli_stdout(
                &list,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: r#"{"contract_version":1,"runs":[],"rmse":1.0}"#.into(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_analysis_run_collection_cli_stdout(
                &list,
                &NaruonLiveResponse {
                    status_code: 200,
                    reason_phrase: "OK",
                    body: format!(
                        r#"{{"contract_version":1,"runs":[],"schema_version":"{SCIENTIFIC_ACCEPTANCE_SCHEMA}"}}"#
                    ),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let error_stdout = render_analysis_run_collection_cli_stdout(
            &list,
            &NaruonLiveResponse {
                status_code: 400,
                reason_phrase: "Bad Request",
                body: r#"{"error_code":"invalid_wire_payload"}"#.into(),
            },
        )
        .expect("error");
        assert!(error_stdout.contains("invalid_wire_payload"));
        assert_eq!(
            render_analysis_run_collection_cli_stdout(
                &list,
                &NaruonLiveResponse {
                    status_code: 400,
                    reason_phrase: "Bad Request",
                    body: format!(r#"{{"schema_version":"{SCIENTIFIC_ACCEPTANCE_SCHEMA}"}}"#),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            render_analysis_run_collection_cli_stdout(
                &list,
                &NaruonLiveResponse {
                    status_code: 400,
                    reason_phrase: "Bad Request",
                    body: r#"{"rmse":0.1}"#.into(),
                }
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        let empty_ok = render_analysis_run_collection_cli_stdout(
            &list,
            &NaruonLiveResponse {
                status_code: 200,
                reason_phrase: "OK",
                body: r#"{"contract_version":1,"runs":[]}"#.into(),
            },
        )
        .expect("empty");
        assert!(empty_ok.contains("\"runs\":[]"));
    }

    #[test]
    fn execute_over_tcp_and_parse_response_failures() {
        let mut service = AnalysisRunLiveService::bind_loopback().expect("bind");
        let addr = service.local_addr().expect("addr");
        let handle = std::thread::spawn(move || {
            drop(service.serve_one());
        });
        let mut invocation = list_invocation();
        invocation.host = addr.to_string();
        let response = execute_analysis_run_collection_cli(&invocation).expect("tcp");
        assert_eq!(response.status_code, 200);
        handle.join().expect("join");

        invocation.host = "127.0.0.1:1".into();
        assert_eq!(
            execute_analysis_run_collection_cli(&invocation).unwrap_err(),
            ApiError::InvalidWirePayload
        );

        let parsed =
            parse_http_response(b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\n\r\n{}").expect("parse");
        assert_eq!(parsed.status_code, 200);
        assert_eq!(
            parse_http_response(b"not-http").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(b"HTTP/1.0 200 OK\r\ncontent-length: 2\r\n\r\n{}").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(b"HTTP/1.1 299 Mystery\r\ncontent-length: 2\r\n\r\n{}")
                .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(
                b"HTTP/1.1 200 OK\r\ncontent-length: 2\r\ncontent-length: 2\r\n\r\n{}"
            )
            .unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(b"HTTP/1.1 200 OK\r\ncontent-length: 9\r\n\r\n{}").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(b"HTTP/1.1 200 OK\r\nbad-header\r\n\r\n{}").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(&[0xff, 0xfe]).unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(static_reason(200).expect("200"), "OK");
        assert_eq!(static_reason(202).expect("202"), "Accepted");
        assert_eq!(static_reason(400).expect("400"), "Bad Request");
        assert_eq!(static_reason(403).expect("403"), "Forbidden");
        assert_eq!(static_reason(413).expect("413"), "Payload Too Large");
        assert_eq!(static_reason(422).expect("422"), "Unprocessable Entity");
        assert_eq!(
            static_reason(500).unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(b"HTTP/1.1\r\ncontent-length: 0\r\n\r\n").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(b"HTTP/1.1 abc OK\r\ncontent-length: 0\r\n\r\n").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(b"HTTP/1.1 200 OK\r\ncontent-length: x\r\n\r\n").unwrap_err(),
            ApiError::InvalidWirePayload
        );
        assert_eq!(
            parse_http_response(b"HTTP/1.1 200 OK\r\nhost: 127.0.0.1\r\n\r\n{}").unwrap_err(),
            ApiError::InvalidWirePayload
        );
    }

    #[test]
    fn stdin_reader_skips_terminal_and_reads_otherwise() {
        let empty = read_analysis_run_collection_cli_stdin(true, std::io::empty()).expect("tty");
        assert!(empty.is_empty());
        let piped =
            read_analysis_run_collection_cli_stdin(false, std::io::Cursor::new(b"leftover"))
                .expect("piped");
        assert_eq!(piped, "leftover");
        let piped_empty = read_analysis_run_collection_cli_stdin(false, std::io::Cursor::new(b""))
            .expect("empty");
        assert!(piped_empty.is_empty());
    }
}
