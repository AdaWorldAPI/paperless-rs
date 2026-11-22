# Railway service setup: paperless (tesseract-rs archive), and WordCraft later

Steps to create the paperless service described in
[REPORT-RS-INTEGRATION-PLAN.md](REPORT-RS-INTEGRATION-PLAN.md) (phase P0), plus
the later WordCraft phase (P5). Nothing here changes report-rs, Zitadel or any
existing service.

## Before you start

- tesseract-rs PR #110 (branding) should be merged. The image clones tesseract-rs's
  default branch at build time, so branding variables only take effect once #110
  is on `master`.
- The archive has **no sign-in**. Anyone who can reach it can read and delete
  documents. The service therefore gets **no public domain** until phase P3
  (identity) is done in its own session.

## 1. Create the service (P0)

1. Add the service to the Railway project that already holds report-rs, so it
   shares the private network. Source: GitHub repository `AdaWorldAPI/paperless-rs`,
   branch `main`. The `Dockerfile` at the repository root is used by default.
2. Name the service `paperless`. Its private address is then
   `paperless.railway.internal`.
3. Add a volume mounted at `/app/data`. Without it the archive and the search
   index are lost on every redeploy.
4. Service variables:

   | Variable | Value | Notes |
   |---|---|---|
   | `PORT` | `8080` | a fixed port, so other services can reach `paperless.railway.internal:8080` |
   | `BRAND_NAME` | `paperless-rs` | optional; unset = unbranded |
   | `BRAND_TAGLINE` | `Document intake and archive for report-rs` | optional |
   | `BRAND_ACCENT` | for example `#0f766e` | optional, `#rgb` or `#rrggbb` |
   | `BRAND_LOGO_URL` | an `https://` URL or `/path` | optional |

   No secrets are needed: the build clones public repositories only. The
   Dockerfile also accepts optional build arguments `TESSERACT_RS_REF`,
   `LANCE_GRAPH_REF`, `NDARRAY_REF` and `OGAR_REF` to reproduce a specific
   build; leave them unset normally.
5. Settings → Deploy → healthcheck path `/` (the upload page; there is no
   `/healthz` until P2), and a healthcheck timeout of a few minutes: the first
   start loads the OCR model and opens the archive.
6. Settings → Networking: **do not generate a public domain.**

The first build compiles lancedb, lance and the OCR stack from source and takes
a long time. A build failing on `protoc` means the builder stage lost
`protobuf-compiler`/`libprotobuf-dev`; both are in the Dockerfile.

## 2. Check P0

| Check | Expected |
|---|---|
| deploy logs | start lines from `tesseract-paperless-web`, no `BRAND_* ignored` line (unless a value was invalid) |
| healthcheck | passes on `/` |
| volume | redeploy the service; the documents uploaded before are still listed |
| from another service in the project | `curl http://paperless.railway.internal:8080/` returns the upload page with the configured name |

Two caveats:

- The upload round trip (upload a scan, upload it again, see "duplicate") needs
  a way to reach the service. Without a public domain that means a shell in a
  service that has `curl` (the paperless image has none), or a short-lived public
  domain with test documents only, removed right after. That choice is the
  operator's.
- The binary binds `0.0.0.0` (IPv4). If `paperless.railway.internal` does not
  answer from another service while the healthcheck passes, the project's private
  network may be IPv6-only; the fix is to bind `[::]` in tesseract-paperless-web.
  This has not been tested on Railway.

## 3. CI (P1)

Not yet in place. Follow §4 of the plan: a `scripts/ci/validate.sh` run inside
the Dockerfile, as report-rs does (RPT-CI-RAILWAY-1). Once it lands, a failing
check fails the Railway build and nothing is deployed. Railway PR environments
then run the same gate for paperless-rs pull requests.

## 4. WordCraft as the editor (P5, later)

Do **not** deploy WordCraft yet. It starts after P3 (identity), because editing
is a write. It never goes into report-rs.

When P5 starts:

1. Build the WordCraft web app from source: `AdaWorldAPI/wordcraft`,
   `apps/wordcraft-web`, `trunk build --release` (it is a static
   WebAssembly site; the fork publishes no release archive, so there is no
   prebuilt zip to download).
2. Serve the built static files from the paperless service under one path (for
   example `/editor/`); this needs a static-file route in tesseract-paperless-web
   and a copy step in this repository's Dockerfile.
3. Add "Edit" on a document page: it opens the editor with that document
   (`.docx`) loaded by its hash.
4. Saving uploads the edited file through the normal intake. It becomes a new
   document with its own hash; the original is never overwritten.
5. Gate: edit and save produces a new hash, and the original still opens
   unchanged.
