"""Audit and disable orphaned GitHub Actions workflow registry identities.

Classification is bound to the exact protected default-branch SHA and tree.
Present repository workflows, GitHub-owned dynamic workflows, and identities
that changed since the last fetch are never disabled. The only accepted
credentials are the standard ``GITHUB_TOKEN`` or ``GH_TOKEN`` variables.
"""

from __future__ import annotations

import argparse
import http.client
import json
import os
import re
import ssl
import sys
import urllib.parse
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from typing import Any, Mapping, MutableMapping, Protocol, Sequence, TextIO

PROTECTED_REPOSITORY_WORKFLOW_PATHS = frozenset(
    {
        ".github/workflows/ci.yml",
        ".github/workflows/docs-quality.yml",
        ".github/workflows/hourly-nim-product-development.yml",
        ".github/workflows/hourly-pr-maintenance.yml",
    }
)
_DISABLED_WORKFLOW_STATES = frozenset(
    {
        "deleted",
        "disabled",
        "disabled_fork",
        "disabled_inactivity",
        "disabled_manually",
    }
)
_API_HOST = "api.github.com"
_LINK_ITEM = re.compile(r"<([^>]+)>\s*;\s*rel=\"([^\"]+)\"")
_WORKFLOW_PREFIX = ".github/workflows/"


class FleetAuditError(Exception):
    """Fail-closed fleet audit or mutation error with a stable reason code."""

    def __init__(self, reason: str, message: str) -> None:
        """Record *reason* and a content-redacting *message*."""

        super().__init__(message)
        self.reason = reason
        self.message = message


@dataclass(frozen=True)
class HttpResponse:
    """One GitHub HTTP response used by both live and test transports."""

    status: int
    headers: Mapping[str, str]
    body: bytes


class Transport(Protocol):
    """Minimal GitHub HTTP transport used by the auditor."""

    def request(self, method: str, path: str) -> HttpResponse:
        """Execute *method* against *path* and return the raw response."""


@dataclass(frozen=True)
class PaginationReceipt:
    """Immutable pagination evidence for one complete workflow listing."""

    page_count: int
    collected_count: int
    reported_total_count: int


@dataclass(frozen=True)
class BranchBinding:
    """Protected default-branch name and exact commit SHA."""

    default_branch: str
    sha: str


@dataclass(frozen=True)
class WorkflowIdentity:
    """One Actions registry identity plus its protected-main classification."""

    workflow_id: int
    name: str
    path: str
    state: str
    classification: str


@dataclass(frozen=True)
class FleetAudit:
    """Complete inventory bound to one default-branch SHA and timestamp."""

    owner: str
    repo: str
    branch_binding: BranchBinding
    pagination: PaginationReceipt
    identities: tuple[WorkflowIdentity, ...]
    timestamp: str

    def orphan_identities(self) -> tuple[WorkflowIdentity, ...]:
        """Return active repository-path identities absent from protected main."""

        return tuple(
            item for item in self.identities if item.classification == "orphan"
        )


@dataclass(frozen=True)
class DisableOutcome:
    """Result of planning or applying one orphan disable."""

    workflow_id: int
    path: str
    disabled: bool


class GithubHttpsTransport:
    """Live GitHub REST transport over a fixed-host HTTPS connection.

    The host is the constant ``api.github.com``; only the request path varies.
    Using ``http.client.HTTPSConnection`` avoids dynamic-URL ``urllib`` opens.
    """

    def __init__(self, token: str) -> None:
        """Store *token* for the Authorization header."""

        self.token = token

    def request(self, method: str, path: str) -> HttpResponse:
        """Call ``https://api.github.com`` *path* and return status/headers/body."""

        if not path.startswith("/"):
            raise FleetAuditError(
                "invalid_path",
                "GitHub REST paths must be absolute under the API host root",
            )
        headers = {
            "Accept": "application/vnd.github+json",
            "Authorization": f"Bearer {self.token}",
            "User-Agent": "tepp-actions-workflow-fleet",
            "X-GitHub-Api-Version": "2022-11-28",
        }
        # Fixed host only; path is absolute-root validated above. Explicit
        # default SSL context requires certificate verification (Python 3.4.3+).
        # nosemgrep: python.lang.security.audit.httpsconnection-detected.httpsconnection-detected
        connection = http.client.HTTPSConnection(
            _API_HOST,
            timeout=60,
            context=ssl.create_default_context(),
        )
        try:
            try:
                connection.request(method, path, headers=headers)
                response = connection.getresponse()
            except (OSError, http.client.HTTPException):
                raise FleetAuditError(
                    "upstream_unavailable",
                    "GitHub API transport failed",
                ) from None
            body = response.read()
            response_headers = {
                str(key): str(value) for key, value in response.getheaders()
            }
            return HttpResponse(
                status=int(response.status),
                headers=response_headers,
                body=body,
            )
        finally:
            connection.close()


