# paperless-rs — Railway image, now a THIN WRAPPER around the real archive.
#
# ---------------------------------------------------------------------------
# Why this Dockerfile no longer builds THIS repo's own crates
# ---------------------------------------------------------------------------
#
# This repo (`paperless-rs`) was the original scaffold for document intake —
# the S-2 dedup gate, the tokenization seam probe — but per
# `tesseract-rs/CLAUDE.md` § "tesseract-paperless — document intake lands
# here, feature-gated" (2026-08-24), the work continued and SHIPPED inside
# `AdaWorldAPI/tesseract-rs` as `crates/tesseract-paperless` (the gate +
# doc.v1 -> DocIr seam) and `crates/tesseract-paperless-web` (the Railway
# archive: upload -> S-2 gate -> OCR -> lancedb -> Tantivy search ->
# paperless-ngx-shaped UI). That entry says it plainly:
#
#   "paperless-rs is now a DEAD COPY. Do not sync the two — there is
#    nothing to sync back to."
#
# So this repo's own `crates/` (paperless-kv / paperless-intake /
# paperless-token) are superseded probes, not the running service — see this
# repo's own CLAUDE.md § Status, which predates that supersession and should
# be read with that correction in mind.
#
# What this Dockerfile does instead: build tesseract-rs's REAL
# `tesseract-paperless-web` binary, exactly as
# `tesseract-rs/crates/tesseract-paperless-web/Dockerfile` does, just from a
# fresh clone rather than a build context that already contains the repo.
# This exists so that whatever Railway service is configured to build from
# THIS repo gets a real, working archive on its next deploy, without needing
# to be repointed at a different GitHub repo in the Railway dashboard.
#
# If you CAN repoint the Railway service at `AdaWorldAPI/tesseract-rs`
# directly (Dockerfile path `crates/tesseract-paperless-web/Dockerfile`),
# that is the more direct route and this wrapper becomes unnecessary — but
# this file makes the fix land either way.

# ── builder ─────────────────────────────────────────────────────────────────
FROM rust:1.97.1-bookworm AS builder

# `git` for the four clones (tesseract-rs + its three siblings);
# `protobuf-compiler` + `libprotobuf-dev` for `lance-encoding`'s build script
# (a `lancedb`/`lance` transitive dep pulled in by the `store` feature —
# Debian's `protobuf-compiler` alone ships `protoc` but not the well-known
# includes like `google/protobuf/empty.proto`; both packages are required
# together, per tesseract-rs's own Dockerfile comment).
RUN apt-get update && apt-get install -y --no-install-recommends \
      git protobuf-compiler libprotobuf-dev \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /src

# Pin every clone for a reproducible build. Unset = each repo's default
# branch (what CI uses). Bump these deliberately, in a reviewed change, not
# by letting a branch ref float.
ARG TESSERACT_RS_REF=""
ARG LANCE_GRAPH_REF=""
ARG NDARRAY_REF=""
ARG OGAR_REF=""
RUN set -eu; \
    clone() { \
      name="$1"; ref="$2"; \
      git clone --depth 1 ${ref:+--branch "$ref"} \
        "https://github.com/AdaWorldAPI/$name.git" "/src/$name"; \
      echo "cloned $name @ ${ref:-default branch}"; \
    }; \
    clone tesseract-rs "$TESSERACT_RS_REF"; \
    clone lance-graph "$LANCE_GRAPH_REF"; \
    clone ndarray "$NDARRAY_REF"; \
    clone OGAR "$OGAR_REF"

WORKDIR /src/tesseract-rs

# Trim ONLY tesseract-ocr-python (the pyo3/maturin wheel crate — needs system
# Python headers this image has no use for). tesseract-ogar and
# tesseract-paperless-web both stay IN the workspace — the whole point of
# this image. Line-delete, not a substring substitution — see
# tesseract-rs's own Dockerfile comment for why (codex P1 on PR #88): the
# members list is one entry per line with a trailing comma, so a leading
# `, "X"` pattern never matches. The guard below turns a silent no-op into a
# hard build failure.
RUN sed -i '/"crates\/tesseract-ocr-python"/d' Cargo.toml; \
    if grep -q '"crates/tesseract-ocr-python"' Cargo.toml; then \
      echo "FATAL: failed to trim tesseract-ocr-python from workspace members (Cargo.toml reformatted?)" >&2; \
      exit 1; \
    fi

# Strip the release binary via cargo (no binutils needed in the slim runtime).
ENV CARGO_PROFILE_RELEASE_STRIP=true
# No BuildKit cache mount — Railway's builder rejects the mount syntax, and a
# plain build works on every builder (just no cross-deploy caching).
RUN cargo build --release -p tesseract-paperless-web \
    && cp target/release/tesseract-paperless-web /usr/local/bin/tesseract-paperless-web

# ── runtime ─────────────────────────────────────────────────────────────────
# debian-slim only for glibc + libgcc; zero OCR/TLS system libraries.
FROM debian:bookworm-slim AS runtime

WORKDIR /app
COPY --from=builder /usr/local/bin/tesseract-paperless-web /app/tesseract-paperless-web
# The eng model (~4 MB): eng.lstm + unicharset + recoder + the three DAWGs.
COPY --from=builder /src/tesseract-rs/corpus/model /app/model
# deepnsm's 4,096-word COCA vocabulary CSVs — the reasoning layer's SPO/
# `NarsTruth` extraction. Absence is NOT a build failure (degrades to
# `reasoner: None`, `spo_json: None`); this COPY exists so a Railway deploy
# gets the real reasoning layer by default.
COPY --from=builder /src/lance-graph/crates/deepnsm/word_frequency /app/word_frequency

# Both the lancedb archive AND the Tantivy search index need a writable,
# ideally PERSISTENT directory — on Railway, mount a volume at /app/data
# (Railway's volume UI lets you pick the mount path; point it here) or both
# are lost on every redeploy.
RUN mkdir -p /app/data \
    && useradd --system --uid 10001 --no-create-home appuser \
    && chown -R appuser:appuser /app/data
USER appuser

ENV MODEL_DIR=/app/model
ENV ARCHIVE_URI=/app/data/archive.lance
ENV SEARCH_INDEX_DIR=/app/data/search_index
ENV DEEPNSM_VOCAB_DIR=/app/word_frequency
# NOTE: the listen port is intentionally NOT set here. The binary binds
# 0.0.0.0:$PORT at runtime; Railway injects PORT itself. For a local run pass
# `-e PORT=8080`. Do not add `ENV PORT=...` — that would shadow Railway's value.

CMD ["/app/tesseract-paperless-web"]
