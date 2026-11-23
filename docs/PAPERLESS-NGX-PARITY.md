# paperless-ngx parity — what the archive has, what it lacks, and where each gap lands

> **Scope.** The person-facing surface of the running archive,
> `tesseract-rs/crates/tesseract-paperless-web`, measured against
> paperless-ngx. Companion to `USABILITY.md`: that document ranks gaps by what
> they buy a person; this one inventories the surface so the ranking has
> something concrete to rank.
>
> **Measured against** tesseract-rs `master` at `7cbc969` (2026-10-26), and
> re-measured at `5be3d17` (2026-11-23, after #112: OIDC sign-in). Rows that
> changed are marked "(5be3d17)". That
> is what this repo's `Dockerfile` builds: it clones each sibling's default
> branch when `TESSERACT_RS_REF` is unset, so **a deployment built before the
> metadata commits (`3a200c9`, `068ddd7`) has none of the metadata features
> below** (definitions, assignments, review, AUTO suggestions); upload, search,
> detail and confirmed delete predate those commits. A live service whose index
> page has no link to `/definitions` is such a build. Rebuild it from a
> `TESSERACT_RS_REF` that contains both commits; if the ref is unchanged,
> rebuild without the cached clone layer, because a plain redeploy can reuse it.
>
> **Sources.** The tesseract-rs side is read from source, each row cited. The
> paperless-ngx side is stated from knowledge of that project's UI and was
> **not re-verified against its source for this document** — treat the
> left column as a checklist to confirm, not as a specification.

## §0 Two corrections to `USABILITY.md`

Both change what is cheap and what is not, so they come first.

**The raw bytes are not stored.** `USABILITY.md` §4 says the archive "stores
the raw bytes (content-addressed, by design) and never renders them", and §5
item 1 builds on that: showing the page image is "a field the ClassView can
project". On master the archive schema
(`crates/tesseract-paperless/src/store.rs:109-123`) is hash, GUID, filename,
mime, source, page count, mean confidence, low-confidence flag, the `DocIr`
JSON, ingest time, and the SPO JSON. There is no blob column, and nothing on
the ingest path persists the source image. The content hash is computed from
the bytes; the bytes themselves are discarded.

So the top-ranked usability gap is **storage work before it is render work**.
It needs a blob store, and that store is governed by S-5 (bytes before
address): write the blob first, so a failed put leaves collectable garbage
rather than a resolvable document pointing at nothing. Until that lands,
neither render path can show a page, because there is no page to show.

**a2ui-rs is no longer seed-stage.** `USABILITY.md` §6 describes `a2ui-core`
as a re-export plus a drift fuse, with the server and the wasm client
*planned*. a2ui-rs's own `CLAUDE.md` now lists `a2ui-server` (RBAC projection,
`NodeDelta` down, `ActionInvoke` up, the sealed session), `a2ui-wasm` (the
fieldview client, `resolve_nested` drill-down) and `a2ui-paint` (layout,
hit-testing, skins) as shipped. That was read from a2ui-rs's documentation and
not re-run here. If it holds, the reason §6 gave for not building on a2ui-rs
yet no longer applies.

## §1 The inventory

The status column uses three values: **present** (master has it), **partial**
(master has a narrower form), **absent**. The last column says what the gap is
in render-path terms, per `USABILITY.md`'s vocabulary: a projected **field**,
an **action** by `ActionDef` address, or **store** work that has to exist
before either.

### Intake

| paperless-ngx | master | status | lands as |
|---|---|---|---|
| Upload in the browser | file or URL, one at a time (`routes.rs` `/upload`; `index.html`) | partial | action |
| Duplicate detection | S-2 gate on the raw-byte hash, before OCR spend (`ingest.rs`) | present | — |
| Multi-file and drag-and-drop upload | single file per request | absent | action |
| Consumption folder, mail fetch | none | absent | intake, not UI |
| Task list (queued, processing, failed) | ingest is synchronous; the outcome is a log line (`log_ingest_outcome`) | absent | field |
| PDF input | raster images only (`decode.rs`) | absent | intake |