def _utc_now() -> str:
    """Return the current UTC timestamp in second-resolution ISO-8601 form."""

    return datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ")


def require_github_token(environ: Mapping[str, str]) -> str:
    """Return ``GITHUB_TOKEN`` or ``GH_TOKEN``; invent no TEPP-specific secret."""

    for key in ("GITHUB_TOKEN", "GH_TOKEN"):
        value = environ.get(key, "")
        if isinstance(value, str) and value.strip():
            return value
    raise FleetAuditError(
        "missing_token",
        "GITHUB_TOKEN or GH_TOKEN is required; TEPP does not invent a PAT",
    )


def build_transport(environ: Mapping[str, str]) -> GithubHttpsTransport:
    """Build the fixed-host HTTPS transport from the standard token environment."""

    return GithubHttpsTransport(require_github_token(environ))


def normalize_workflow_path(path: str) -> str:
    """Decode percent-encoding and strip a leading ``./`` without folding case."""

    decoded = urllib.parse.unquote(path.strip())
    if decoded.startswith("./"):
        decoded = decoded[2:]
    return decoded


def is_disabled_workflow_state(state: str) -> bool:
    """Return True for GitHub disabled_* states, deleted, and the bare disabled token."""

    return state in _DISABLED_WORKFLOW_STATES


def classify_workflow(
    workflow: Mapping[str, Any], *, tree_paths: set[str]
) -> str:
    """Classify one registry identity against the protected-main workflow tree."""

    raw_path = str(workflow.get("path") or "")
    path = normalize_workflow_path(raw_path)
    state = str(workflow.get("state") or "")
    if raw_path.startswith("dynamic/") or not path.startswith(_WORKFLOW_PREFIX):
        return "github_dynamic"
    if is_disabled_workflow_state(state):
        return "disabled"
    if path in tree_paths:
        return "present"
    return "orphan"


def next_link_path(link_header: str | None) -> str | None:
    """Return the ``rel=next`` path when it stays on ``api.github.com``."""

    if not link_header:
        return None
    for match in _LINK_ITEM.finditer(link_header):
        if match.group(2) != "next":
            continue
        parsed = urllib.parse.urlparse(match.group(1))
        if parsed.scheme != "https" or parsed.netloc != _API_HOST:
            return None
        suffix = f"?{parsed.query}" if parsed.query else ""
        return f"{parsed.path}{suffix}"
    return None


def _header(headers: Mapping[str, str], name: str) -> str | None:
    """Return a header value using case-insensitive lookup."""

    wanted = name.lower()
    for key, value in headers.items():
        if key.lower() == wanted:
            return value
    return None


def _raise_for_status(response: HttpResponse) -> None:
    """Map non-success HTTP statuses onto distinct fail-closed reason codes."""

    if response.status == 204 or 200 <= response.status < 300:
        return
    if response.status == 403:
        raise FleetAuditError("permission_loss", "GitHub API returned 403")
    if response.status == 404:
        raise FleetAuditError("not_found", "GitHub API returned 404")
    if response.status >= 500:
        raise FleetAuditError("upstream_unavailable", "GitHub API returned 5xx")
    raise FleetAuditError("invalid_payload", f"GitHub API returned {response.status}")


