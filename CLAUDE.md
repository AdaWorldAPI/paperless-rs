# CLAUDE.md — paperless-rs

Read first, every session. This repo is young; the conventions below come from
its siblings, which paid for them.

> **⊘ SUPERSEDED (2026-08-24) — this repo's own crates are a DEAD COPY.**
> Per `tesseract-rs/CLAUDE.md` § "tesseract-paperless — document intake lands
> here, feature-gated": the work in `crates/paperless-kv` /
> `paperless-intake` / `paperless-token` continued and shipped as
> `AdaWorldAPI/tesseract-rs`'s `crates/tesseract-paperless` (the gate + seam)
> and `crates/tesseract-paperless-web` (the Railway archive: upload → S-2
> gate → OCR → lancedb → Tantivy search → paperless-ngx-shaped UI, with SPO
> extraction wired in). **Do not sync the two — there is nothing to sync back
> to.** This repo's `Dockerfile` now builds and ships THAT binary (cloning
> `tesseract-rs` + its siblings fresh) rather than this repo's own stub
> crates, so a Railway service still pointed at this repo gets the real
> service on its next deploy. The section below (§ Status) predates the
> supersession; read it as history, not as the current shape of the running
> service.

## What this is

**The assembly point.** `paperless-rs` configures storage, owns the Dockerfile
and the deployment settings, and holds the **ingestion stage** — the stage that
exists in no other repo. It assembles pieces that keep their own concerns:

| repo | owns | never does |
|---|---|---|
| `tesseract-rs` | recognition → `doc.v1` | storage, hashing, persistence |
| `OGAR` | the vocabulary — classids, `ActionDef`s, the hot-plug fuse | I/O |
| `lance-graph` | the type layer, and eventually the KV store | ingestion policy |
| **`paperless-rs`** | **ingestion, the dedup gate, the store wiring, deployment** | recognition, minting concepts |

The direction matters and is easy to get backwards: **settings and the
Dockerfile flow OUT of here into the others**, never the reverse. OGAR is the
connective tissue between recognition and storage. `tesseract-rs` stays
storage-less.

## Where the design came from

`AdaWorldAPI/OGAR` `docs/OGAR-DOC-INGESTION-SPINE.md` — thirteen invariants
(`S-1`..`S-13`) extracted from a decade of `paperless-ngx` production history,
because this stack has months of experience on that axis and it had none to
spare. **Read it before touching the ingestion path.** Two invariants are
load-bearing here and expensive to retrofit:

- **S-2 — dedup runs BEFORE recognition spend.** `OGAR-DOC-W4-BUILD-SPEC`
  makes `persist_document` idempotent on `content_sha256`, which prevents a
  duplicate *subtree* but not a duplicate *spend* — by then a full OCR pass is
  already paid for. The gate belongs on the raw bytes, ahead of
  `OcrExecutor`. That is why it lives in this repo. And its second half is
  the one people drop: match the incoming hash against **both** the original
  and any derived-artifact hash, or a re-ingested export of a held document
  reads as novel.
- **S-5 — bytes before address.** The document root value carries a
  *reference* to the blob, never the bytes. Write the blob first: a failed put
  then leaves collectable garbage, where the other order leaves a resolvable
  `document_guid` pointing at nothing — corruption invisible until read.

## Dependency wiring — git + `[patch]`, never path deps

Siblings are wired by **git with fixed `rev` pins**, so this repo builds
standalone from a bare clone and the Dockerfile needs no sibling-clone step.
Branch refs are forbidden: a floating ref makes a build unreproducible and
silently changes what a measurement measured.

**The trap, and the whole reason the `[patch]` sections exist** (woa-rs hit it
first and documents it in its own `Cargo.toml`): crates *inside* those repos
declare their siblings by escaping relative path —
`ndarray = { path = "../../../ndarray" }`,
`ogar-vocab = { path = "../../../OGAR/crates/ogar-vocab" }`. From a Cargo
**git checkout** those paths point outside the checkout and resolve to
nothing. Each `[patch."<git-url>"]` redirects the escaping names to their own
git source.

Add a name to a patch list the moment it starts escaping. The failure mode is
a confusing "failed to load source", not a clear error.