### Finding documents

| paperless-ngx | master | status | lands as |
|---|---|---|---|
| Full-text search with highlights | Tantivy BM25, per-language stemmed fields, snippet highlighting (`search.rs`; `documents()` in `routes.rs`) | present | — |
| Filter by tag, correspondent, type | none — the list is search or the most recent `LIST_LIMIT` rows (`documents()`) | absent | field |
| Filter by date range | none | absent | field |
| Sort by column | none | absent | field |
| Saved views | none | absent | store + field |
| Pagination | hard `LIST_LIMIT` cut-off | absent | field |
| Bulk select and bulk edit | none | absent | action |

The filter rows are the largest gap a paperless-ngx user would notice first.
The data to filter on now exists (`archive_meta` assignments); the list does
not read it.

### One document

| paperless-ngx | master | status | lands as |
|---|---|---|---|
| Page preview beside the details | none, and no stored image to preview (§0) | absent | **store**, then field |
| Download original | none — the original is not kept (re-checked at 5be3d17: `ingest.rs` hashes the bytes, OCRs them and drops them) | absent | store, then action |
| Editable title and created date | filename and ingest time only, read-only | absent | field + action |
| Correspondent, type, tags | assign and unassign per kind (`/documents/:hash/assign`, `/unassign`) | present | — |
| Suggested metadata | per-kind suggestions with expected percent, evidence and cues; accept (`/documents/:hash/accept`) | present | — |
| Inbox / to-do state | a reviewed flag (`/documents/:hash/review`) | partial | — |
| Content (recognized text) | plain text, plus harvested fields, regions and extracted SPO facts | present | — |
| Notes | none | absent | store + action |
| History / audit trail | none | absent | store + field |
| Permissions and share links | none | absent | action |
| Delete | a POST route with a JS `confirm()` | present | action |
| Confidence | the page **mean**, with an orange "(low)" | partial | field |

The last row is `USABILITY.md` §5 item 2 and is unchanged by this inventory:
the aggregate should follow the cell-level rule (minimum, not mean) and become
a status the list can filter on.

### Managing the vocabulary

| paperless-ngx | master | status | lands as |
|---|---|---|---|
| Tags, correspondents, document types | one `/definitions` page over `MetaKind::{Correspondent, DocumentType, Tag}` (`archive_meta.rs:107-114`): create, rename, retire | present | — |
| Matching rules per definition | S-8 rules, the paperless-ngx algorithms (`/definitions/rule`; `matching.rs`) | present | — |
| Learned (auto) matching | association-rule mining, triggered by hand (`/auto/mine`; `auto_match.rs`) | present | — |
| Storage paths | none | absent | store |
| Custom fields | none; harvested key/value fields are recognized, not user-defined | absent | store + field |
| Document counts per definition | none | absent | field |

### Around the edges

| paperless-ngx | master | status | lands as |
|---|---|---|---|
| Dashboard (statistics, saved-view widgets) | none; `/` is the upload form | absent | field |
| Navigation shell (sidebar) | a back link per page | absent | field |
| Workflows (triggers and actions) | none | absent | action |
| Users, groups, permissions | (5be3d17) OIDC sign-in against Zitadel with role admission, sealed session cookie, same-origin check on writes, bearer tokens for delegating services (`auth.rs`). No per-object permissions, no groups, no customer separation: every admitted user sees and edits every document. Sign-in is off when `PAPERLESS_OIDC_ISSUER` is unset | partial | action |
| Export and import | none | absent | action |

## §2 What the inventory says

**The data model is ahead of the screens.** Master already stores the three
paperless-ngx metadata kinds, their matching rules, learned suggestions and a
review state. What is missing is mostly *reading* that data where a person
looks: the list cannot filter on it, and nothing counts it. Those are field
projections over data that exists — the cheapest kind of gap.

