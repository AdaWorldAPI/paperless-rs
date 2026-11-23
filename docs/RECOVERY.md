# Recovery: an S3 copy of the archive (slice C)

Slice C of `PAPERLESS-NGX-PARITY.md` §8. This document fixes the commit order,
the recovery protocol and the failure modes **before** the code, as §6 requires.
It describes the tesseract-rs `tesseract-paperless-web` service.

## 1. What is irreplaceable

| Data | Where today | Replaceable? |
|---|---|---|
| Original uploads | `/app/data/blobs` (slice B) | **No.** |
| Document rows: filename, upload time, OCR result, SPO facts | LanceDB `/app/data/archive.lance` | Filename and upload time **no**; the OCR result could be recomputed from the original, but only at the cost of a full recognition pass |
| Metadata: correspondents, document types, tags, their rules, assignments, review flags | LanceDB, same dataset | **No.** It is the users' work |
| Search index | `/app/data/search_index` | Yes: rebuilt from the rows on every start (`reconcile`) |
| AUTO model | memory | Yes: re-mined after start |

So the S3 copy holds originals, document records and a metadata snapshot. It
does not hold the search index or the LanceDB files.

## 2. Configuration

S3 is off unless configured. The variables are the service's own and are read
explicitly. There is **no** fallback to `AWS_*` or to any SDK credential chain,
so report-rs's buckets and keys can never be picked up by accident.

| Variable | Meaning |
|---|---|
| `PAPERLESS_S3_ENDPOINT` | `https://…` endpoint, path-style (Tigris: `https://fly.storage.tigris.dev`) |
| `PAPERLESS_S3_BUCKET` | bucket name |
| `PAPERLESS_S3_ACCESS_KEY_ID` | access key (sealed) |
| `PAPERLESS_S3_SECRET_ACCESS_KEY` | secret (sealed) |
| `PAPERLESS_S3_REGION` | optional, default `auto` |
| `PAPERLESS_S3_PREFIX` | optional, default `paperless` |

Either the four required variables are all set or none is. A partial set
stops the service at start, so a missing secret can never silently turn the
copy off. An endpoint that is not `https://` is refused.

## 3. Layout in the bucket

Everything sits under `<prefix>/`:

| Key | Content | Written |
|---|---|---|
| `originals/<sha256>` | the exact uploaded bytes | once, never changed |
| `documents/<sha256>.json` | the document record (§3.1) | once; rewritten once if an original is attached later (§4) |
| `metadata/current.json` | the metadata snapshot (§3.2) | replaced after every metadata change |

The key of an original is its SHA-256, so a restored original is checked
against its own name.

### 3.1 Document record

```json
{
  "schema": "paperless-rs/document-record/v1",
  "content_sha256_hex": "…",
  "filename": "scan.png",
  "ingested_at_unix_ms": 1700000000000,
  "mean_confidence": 92,
  "low_confidence": false,
  "doc_ir_json": "…",
  "spo_json": null,
  "has_original": true
}
```

`doc_ir_json` is the stored `DocIr`, so a restore recomputes nothing.
`has_original` is false for a document stored before slice B. Such a document
is restored as it is today: a row without an original.

### 3.2 Metadata snapshot

```json
{
  "schema": "paperless-rs/metadata-snapshot/v1",
  "taken_at_unix_ms": 1700000000000,
  "definitions": [ … every definition, retired ones included, with its id … ],
  "assignments": [ … ],
  "reviews": [ { "content_sha256_hex": "…", "reviewed_at_unix_ms": … } ]
}
```

Definition ids are kept, so a restored assignment points at the same
definition.

## 4. Commit order

Local first, then S3. The local volume stays the primary copy; S3 is the copy
that survives losing the volume.

**Upload of a new document**

1. Local original (slice B: temporary file, sync, rename, sync).
2. Local row.
3. S3 `originals/<sha256>`.
4. S3 `documents/<sha256>.json`.

A record is written only after its original, so a record in S3 always has its
original in S3. The upload answers once step 2 succeeds. Steps 3 and 4 are
attempted immediately, and a failure there is logged and retried by the
replicator (§5). A failed S3 write never fails the upload or removes the local
document.

**Original attached to an older document** (a re-upload of a document stored
before slice B backfills its original locally, `PAPERLESS-NGX-PARITY.md` §6):

1. Local original.
2. S3 `originals/<sha256>`.
3. S3 `documents/<sha256>.json`, rewritten with `has_original: true`.

The record is rewritten only after the original is in S3, so a record that
says `has_original: true` always has its original. If step 2 or 3 fails, the
replicator (§5) repeats both.

**Metadata change** (assign, unassign, review, definition create, rename,
rule, retire): after the local write, the snapshot is rebuilt from the current
local state and uploaded. Snapshot uploads are serialized, and each one reads
the state at upload time, so an older snapshot can never overwrite a newer
one.