def _parse_json_object(response: HttpResponse) -> MutableMapping[str, Any]:
    """Parse a JSON object body or fail closed as ``invalid_payload``."""

    _raise_for_status(response)
    try:
        payload = json.loads(response.body.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise FleetAuditError("invalid_payload", "response is not JSON") from error
    if not isinstance(payload, MutableMapping):
        raise FleetAuditError("invalid_payload", "response is not a JSON object")
    return payload


def _require_string(payload: Mapping[str, Any], key: str) -> str:
    """Return a non-empty string field or fail closed."""

    value = payload.get(key)
    if not isinstance(value, str) or not value:
        raise FleetAuditError("invalid_payload", f"{key} must be a non-empty string")
    return value


def _require_int(payload: Mapping[str, Any], key: str) -> int:
    """Return an integer field or fail closed."""

    value = payload.get(key)
    if isinstance(value, bool) or not isinstance(value, int):
        raise FleetAuditError("invalid_payload", f"{key} must be an integer")
    return value


def _parse_workflow(item: Any) -> dict[str, Any]:
    """Validate one workflow identity mapping from the Actions registry."""

    if not isinstance(item, Mapping):
        raise FleetAuditError("invalid_payload", "workflow entries must be objects")
    workflow_id = _require_int(item, "id")
    path = _require_string(item, "path")
    state = _require_string(item, "state")
    name = item.get("name")
    if not isinstance(name, str) or not name:
        name = path
    return {"id": workflow_id, "path": path, "state": state, "name": name}


def paginate_workflows(
    transport: Transport, owner: str, repo: str
) -> tuple[list[dict[str, Any]], PaginationReceipt]:
    """Fetch every workflow page and refuse truncated listings."""

    path = f"/repos/{owner}/{repo}/actions/workflows?per_page=100"
    collected: list[dict[str, Any]] = []
    page_count = 0
    reported_total: int | None = None
    while path:
        response = transport.request("GET", path)
        payload = _parse_json_object(response)
        if reported_total is None:
            reported_total = _require_int(payload, "total_count")
        workflows = payload.get("workflows")
        if not isinstance(workflows, list):
            raise FleetAuditError("invalid_payload", "workflows must be a list")
        collected.extend(_parse_workflow(item) for item in workflows)
        page_count += 1
        path = next_link_path(_header(response.headers, "Link"))
    if reported_total is None or len(collected) != reported_total:
        raise FleetAuditError(
            "pagination_truncated",
            "collected workflow count does not match total_count",
        )
    return collected, PaginationReceipt(
        page_count=page_count,
        collected_count=len(collected),
        reported_total_count=reported_total,
    )


def _branch_binding(transport: Transport, owner: str, repo: str) -> BranchBinding:
    """Bind the audit to the repository default branch and its exact SHA."""

    repository = _parse_json_object(transport.request("GET", f"/repos/{owner}/{repo}"))
    default_branch = _require_string(repository, "default_branch")
    encoded_branch = urllib.parse.quote(default_branch, safe="")
    reference = _parse_json_object(
        transport.request("GET", f"/repos/{owner}/{repo}/git/ref/heads/{encoded_branch}")
    )
    obj = reference.get("object")
    if not isinstance(obj, Mapping):
        raise FleetAuditError("invalid_payload", "git ref object must be an object")
    sha = _require_string(obj, "sha")
    return BranchBinding(default_branch=default_branch, sha=sha)


def _workflow_tree_paths(
    transport: Transport, owner: str, repo: str, sha: str
) -> set[str]:
    """Return protected-main ``.github/workflows`` blob paths for *sha*."""

    payload = _parse_json_object(
        transport.request("GET", f"/repos/{owner}/{repo}/git/trees/{sha}?recursive=1")
    )
    if payload.get("truncated") is True:
        raise FleetAuditError("tree_truncated", "git tree listing was truncated")
    tree = payload.get("tree")
    if not isinstance(tree, list):
        raise FleetAuditError("invalid_payload", "git tree must be a list")
    paths: set[str] = set()
    for entry in tree:
        if not isinstance(entry, Mapping):
            raise FleetAuditError("invalid_payload", "git tree entries must be objects")
        if entry.get("type") != "blob":
            continue
        raw_path = entry.get("path")
        if not isinstance(raw_path, str) or not raw_path:
            raise FleetAuditError("invalid_payload", "git tree path must be a string")
        normalized = normalize_workflow_path(raw_path)
        if normalized.startswith(_WORKFLOW_PREFIX):
            paths.add(normalized)
    return paths


def audit_repository(
    transport: Transport,
    owner: str,
    repo: str,
    *,
    now: str | None = None,
) -> FleetAudit:
    """Inventory every Actions identity against the current default-branch tree."""

    binding = _branch_binding(transport, owner, repo)
    tree_paths = _workflow_tree_paths(transport, owner, repo, binding.sha)
    workflows, pagination = paginate_workflows(transport, owner, repo)
    identities = tuple(
        WorkflowIdentity(
            workflow_id=int(item["id"]),
            name=str(item["name"]),
            path=str(item["path"]),
            state=str(item["state"]),
            classification=classify_workflow(item, tree_paths=tree_paths),
        )
        for item in workflows
    )
    return FleetAudit(
        owner=owner,
        repo=repo,
        branch_binding=binding,
        pagination=pagination,
        identities=identities,
        timestamp=now or _utc_now(),
    )


def refuse_protected_disable(identity: WorkflowIdentity) -> None:
    """Refuse to disable an exact protected production workflow path."""

    if normalize_workflow_path(identity.path) in PROTECTED_REPOSITORY_WORKFLOW_PATHS:
        raise FleetAuditError(
            "protected_workflow",
            f"refusing to disable protected workflow path {identity.path}",
        )


def _refetch_workflow(
    transport: Transport, owner: str, repo: str, workflow_id: int
) -> dict[str, Any]:
    """Re-read one workflow identity immediately before or after mutation."""

    payload = _parse_json_object(
        transport.request("GET", f"/repos/{owner}/{repo}/actions/workflows/{workflow_id}")
    )
    return _parse_workflow(payload)


def disable_orphans(
    transport: Transport,
    audit: FleetAudit,
    *,
    apply_changes: bool,
) -> list[DisableOutcome]:
    """Plan or apply disable for every audited orphan after a live re-fetch."""

    orphans = audit.orphan_identities()
    if not apply_changes:
        return [
            DisableOutcome(
                workflow_id=item.workflow_id, path=item.path, disabled=False
            )
            for item in orphans
        ]

    live_binding = _branch_binding(transport, audit.owner, audit.repo)
    if live_binding.sha != audit.branch_binding.sha:
        raise FleetAuditError(
            "branch_moved",
            "default branch SHA changed since the audit binding",
        )
    live_tree = _workflow_tree_paths(
        transport, audit.owner, audit.repo, live_binding.sha
    )
    outcomes: list[DisableOutcome] = []
    for identity in orphans:
        refuse_protected_disable(identity)
        live = _refetch_workflow(
            transport, audit.owner, audit.repo, identity.workflow_id
        )
        if (
            normalize_workflow_path(str(live["path"]))
            != normalize_workflow_path(identity.path)
            or live["state"] != "active"
        ):
            raise FleetAuditError(
                "workflow_identity_changed",
                f"workflow {identity.workflow_id} changed path or state",
            )
        if normalize_workflow_path(str(live["path"])) in live_tree:
            raise FleetAuditError(
                "no_longer_orphan",
                f"workflow {identity.workflow_id} is present on protected main",
            )
        disable_path = (
            f"/repos/{audit.owner}/{audit.repo}/actions/workflows/"
            f"{identity.workflow_id}/disable"
        )
        _raise_for_status(transport.request("PUT", disable_path))
        confirmed = _refetch_workflow(
            transport, audit.owner, audit.repo, identity.workflow_id
        )
        if not is_disabled_workflow_state(str(confirmed["state"])):
            raise FleetAuditError(
                "disable_unconfirmed",
                f"workflow {identity.workflow_id} did not become disabled",
            )
        outcomes.append(
            DisableOutcome(
                workflow_id=identity.workflow_id,
                path=identity.path,
                disabled=True,
            )
        )
    return outcomes


def audit_to_dict(audit: FleetAudit) -> dict[str, Any]:
    """Render a machine-readable inventory for recurrence evidence."""

    return {
        "owner": audit.owner,
        "repo": audit.repo,
        "timestamp": audit.timestamp,
        "default_branch": audit.branch_binding.default_branch,
        "default_branch_sha": audit.branch_binding.sha,
        "pagination": asdict(audit.pagination),
        "identities": [asdict(item) for item in audit.identities],
        "orphan_count": len(audit.orphan_identities()),
        "protected_paths": sorted(PROTECTED_REPOSITORY_WORKFLOW_PATHS),
    }


def main(
    arguments: Sequence[str] | None = None,
    *,
    environ: Mapping[str, str] | None = None,
    stdout: TextIO | None = None,
    stderr: TextIO | None = None,
) -> int:
    """CLI entry point for read-only audit or planned/applied orphan disable."""

    parser = argparse.ArgumentParser(
        description="Audit or disable orphaned GitHub Actions workflow identities"
    )
    subparsers = parser.add_subparsers(dest="command", required=True)
    audit_parser = subparsers.add_parser("audit", help="read-only inventory")
    disable_parser = subparsers.add_parser(
        "disable-orphans", help="plan or apply orphan disable"
    )
    for subparser in (audit_parser, disable_parser):
        subparser.add_argument("--owner", required=True)
        subparser.add_argument("--repo", required=True)
    disable_parser.add_argument(
        "--apply",
        action="store_true",
        help="mutate the Actions registry after live re-fetch",
    )

    out = stdout if stdout is not None else sys.stdout
    err = stderr if stderr is not None else sys.stderr
    env = dict(os.environ) if environ is None else environ
    try:
        parsed = parser.parse_args(list(arguments if arguments is not None else sys.argv[1:]))
        transport = build_transport(env)
        audit = audit_repository(transport, parsed.owner, parsed.repo)
        if parsed.command == "audit":
            json.dump(audit_to_dict(audit), out, indent=2, sort_keys=True)
            out.write("\n")
            return 0
        outcomes = disable_orphans(
            transport, audit, apply_changes=bool(parsed.apply)
        )
        json.dump(
            {
                "audit": audit_to_dict(audit),
                "outcomes": [asdict(item) for item in outcomes],
            },
            out,
            indent=2,
            sort_keys=True,
        )
        out.write("\n")
        return 0
    except FleetAuditError as error:
        print(f"{error.reason}: {error.message}", file=err)
        return 2


if __name__ == "__main__":  # pragma: no cover
    raise SystemExit(main())
