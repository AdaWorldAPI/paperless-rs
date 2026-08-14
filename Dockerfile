# paperless-rs — the assembly point.
#
# ---------------------------------------------------------------------------
# Why this has no sibling-clone step
# ---------------------------------------------------------------------------
#
# tesseract-rs's own Dockerfiles clone lance-graph / ndarray / OGAR into /src
# before building, because its crates wire siblings by *path*
# (`../../../lance-graph/...`). Those paths only resolve when the siblings sit
# beside the checkout, so the image has to place them.
#
# This repo wires siblings by **git rev + [patch]** instead (see Cargo.toml),
# so cargo fetches them itself at the pinned revs. That removes the clone step,
# removes the "did the sibling drift?" question — the revs are in Cargo.toml,
# reviewed like any other change — and makes an image build reproduce exactly
# what a laptop build resolved.
#
# The cost is honest and worth naming: cargo re-fetches the git sources on a
# cold layer. `cargo fetch` is split into its own layer below so that cost is
# paid once per dependency change, not once per source edit.

FROM rust:1.97.1-bookworm AS builder

# Matches rust-toolchain.toml. The base tag and the pin must move together —
# lance-graph #896 caught exactly this drift shape (pins living only in prose,
# CI install lines and base tags, disagreeing with each other).
WORKDIR /src

# Dependency layer: manifests only, so editing source does not re-fetch the
# git sources. The stub member sources are required for `cargo fetch` to
# resolve the workspace.
COPY Cargo.toml rust-toolchain.toml ./
COPY crates/paperless-kv/Cargo.toml crates/paperless-kv/
COPY crates/paperless-intake/Cargo.toml crates/paperless-intake/
RUN mkdir -p crates/paperless-kv/src crates/paperless-intake/src \
 && echo '' > crates/paperless-kv/src/lib.rs \
 && echo '' > crates/paperless-intake/src/lib.rs \
 && cargo fetch --locked 2>/dev/null || cargo fetch

# Real sources.
COPY . .

# `--offline` proves the dependency layer above actually captured everything:
# if a source edit somehow pulls a new dependency, this fails loudly here
# rather than silently re-fetching and making the layer split a lie.
ENV CARGO_PROFILE_RELEASE_STRIP=true
RUN cargo build --release --workspace --offline \
 && cargo test  --release --workspace --offline

# ---------------------------------------------------------------------------
FROM debian:bookworm-slim AS runtime

# No OCR system libraries. tesseract-rs is a pure-Rust transcode — no
# libtesseract, no leptonica — so the runtime is the glibc base plus the
# binary. That property is worth protecting: if this stanza ever needs an
# apt-get for an OCR library, something has regressed upstream.
RUN useradd --system --uid 10001 --no-create-home appuser
USER appuser
WORKDIR /app

# Placeholder: paperless-intake is a stub today (see CLAUDE.md § Status), so
# there is no binary to ship yet. When intake gains one, COPY it here along
# with the model directory:
#   COPY --from=builder /src/target/release/<bin> /app/<bin>
#   COPY --from=builder /src/corpus/model /app/model
#   ENV MODEL_DIR=/app/model
#
# Deliberately no `ENV PORT` — Railway injects it, and hardcoding it makes the
# container disagree with its platform (tesseract-ocr-web's own note).
CMD ["/bin/sh", "-c", "echo 'paperless-rs: no runtime binary yet — see CLAUDE.md § Status' && exit 1"]
