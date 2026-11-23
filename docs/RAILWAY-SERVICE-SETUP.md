# Railway service setup: paperless (tesseract-rs archive), and WordCraft later

Steps to create the paperless service described in
[REPORT-RS-INTEGRATION-PLAN.md](REPORT-RS-INTEGRATION-PLAN.md) (phase P0), plus
the later WordCraft phase (P5). Nothing here changes report-rs, Zitadel or any
existing service.

## Before you start

- tesseract-rs PR #110 (branding) should be merged. The image clones tesseract-rs's
  default branch at build time, so branding variables only take effect once #110
  is on `master`.
- Without the `PAPERLESS_OIDC_*` variables the archive has **no sign-in**:
  anyone who can reach it can read and delete documents, and the log says so at
  start. Generate a public domain only after sign-in is configured and checked
  (§2a). Sign-in needs tesseract-rs with the `auth` module (tesseract-rs #112) on
  its default branch.

## 1. Create the service (P0)

1. Add the service to the Railway project that already holds report-rs, so it
   shares the private network. Source: GitHub repository `AdaWorldAPI/paperless-rs`,
   branch `main`. The `Dockerfile` at the repository root is used by default.
2. Name the service `paperless`. Its private address is then
   `paperless.railway.internal`.
3. Add a volume mounted at `/app/data`. Without it the archive and the search
   index are lost on every redeploy. **The image runs as a non-root user
   (`appuser`, uid 10001), and a mounted volume does not keep the image's
   `chown`**, so the process may be unable to write there. Set
   `RAILWAY_RUN_UID=0` (Railway's documented setting for this case); the
   process then runs as root. The P0 check below proves writes work.
4. Service variables:

   | Variable | Value | Notes |
   |---|---|---|
   | `PORT` | `8080` | a fixed port, so other services can reach `paperless.railway.internal:8080` |
   | `RAILWAY_RUN_UID` | `0` | lets the process write the mounted volume (it then runs as root) |
   | `BRAND_NAME` | `paperless-rs` | optional; unset = unbranded |
   | `BRAND_TAGLINE` | `Document intake and archive for report-rs` | optional |
   | `BRAND_ACCENT` | for example `#0f766e` | optional, `#rgb` or `#rrggbb` |
   | `BRAND_LOGO_URL` | an `https://` URL or `/path` | optional |

   No secrets are needed: the build clones public repositories only. The
   Dockerfile also accepts optional build arguments `TESSERACT_RS_REF`,
   `LANCE_GRAPH_REF`, `NDARRAY_REF` and `OGAR_REF` to reproduce a specific
   build; leave them unset normally.
5. Settings → Deploy → healthcheck path `/healthz` (answers `ok` without a
   session), and a healthcheck timeout of a few minutes: the first
   start loads the OCR model and opens the archive.
6. Settings → Networking: **do not generate a public domain** until §2a is
   done.

The first build compiles lancedb, lance and the OCR stack from source and takes
a long time. A build failing on `protoc` means the builder stage lost
`protobuf-compiler`/`libprotobuf-dev`; both are in the Dockerfile.

## 2. Check P0

| Check | Expected |
|---|---|
| deploy logs | start lines from `tesseract-paperless-web`, no `BRAND_* ignored` line (unless a value was invalid) |
| healthcheck | passes on `/healthz` |
| volume | the log shows no permission error for `/app/data`; after an upload and a redeploy the document is still listed |
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

## 2a. Sign-in with the existing Zitadel (`login.msgraph.de`)

The archive signs users in with OIDC (authorization code + PKCE) through
`ogar-oidc` / `ogar-auth`, the same way report-rs does. It is off until
`PAPERLESS_OIDC_ISSUER` is set; once set, every page except `/auth/*` and
`/healthz` needs a session, and a set but invalid configuration stops the
service at start instead of running it open.

**Zitadel (a new application; existing applications are not changed):**

1. In the project that should hold the paperless roles (a new project
   `paperless` keeps them apart from report-rs's roles), create the role key
   `paperless.user` (and `paperless.admin` if wanted).
2. In that project, create a new **Web** application, authentication method
   **PKCE** (no client secret). Redirect URI
   `https://<paperless domain>/auth/callback`. No post-logout URI is needed:
   "Sign out" ends the archive session only, and the Zitadel session stays, so
   the next sign-in may go through without asking for a password.
3. In the application's token settings, enable **User roles inside ID Token**.
   Without it the token carries no roles and every sign-in is refused as
   "access not set up".
4. Grant `paperless.user` to the people who may use the archive.

**Railway variables on the paperless service:**

| Variable | Value | Required |
|---|---|---|
| `PAPERLESS_OIDC_ISSUER` | `https://login.msgraph.de` | yes (turns sign-in on) |
| `PAPERLESS_OIDC_CLIENT_ID` | the client id of the new application | yes |
| `PAPERLESS_OIDC_REDIRECT_URL` | `https://<paperless domain>/auth/callback` (must match Zitadel exactly) | yes |
| `PAPERLESS_OIDC_ROLES` | `paperless.user` (comma-separated role keys that admit a user; exact match) | yes |
| `PAPERLESS_SESSION_KEY` | 32 random bytes, `openssl rand -hex 32` (sealed variable) | recommended; without it every redeploy signs everyone out |
| `PAPERLESS_OIDC_ORGS` | Zitadel organization ids; a role then counts only when granted under one of them | optional |
| `PAPERLESS_OIDC_CLIENT_SECRET` | only for a confidential (non-PKCE) application (sealed) | optional |
| `PAPERLESS_OIDC_ROLES_CLAIM` / `PAPERLESS_OIDC_EXTRA_SCOPES` | defaults `urn:zitadel:iam:org:project:roles` / `urn:zitadel:iam:org:projects:roles` | optional |

**Order:** create the Zitadel application with the domain you will use, set the
variables, redeploy, then generate the domain. The deploy log shows
`sign-in via https://login.msgraph.de (client …), roles [paperless.user]`.

**Checks:**

| Check | Expected |
|---|---|
| open the domain while signed out | redirect to `login.msgraph.de` |
| sign in as a user holding `paperless.user` | lands on the requested page; "Sign out" in the footer |
| sign in as a user without the role | "access has not been set up for your account" (log: `sign-in refused (denied)`) |
| `POST /upload` without a session (curl) | `401` |
| redeploy with `PAPERLESS_SESSION_KEY` set | still signed in |
| `/healthz` | `ok` without a session |

**Not covered yet:** every admitted user sees every document. Separation by
customer (plan §6) is still to do.

## 3. CI (P1)

Not yet in place. Follow §4 of the plan: a `scripts/ci/validate.sh` run inside
the Dockerfile, as report-rs does (RPT-CI-RAILWAY-1). Once it lands, a failing
check fails the Railway build and nothing is deployed. Railway PR environments
then run the same gate for paperless-rs pull requests.

## 4. WordCraft as the editor (P5, later)

Do **not** deploy WordCraft yet. It starts after P3 (identity), because editing
is a write, and after the archive keeps the original file: today it discards
the uploaded bytes after hashing, so there is no `.docx` to open. It never goes
into report-rs.

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
