# paperless-rs as a separate Railway service for report-rs: document management and intake (v1)

**Status:** PLAN. Nothing in report-rs changes under this plan until its own decision is recorded (report-rs decisions are Linear issues in `P-EXO-9`; see §9).
**Date:** 2026-11-22
**Scope:** the paperless-rs Railway service (this repository's `Dockerfile`, which builds tesseract-rs's `tesseract-paperless-web`), its CI, its integration seam with report-rs, and a later optional WordCraft editor inside paperless-rs.

## 0. Decision in one paragraph

Run document management and intake as **its own Railway service**, built from this repository. This repository owns the deployment (Dockerfile, settings, CI). tesseract-rs owns the binary and the archive (`tesseract-paperless-web`: upload → S-2 dedup gate → OCR → lancedb archive → Tantivy search). report-rs stays a separate service and talks to it **server-to-server**, the same way it already talks to the ticket service (`src/ticket_api.rs`, RPT-TICKET-API-1). report-rs stores only a **document reference** (the content hash), never document bytes or text. CI copies report-rs's model (RPT-CI-RAILWAY-1): one `scripts/ci/validate.sh`, run inside the Railway image build, so a failing check fails the deployment. **WordCraft** may later become the document editor **inside paperless-rs**; report-rs never depends on it.

## 1. Services and ownership

| Service | Built from | Owns | Never owns |
|---|---|---|---|
| **report-rs** (production) | `AdaWorldAPI/report-rs` | the reporting cockpit, PostgreSQL, Zitadel sign-in, the report matrix | document bytes, OCR, the archive, an editor |
| **paperless** (new) | `AdaWorldAPI/paperless-rs` → `Dockerfile` (clones tesseract-rs and its siblings) | document intake, dedup, OCR, archive (lancedb on a volume), search (Tantivy), filing metadata, later the editor | report data, report-rs's database, identity |
| tesseract-rs | (library + binary, no service of its own here) | recognition and `tesseract-paperless-web` | deployment settings |

The direction of dependency stays as this repository's CLAUDE.md states it: settings and the Dockerfile flow **out** of paperless-rs; tesseract-rs stays deployment-agnostic; report-rs depends on neither repository's code.

## 2. What exists today (verified 2026-11-22)