**Two gaps are not projections at all.** The page preview and the original
download both need the source bytes, and the archive does not keep them. That
is store work under S-5, and it is the prerequisite for `USABILITY.md`'s
top-ranked item. It is also path-neutral: a blob store is needed whether the
page is later rendered by an Askama template or projected through a2ui-rs, so
it can be built now without settling the render-path question.

**Every action row is a render-path decision.** Master's actions are
form-POST routes (assign, unassign, accept, review, the definition verbs,
delete). `USABILITY.md` asks that new actions travel by `ActionDef` address
instead. Each further action added as a route is one more to convert later,
so the action rows are the ones to hold until that decision is made.

## §3 What this does not settle

- **Which render path.** This document places each gap; it does not choose
  between extending the templates and moving to a2ui-rs.
- **The paperless-ngx column is unverified** against that project's source,
  as stated above.
- **No user research.** Like `USABILITY.md`, this is an inventory and an
  inference, not a measurement of what people need.

## §4 Reusable code (5be3d17)

Build on these; do not write a second one.

| Concern | Existing implementation | Use for |
|---|---|---|
| Filing metadata | `tesseract-paperless::archive_meta::MetaStore`: definitions per `MetaKind` (correspondent, document type, tag), assignments with `Source` (manual, rule, auto), review flag, `reconcile`, one write mutex | Inbox (`reviewed_hashes`), list filters (`all_assignments`), batch assign (`assign` per document), counts per definition |
| Document rows | `tesseract-paperless::store::LanceStore` (`put` is a whole-row `merge_insert`, `list`, `get`, `hashes`, `delete`, `count`) | List, detail, delete; never metadata columns (see `archive_meta` module doc) |
| Search | `tesseract-paperless::search` (Tantivy, BM25, stemmed per language, snippets) | Search box; result set intersected with filters |
| Matching | `matching.rs` (paperless-ngx algorithms), `auto_match.rs` (learned) | Suggestions on detail, inbox triage |
| Dedup gate | `intake.rs` S-2 preflight on the raw-byte SHA-256, before OCR | Every intake path, including a future blob store |
| Sign-in | `tesseract-paperless-web::auth` (`Login`, `require_session`, `same_origin_request`, `verify_bearer`) | Every route and every A2UI frame/action |
| Branding | `tesseract-paperless-web::branding` (`BRAND_*`) | The shared shell |
| Object store client | `report-rs::objstore` (`S3Config::from_suffixed_env`, `ObjectStore` trait, SigV4, stable error classes) | Phase 4. Reuse the concept or extract it into a shared crate; report-rs itself stays read-only, and its buckets and credentials are not used |
| Boot hydration | `report-rs::mapping_hydrate` (fetch, verify, publish to a local file) | Phase 4 recovery shape |
| UI projection | a2ui-rs `a2ui-server` (`project_node`, `project_surface` with `WideFieldMask` RBAC, `resolve_action`, `DesktopSession` sealed transport), `ogar-render-askama::render_field_view` | Phase 2 |

## §5 Navigation and workflow targets

Target navigation (paperless-ngx order). "Today" is what a link would open
at 5be3d17.

| Entry | Today | First real form |
|---|---|---|
| Dashboard | `/` is the upload form | counts (documents, unreviewed, per kind) + upload, from `count` and `reviewed_hashes` |
| Inbox / Unreviewed | none | the document list filtered to hashes not in `reviewed_hashes` |
| Documents | `/documents` (search or latest 200) | same, plus filters by definition |
| Correspondents, Tags, Document Types | one `/definitions` page with three sections | the same page, one section per entry (`?kind=`), with document counts |
| Storage Paths | absent | absent: needs a `MetaKind` or its own table, and a blob store to apply to |
| Settings | absent | absent: nothing is user-configurable at runtime today |

A nav entry is shown only when it opens something that works. Storage Paths
and Settings stay out of the nav until they exist.