**Delete**

1. Local row, search entry, local original (slice B order).
2. S3 `documents/<sha256>.json`.
3. S3 `originals/<sha256>`.
4. Metadata snapshot.

The record goes before the original, so a failure leaves an unused original
in S3, never a record whose original is missing.

**Copy and delete of the same document never interleave.** A copy (the one
after an upload and every replicator copy) takes the same per-document lock a
delete holds, and reads the local row under it. A copy that waited for a delete
finds the row gone and writes nothing, so a deleted document cannot reappear in
S3.

## 5. Replication

At start (after any restore) and every 15 minutes, the replicator:

- lists `documents/` and `originals/`, and copies every local document that
  has no record, or has a local original that S3 does not hold (an original
  attached after the document was copied): its original (if any) first, then
  its record;
- uploads the metadata snapshot.

The replicator **never deletes** from S3. Only an explicit delete removes S3
objects. If that S3 delete fails, the record stays in S3 and a later restore
brings the deleted document back. That is logged, and it is the deliberate
trade: an automatic cleanup that treats "missing locally" as "deleted" would
erase the S3 copy exactly when the local copy is damaged.

The dashboard shows how many documents are not yet fully in S3 (no record, or
an original missing), so a stuck copy is visible.

## 6. Restore

**When.** At start, if S3 is configured and the local archive holds no
documents, or a restore marker (`<BLOB_DIR>/.restore-in-progress`) is present.

**Steps.**

1. Write the restore marker.
2. List `documents/` (all pages).
3. For each record not already local: read the record and check its schema and
   hash; if `has_original`, or if `originals/<sha256>` is listed (a record
   whose rewrite after a backfill failed still says false), read it, check that its SHA-256
   equals the key, and write it locally (slice B store); then write the row from
   the record.
4. Read `metadata/current.json` and import definitions, assignments and reviews,
   keeping ids. Assignments and reviews whose document was not restored are
   dropped.
5. Remove the marker.
6. The existing start-up `reconcile` rebuilds the search index from the
   restored rows.

Each step is idempotent, so a restore interrupted by a crash continues on the
next start: the marker is still there, and documents already restored are
skipped.

**A record that fails its check** (unreadable, wrong schema or hash) is
skipped and logged. An original that is missing or fails its hash check is left
out, and its document is restored without it. The restore continues, and the marker
is removed at the end, so one damaged object cannot block the archive from
starting.

## 7. Failure modes

| Situation | Behaviour |
|---|---|
| S3 not configured | Today's behaviour; no copy. Logged once at start |
| Partial S3 configuration | Refuse to start |
| S3 unreachable at start, local archive present | Start. Replication retries; the dashboard shows the backlog |
| S3 unreachable at start, local archive empty | **Refuse to start.** Starting empty would accept new uploads into an archive that does not know its own history. Railway restarts the service, and it starts once S3 answers |
| S3 reachable and empty, local archive empty | A new archive; start normally |
| S3 fails during an upload | The upload succeeds locally; the replicator copies it later |
| S3 fails during a delete | The local delete stands; the S3 objects remain and are listed in the log (§5) |
| Restore interrupted | The marker stays; the next start continues |
| Search index unreadable (corrupt files) | Wiped and rebuilt from the rows at start, as a schema mismatch already is |
| Original in S3 does not match its hash | That document is restored without an original, and logged |
| Metadata snapshot unreadable | The restore stops and the start fails, marker kept. Starting without the users' metadata would upload an empty snapshot over the damaged one |

## 8. Acceptance tests (slice C)

From `PAPERLESS-NGX-PARITY.md` §9, made concrete:

- Restart and redeploy keep originals, metadata and search (volume present: no
  S3 traffic beyond the replicator's listing).
- An empty volume restores from S3: every document, its original
  (byte-identical), its filename and upload time, every definition with its id,
  every assignment and review flag. Search finds the restored documents.
- S3 unreachable with an empty volume: the service refuses to start. With a
  volume present it starts and reports the backlog.
- A partial S3 configuration refuses to start.
- A corrupt search index is rebuilt and finds the same documents.
- An interrupted restore completes on the next start.
- A record whose original fails its hash check restores without an original.
- The replicator never deletes from S3.
- An older document copied without an original, then given its original by a
  re-upload whose immediate S3 copy fails: the replicator uploads the original,
  then rewrites the record with `has_original: true`, and a restore brings the
  original back.

All of these run against an in-memory object store in the test suite. The
request signing is tested against fixed vectors, and once against a real
S3-compatible server where one is available.

## 9. Not in this slice

- Customer separation (slice T). The S3 layout keys documents by hash only;
  slice T adds its association table to the snapshot.
- Encryption at rest beyond what the bucket provides.
- Versioned history of metadata: `metadata/current.json` is replaced, not kept.
  Bucket versioning, if enabled, keeps older snapshots.