| Item | State | Where |
|---|---|---|
| Archive binary | ships: upload, list, search, detail, delete, definitions, assignments, review, AUTO suggestions | tesseract-rs `crates/tesseract-paperless-web/src/routes.rs:53-67` |
| Railway image | ships, wraps the binary, volume at `/app/data` for archive + index | this repo `Dockerfile` |
| Optional branding | `BRAND_NAME`, `BRAND_TAGLINE`, `BRAND_ACCENT`, `BRAND_LOGO_URL`; unset = unbranded | tesseract-rs `crates/tesseract-paperless-web/src/branding.rs` (tesseract-rs #110) |
| JSON API | **missing**: every route is an HTML page or a form post | — |
| Health route | `/healthz`, answers `ok` without a session (tesseract-rs #112; not in images built before it) | tesseract-rs `crates/tesseract-paperless-web/src/routes.rs` |
| Authentication | sign-in through `ogar-oidc` / `ogar-auth` against the existing Zitadel when `PAPERLESS_OIDC_ISSUER` is set (allow-listed project roles), from tesseract-rs #112 on; off otherwise, and absent from images built before #112. No separation by customer yet | tesseract-rs `crates/tesseract-paperless-web/src/auth.rs`; setup in [RAILWAY-SERVICE-SETUP.md](RAILWAY-SERVICE-SETUP.md) §2a |
| CI for the image | **missing**: the Dockerfile builds without running any test | this repo `Dockerfile` |
| report-rs client | **missing**, by design until §9 decides | — |

Without sign-in the service gets **no public domain** and stays on Railway's private network. It may get one only after an image that includes tesseract-rs #112 is deployed with the sign-in variables set and an unauthenticated request is confirmed refused (RAILWAY-SERVICE-SETUP.md §2a, "Order"); every signed-in user then sees every document until the customer separation in §6 lands.

## 3. Integration seam with report-rs

Model: RPT-TICKET-API-1 (report-rs `src/ticket_api.rs`). report-rs is a server-side client; the feature is off while its URL variable is unset; calls are made on behalf of the signed-in user.

| Concern | Rule |
|---|---|
| Switch | `REPORT_DOCS_API_URL` in report-rs; unset or blank = feature off, no route answers |
| Transport | HTTPS only, no redirects, 10 s timeout, the same `reqwest` settings as `ReqwestTicketHttp` |
| Identity | forward the user's verified Zitadel ID token as the bearer, as for tickets; paperless decides role and customer from that token (§6) |
| What report-rs stores | the document's content hash (64 hex) and the filename it was shown; never bytes, text or OCR output |
| Intake | `POST /api/v1/documents` (multipart file) → `{hash, duplicate, pages, low_confidence}`; a duplicate is reported, not re-archived (S-2) |
| Lookup | `GET /api/v1/documents/{hash}` → metadata; `GET /api/v1/documents?q=` → ranked hits with snippets |
| Viewing | report-rs links to the paperless page for a hash; it never renders the document itself. The browser must be able to reach that page: under the private-network shape it cannot resolve `paperless.railway.internal`, so the link needs an authenticated route (for example through report-rs) or waits for the public domain after §6 (open decision 5) |
| Failure | paperless down = report-rs shows "documents unavailable"; report-rs never queues or caches documents |

Two ways report-rs could reach the API, decided in §9:

- **Private network** (`http://paperless.railway.internal:<port>`): no public exposure, works before §6. It cannot use `https_only`, so report-rs needs a separate setting for an internal origin.
- **Public HTTPS** after §6: the same shape as the ticket service.

## 4. CI: the report-rs model (RPT-CI-RAILWAY-1)

report-rs runs its whole quality gate inside the Railway image build (`Dockerfile` → `scripts/ci/validate.sh`). GitHub Actions is manual only (`workflow_dispatch`), because hosted runner minutes are capped. A failing check fails the image build, the deployment shows as failed on the PR, and nothing is deployed. GridCraft reaches report-rs through that same gate as a floating git dependency on `main`, with no committed lock file.

paperless-rs copies that model:

| Piece | paperless-rs |
|---|---|
| One command | `scripts/ci/validate.sh`; local runs and the image build call the same script |
| Where it runs | in the `Dockerfile` builder stage, after the clones and before `cargo build --release` |
| Scope | `-p` scoped only, never `--all` (tesseract-rs iron rule 1): `cargo fmt -p tesseract-paperless-web -- --check`; `cargo clippy -p tesseract-paperless --features store,search -- -D warnings`; `cargo clippy -p tesseract-paperless-web --all-targets -- -D warnings`; `cargo test --release -p tesseract-paperless --features store,search`; `cargo test --release -p tesseract-paperless-web` |
| Build settings | `CARGO_PROFILE_{DEV,TEST,RELEASE}_DEBUG=0`, `CARGO_INCREMENTAL=0`; refuse to run when `RUSTFLAGS` is set, as report-rs does |
| Preview vs production | `VALIDATE_MODE` from `RAILWAY_ENVIRONMENT_NAME`: production runs everything and every failure blocks; a PR preview blocks on build, security and the intake round trip, and reports the rest |
| GitHub Actions | one `workflow_dispatch` job that runs the same script; no push or pull_request trigger |
| Dependencies | floating: tesseract-rs, lance-graph, ndarray and OGAR clone their default branch, as today; no lock file; refs pinned only through the existing `*_REF` build args when a deploy must be reproduced |
| Toolchain | read from the cloned tesseract-rs `rust-toolchain.toml` (rustup honours it); the base image tag only needs the right minor |

The test gate needs `--release`: tesseract-rs's recognition tests do not finish in a debug build (tesseract-rs CLAUDE.md, "Debug-vs-release lesson"). That makes the build long; measure the first run before tightening anything.

## 5. Branding

Shipped with this plan: tesseract-rs #110 ("optional branding"). Railway steps: [RAILWAY-SERVICE-SETUP.md](RAILWAY-SERVICE-SETUP.md). paperless-rs sets it on the Railway service, not in the image:

```text
BRAND_NAME=paperless-rs
BRAND_TAGLINE=Document intake and archive for report-rs
BRAND_ACCENT=#0f766e          # example
BRAND_LOGO_URL=               # optional, https:// or a same-site path
```

Unset = the unbranded archive. A custom name adds "powered by tesseract-rs" to the upload page's footer. Invalid values are ignored with one log line.

## 6. Identity and access: POSTPONED

**No authentication or authorization change is made under this plan's first phases.** Zitadel is the identity provider and works in production; any change there needs its own session.

What is recorded for that session:

- Browser sign-in exists from tesseract-rs #112 on (RAILWAY-SERVICE-SETUP.md §2a): authorization code + PKCE, ID token verified by `ogar-oidc`, login established by `ogar-auth`, admission by allow-listed project roles. What remains for this section is the API side (verifying a token report-rs forwards) and the customer separation below.
- The ticket service is the precedent: it verifies the Zitadel ID token that report-rs forwards (`aud`/`azp` checked against an allow-list of client ids), and decides role and customer itself.
- Customer separation in paperless: access is per **customer-to-document association**, not one owner per document. Two customers can upload the same file; the dedup gate then reports the second upload as a duplicate of the same content hash, and both customers must still reach their own upload. So intake records an association (customer, hash) from the token, and every read and search is filtered by the caller's associations. The archive has no such table today (`tesseract_paperless::store` schema); adding it is a schema change.
- report-rs adds no authority of its own when it forwards a token (RPT-TICKET-API-1).

## 7. WordCraft as the editor (later, paperless-rs only)

WordCraft (`AdaWorldAPI/wordcraft`) is a pure-Rust word processor that reads and writes `.docx` and has a browser build (`apps/wordcraft-web`, a static WebAssembly site).

| Decision | Rule |
|---|---|
| Where | inside the paperless service only, as an "edit" action on an archived document |
| Not in report-rs | report-rs never depends on WordCraft, embeds it, or links its crates; it links to the paperless page, as for viewing |
| Versions | an edited document is a new archived document with its own hash; the original is never overwritten (content addressing, S-2) |
| Shape | serve the static WordCraft web build from paperless and open a document by hash; saving uploads through the same intake endpoint |
| Prerequisite | the archive keeps the original uploaded file, retrievable by hash. Today it computes the hash and discards the bytes, so there is no `.docx` for WordCraft to open. That needs a blob store written before the document address (S-5), as its own phase |
| Gate | starts only after §6 is done, because editing is a write |

## 8. Phases and gates

| Phase | Deliverable | Gate (falsifiable) |
|---|---|---|
| **P0** | Railway service from this repo's Dockerfile, private network only, volume at `/app/data`, branding variables set | upload a scan through the private address; the same file again reports a duplicate; the archive survives a redeploy |
| **P1** | `scripts/ci/validate.sh` in the image build, plus the manual workflow | a deliberately failing test fails the Railway build and nothing deploys; the green build deploys |
| **P2** | `/healthz` and the JSON API (§3) in tesseract-paperless-web | `railway.json` health check passes; the API returns the same hash as the HTML upload for the same file; a duplicate returns `duplicate: true` |
| **P3** | identity (§6), in its own session | no token = 401; another customer's hash = 404; a token for another client id is refused |
| **P4** | report-rs client behind `REPORT_DOCS_API_URL` (report-rs session, after its decision) | unset = no route; set = intake from report-rs stores only the hash; paperless down = a clear message, no report-rs failure; a signed-in user follows a document link from report-rs and sees that document, and a user of another customer does not |
| **P4b** | original-file storage: a blob store keyed by hash, written before the document row (S-5) | an uploaded `.docx` can be downloaded byte-identical by its hash; a failed blob write leaves no document row |
| **P5** | WordCraft editor inside paperless (§7) | edit and save makes a new hash; the original still opens unchanged |

## 9. Open decisions

Record each as a Linear issue before acting on it; report-rs decisions belong in `P-EXO-9` as `DECISION: …`.

1. Private-network origin vs public HTTPS for the report-rs client before P3.
2. Whether the JSON API lives in tesseract-paperless-web (recommended: one binary, one source of truth) or in a paperless-rs crate in front of it.
3. Source of the customer in each (customer, hash) association (§6): the caller's verified token at intake
   (recommended, as §6 specifies) or a report-rs field. Either way it records an association, never a single
   owning customer per document.
4. How long the CI gate may take inside a Railway build before tests move to the manual workflow.
5. How a user's browser reaches a paperless document page from report-rs before P3: an authenticated route through report-rs, or no viewing until paperless has its own public, signed-in domain.

## 10. Non-goals

- No change to report-rs in this plan's P0–P3.
- No PostgreSQL in paperless; report-rs's database is not shared.
- No authentication change outside P3's own session.
- No WordCraft in report-rs.
- No second copy of the archive code in this repository: tesseract-rs stays the one source (this repository's `CLAUDE.md`, "DEAD COPY").