> **`warning: patch ... was not used in the crate graph` is a policy alert,
> not noise** (lance-graph P0). It means either missing fork wiring or a
> transitive semver mismatch. Diagnose it; never suppress it. In this repo it
> most often means a patch was declared for a crate no member actually pulls
> yet — resolve by removing the patch or adding the consumer, not by ignoring
> the line.

## Hard rules

- **Never mint a classid here.** Concepts are minted in `ogar-vocab`; this
  repo pulls them. Never construct a `*Bridge`, never copy the codebook.
- **Never call `LstmRecognizer` / `structured` / the renderers directly.**
  Recognition goes through `tesseract_ogar::OcrExecutor`, per
  tesseract-rs's `docs/CONSUMER-GUIDE.md`.
- **BBB barrier.** Allowed: `ogar-vocab`, `lance-graph-contract`,
  `tesseract-*`, `ndarray`. **Forbidden: lance-graph engine / planner** — the
  "brain" crates never enter a customer binary.
- **`cargo clippy --all-targets -- -D warnings` and `cargo fmt --check` must
  pass.** `unsafe_code = "forbid"` workspace-wide.
- **Every test must be able to fail.** Before a test lands, answer: *what
  input would make this red?* Then actually break the code and watch it go
  red — the sibling repos have caught vacuous assertions this way repeatedly,
  including ones whose own comment claimed they were falsifiable. A guard
  needs both a can-it-fire and a can-it-stay-silent case, over non-trivial
  input.
- **Fixtures live in the repo.** A test fixture under `/tmp` is a time bomb
  with a skip-guard for a fuse: two `tesseract-core` tests sat red for 13 days
  because their fixtures lived outside the repo, and on a fresh container the
  skip made them pass.

## Toolchain

Pinned `1.97.1` in `rust-toolchain.toml`, matching the workspace-wide sweep
(lance-graph #896). Bare `cargo` in this checkout resolves to it. Never
auto-track `stable`; bump explicitly with the gates green.

## Layout

```
crates/
├── paperless-kv/       the KV layout — S-2 preflight gate, document subtree keys
├── paperless-intake/   raw bytes → gate → tesseract-rs recognition → doc.v1
└── paperless-token/    the tokenization seam — ONE versioned BPE receipt,
                        borrowed by Tantivy, DeepNSM-v2 and a forward surface
docs/
└── TOKEN-SEAM-ARCHITECTURE.md   the bounded architecture + the measured probe
```

## Status — honest

- `paperless-kv`: the S-2 gate and the convergence key are real and tested;
  `DedupIndex` is a **trait**, because there is no KV put/get in lance-graph
  today (`symbiont` is a link probe that prints that the stack linked;
  `surreal_container::open` returns `Err(Blocked)` and every module is a
  `// TODO task NN` header). That is also what `W4-8` prescribes: *"No storage
  backend chosen (KV blob is the consumer's)."*
- `paperless-token`: a **probe**, and it is honest about being one — 37 gates,
  9 disable-runs verified red-then-green, two committed real corpora. It proves
  the seam (one tokenization per span; Tantivy, DeepNSM-v2 and a forward
  surface all borrowed off it; zero changes needed to either consumer crate)
  and names eight gaps that stand between it and a production carrier. The
  resident lane is still a probe-local `Vec<[u8;12]>`, not a lawful
  `SoaEnvelope` lane. Read `docs/TOKEN-SEAM-ARCHITECTURE.md` before extending
  it; §7 is the list of what is NOT settled.
- `paperless-intake`: **a stub.** It exists so the `[patch]` chain is actually
  exercised by the crate graph rather than declared and unused. Recognition is
  not wired yet.
- No storage backend is implemented. When one lands, `holograph::storage` is
  the working template — `FixedSizeBinary` + `ArrowStore::{get,get_bytes,save,load}` —
  and `canonical_node.rs:1586-1596` names `FixedSizeBinary(512)`, 64-byte
  aligned and uncompressed, as the **only** column type satisfying the
  zero-copy `node_rows_from_le_bytes` contract. A variable-length `Binary`
  column silently disqualifies.