| Workflow | Today | Missing step |
|---|---|---|
| Dashboard → Inbox → Document → Review → next document | review exists on the detail page | inbox list; "review and next" |
| Documents → Filter → Detail → Edit metadata → back to results | search, detail, assign | filters; the return keeps the query |
| Upload → OCR → Document detail | works (redirect to detail) | — |
| Definitions → Create/Edit → Assign → Document | works | counts per definition |
| Multi-select → batch metadata | absent | checkboxes + one batch route over `MetaStore::assign` |

## §6 Persistence changes required

- **Originals (Phase 3).** A content-addressed blob store keyed by the
  SHA-256 the S-2 gate already computes. Write the blob (temp file, fsync,
  rename) before the document row (S-5); verify the hash on read; a duplicate
  upload reuses the blob. Filename and MIME stay in the document row. A
  protected download route serves the exact bytes.
- **Recovery (Phase 4).** S3 (Tigris) as the durable copy of originals and of
  recoverable metadata records; `/app/data` (volume `paperless-archive`, 50 GB)
  keeps LanceDB, Tantivy and a local blob cache. The commit order, recovery
  protocol and failure modes are specified before code.
- **Formats.** `decode.rs` accepts raster images only. Keeping originals does
  not make PDF or DOCX ingestible; that is separate intake work.

## §7 Askama and A2UI integration points

- **One shell.** A `base.html` that every page extends: navigation, branding,
  sign-out, one stylesheet. Today each of the four templates carries its own
  copy.
- **One view model per screen.** The handler builds a typed view (fields and
  actions) once; Askama renders it to HTML, A2UI projects the same view to
  `NodeDelta` frames. Authorization runs before either.
- **Dependency seam.** `a2ui-server` depends on OGAR by git (`branch = main`);
  `tesseract-paperless-web` uses OGAR by path. Adding `a2ui-server` needs a
  `[patch]` that points the git OGAR crates at the same path checkout, or the
  two sides compile two incompatible copies of every OGAR type.
- **Actions.** Today's actions are form POSTs. A2UI actions resolve by
  `ActionDef` ordinal on the server and go through the same authorization and
  same-origin rules as the POST routes.

## §8 Slices, in order

Durability and authentication come before more UI.

| Slice | Content | Repo |
|---|---|---|
| A1 | Enforced auth-required mode (`PAPERLESS_REQUIRE_AUTH`): refuse to start without sign-in | tesseract-rs |
| A2 | Shared Askama shell; Dashboard, Inbox, Documents, per-kind definition entries; existing routes unchanged | tesseract-rs |
| B | Original-file store, download route, S-5 order, failure injection | tesseract-rs |
| C | S3 durable copy, boot verification and hydration, index rebuild | tesseract-rs (+ shared store crate) |
| D | A2UI vertical slice on the document list or detail, same view model and authorization as Askama | tesseract-rs + a2ui-rs |
| E | Filters, batch metadata, review-and-next, WordCraft readiness (DOCX contract first) | tesseract-rs |

## §9 Acceptance tests per slice

- **A1:** with `PAPERLESS_REQUIRE_AUTH=1` and no issuer, startup fails; with
  an issuer it starts; without the flag, today's behaviour is unchanged.
- **A2:** every nav entry returns 200 for a signed-in user and 303/401 without
  a session; every pre-existing route still answers as before; Inbox lists
  exactly the documents not marked reviewed, and marking one reviewed removes it.
- **B:** upload then download returns byte-identical content; a tampered blob
  is refused on read; a duplicate upload stores no second blob and runs no OCR;
  a crash between blob and row leaves an orphan blob, never a row without a blob;
  an unauthenticated download is refused.
- **C:** restart and redeploy keep originals, metadata and search; an empty
  volume rehydrates from S3; S3 down gives the specified degraded or fail-closed
  answer; a corrupted search index rebuilds deterministically; a request for
  another tenant's document neither returns it nor reveals that it exists.
- **D:** the Askama page and the A2UI frame for one document carry the same
  fields; a field outside the role mask is absent from the frame; an action
  without authorization is refused on the server.
- **E:** filters return exactly the documents with the assignment; a batch
  assign changes every selected document and nothing else.
