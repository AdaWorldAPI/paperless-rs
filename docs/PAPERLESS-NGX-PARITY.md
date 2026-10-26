# paperless-ngx parity — what the archive has, what it lacks, and where each gap lands

> **Scope.** The person-facing surface of the running archive,
> `tesseract-rs/crates/tesseract-paperless-web`, measured against
> paperless-ngx. Companion to `USABILITY.md`: that document ranks gaps by what
> they buy a person; this one inventories the surface so the ranking has
> something concrete to rank.
>
> **Measured against** tesseract-rs `master` at `7cbc969` (2026-10-26). That
> is what this repo's `Dockerfile` builds: it clones each sibling's default
> branch when `TESSERACT_RS_REF` is unset, so **a deployment built before the
> metadata commits (`3a200c9`, `068ddd7`) shows none of the rows marked
> *present* below.** A live service whose index page has no link to
> `/definitions` is such a build; redeploying is enough to pick them up.
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
| Download original | none — the original is not kept | absent | store, then action |
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
| Users, groups, permissions | none — `/` and every route are unauthenticated | absent | action |
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
