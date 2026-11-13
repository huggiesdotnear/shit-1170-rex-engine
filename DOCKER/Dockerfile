# ========================================
# shit-1170-rex-engine — Dockerfile
# requires BuildKit for --mount=type=cache.
#
# two stages:
#   frontend     — bun build compiles `frontend/` (HTML + TS) into `static/`
#   rust-builder — rust:slim-bookworm, builds the release binary with cached
#                  cargo/registry + cargo/git mounts
#
# final image is debian:bookworm-slim so the binary's libc matches the one
# it was linked against.
#
# NOTE: the vault. the sqlite db lives OUTSIDE the image, in a volume
# mounted at /home/shit-1170/.shit-1170-rex-engine — see DOCS/VAULT.md. the
# image carries no db, so a rebuild never touches the data.
# ========================================
# syntax=docker/dockerfile:1.7

# ========================================
# Stage 1: frontend bundle (bun).
# ========================================
FROM oven/bun:alpine AS frontend

WORKDIR /src

# copy the package manifest + lockfile FIRST so node_modules is
# cached independently of source-only edits.
COPY package.json bun.lock ./
COPY tsconfig.json ./
COPY middleman ./middleman
COPY frontend ./frontend
COPY build.ts ./build.ts
# --- WHY: frontend/llm.md comes along ---
# build.ts copies frontend/llm.md to static/llms.txt — the served name
# stays llms.txt and the source stays markdown. the file lives in the
# frontend folder, so the `COPY frontend ./frontend` above already
# brings it in.

# install only what's needed for the frontend build. the middleman/
# and frontend trees import `preact` indirectly (components that call
# useState / h) so we need it on disk for bun to resolve.
# `--frozen-lockfile` is too strict across bun minor versions since
# the lockfile format can shift; package.json is tiny so a free-form
# install is fine.
#
# --- WHY: the workspace install links the local package ---
# the root package.json declares `workspaces: ["middleman"]`, so
# `bun install` links `@huggies/shit-1170-rex-engine-middleman` from
# node_modules/ to middleman/ — no registry round trip.
RUN bun install

# build the middleman package to dist/ BEFORE the frontend bundle.
#
# --- WHY: the frontend imports the package by name ---
# the frontend does `import { api_trezu_treasuries_get } from
# "@huggies/shit-1170-rex-engine-middleman"`, and that package's
# `exports` field points at `./dist/index.js`. the workspace link
# alone is not enough: without dist/ the import does not resolve and
# the frontend build fails with "Could not resolve".
#
# `dist/` is gitignored, so it never exists in the build context —
# it has to be built here, inside the image.
RUN cd /src/middleman && bun run build

# produce minified static files at /src/static, preserving the
# `app/` subdirectory so the nested pages keep their depth.
#
# --- WHY: no --public-path here (unlike the graupelos Dockerfile) ---
# the pages reference their assets RELATIVELY (`./favicon.svg` from
# /index.html, `../favicon.svg` from /app/index.html). bun rewrites
# those to match each page's depth. forcing `--public-path=/` would
# rewrite every reference to an absolute /name.ext, which the
# ServeDir can still resolve, but it breaks the relative layout this
# repo depends on — so leave the default.
#
# the asset naming templates embed `[hash]` so two pages sharing the
# same CSS path don't collide on disk.
#
# --- WHY: `[dir]` in the chunk and asset naming ---
# without it, every page's css and js lands flat in static/. the root
# index.html and app/index.html then both produce a file called
# `index-<hash>.css` — and when their content happens to be identical
# (which it is for pages that share the same css), the hash matches
# too and bun refuses to build: "Multiple files share the same output
# path". keeping the directory puts each page's assets beside it, so
# two pages can never collide no matter what their content is.
#
# `find` instead of a `./**/*.html` glob: globstar is not enabled in
# the default sh that runs this, so the glob would silently match
# nothing and we'd ship an empty static folder. `find` also picks up
# the nested pages under app/contract/ and app/token/.
#
# --- WHY: sw.js is its own entry point ---
# the service worker has to land at the ROOT of the static folder as a
# standalone, unbundled file, because the browser loads /sw.js as a
# CLASSIC script and gives it the scope `/` (which is what lets it
# control every page under /app/). if bun bundled it, the worker would
# come out wrapped in a CommonJS interop shim (`export default s()`),
# which is a SyntaxError in a classic worker — the worker would die
# silently and the app would never be installable.
#
# so sw.js is listed as an entry here, NOT referenced from any html.
# a `<script src="./sw.js">` in a page would make bun inline it into
# that page's chunk and leave no /sw.js to register.
#
# the manifest does NOT need to be here: bun picks it up automatically
# as an asset, because the pages link to it with
# `<link rel="manifest" href="...">`.
#
# --- WHY: --asset-naming has NO [hash] in it ---
# bun does NOT rewrite the `icons[].src` paths inside the manifest —
# it treats the manifest as opaque JSON, so whatever path is written
# there is what the browser will ask for. the manifest says
# `./lib/img/pwa/icon-192.png`, so the built file has to be at exactly
# that path. a hash would rename it to `icon-192-<hash>.png` and every
# icon would 404.
#
# the same reason the html links to all twelve icons with
# `<link rel="icon" ...>`: bun only copies an asset into the output if
# something references it, and the manifest alone does not count.
# the css keeps its hash — that comes from --chunk-naming, and hashed
# css names are what let two pages with identical css coexist.
#
# the shared build script discovers every frontend HTML entry point,
# includes sw.js as a standalone entry point, and writes the production
# bundle to /src/static.
RUN cd /src \
    && bun run build.ts

# ========================================
# Stage 2: rust builder.
# ========================================
FROM rust:slim-bookworm AS rust-builder

WORKDIR /build

# the backend is plain axum — no near-kit, no aws-lc-sys — so the
# only system dep needed is a linker plus pkg-config for openssl-sys
# via rustls backends.
RUN apt-get update \
    && apt-get install -y --no-install-recommends \
        pkg-config libssl-dev \
    && rm -rf /var/lib/apt/lists/*

# layer the dependency manifest first so source-only edits skip the
# dependency build. /build/target lives on the layer filesystem (not a
# cache mount) so the final binary is visible to the next stage's COPY.
#
# --- WHY: the bins are NOT built here ---
# this stage exists to warm the dependency cache. it builds one empty
# main and throws the result away. the real build below builds the
# bins, and if this stage built them too the cleanup below would have to
# know about every one of them.
COPY Cargo.toml Cargo.lock ./
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    mkdir -p src \
    && echo "fn main() {}" > src/main.rs \
    && cargo build --release \
    && rm -rf src target/release

# real build. NOTE: no `static/` copy here — `static/` is gitignored and
# never exists in the build context, and the binary doesn't read it at
# compile time (ServeDir opens it at runtime). the frontend bundle is
# copied from the frontend stage straight into the runtime stage below.
COPY src ./src
RUN --mount=type=cache,target=/usr/local/cargo/registry,sharing=locked \
    --mount=type=cache,target=/usr/local/cargo/git,sharing=locked \
    touch src/main.rs src/bin/*.rs \
    && cargo build --release --bins \
    && mkdir -p target/release/db-admin \
    && for bin in list prune backup export import; do \
         cp "target/release/db_admin_$bin" "target/release/db-admin/db-admin-$bin"; \
       done

# ========================================
# Stage 3: minimal runtime image.
# debian:bookworm-slim matches the libc the rust builder linked against.
# ========================================
FROM debian:bookworm-slim

# ca-certificates for talking to public NEAR RPC / intear api over HTTPS.
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/* \
    && useradd --system --uid 1000 --home /home/shit-1170 \
        --shell /usr/sbin/nologin shit1170

# --- WHY: the vault folder is created in the image, owned by the app
# user ---
# a named volume mounted over a folder INHERITS that folder's
# ownership the first time it is created — but only if the folder
# exists in the image. a volume mounted over a folder that does NOT
# exist is created empty and owned by root, and then the app (running
# as uid 1000) cannot write the db into it.
#
# so the folder is created here, owned by the shit1170 user, and the
# volume takes that ownership when it is first mounted.
#
# --- WHY the user is `shit1170` and not `shit-1170` ---
# a LINUX USER NAME may not contain a hyphen. `useradd` refuses
# `shit-1170`, and a `chown shit-1170:...` would not resolve. the
# home FOLDER keeps the hyphen, because that is a path and not a name.
RUN mkdir -p /home/shit-1170/.shit-1170-rex-engine \
    && chown -R shit1170:shit1170 /home/shit-1170

COPY --from=rust-builder /build/target/release/shit-1170-rex-engine /usr/local/bin/shit-1170-backend
# --- WHY: the db admin bins are in the image too ---
# the bins are the way to do db work on a vault from outside the app,
# and the production vault lives on a docker volume. so the bins have
# to be reachable inside the container, or the only way to prune the
# production vault would be to copy the db off the host first.
#
# `docker compose exec shit-1170-rex-engine-backend db-admin-prune --db ...`
# is the command that does it.
#
# --- WHY: they come in as ONE COPY of a folder ---
# one COPY of a folder is one source path for the builder to resolve,
# and a folder copy takes whatever the builder produced. five separate
# COPY lines meant five separate lookups, and a lookup that missed one
# file failed the whole build with a checksum error while the rest of
# the layer sat cached.
COPY --from=rust-builder /build/target/release/db-admin /usr/local/bin/
COPY --from=frontend /src/static /app/static

# --- WHY: ServeDir::new("static") is a RELATIVE path ---
# app_router_fun serves the static folder with a relative path, so the
# process cwd decides where `static/` is looked up. it must be /app.
WORKDIR /app

# defaults baked into the binary: 0.0.0.0:54733. 0.0.0.0 is required
# inside a container — binding localhost inside the container would
# make the published port unreachable from the host.
ENV SHIT_1170_HOST=0.0.0.0
ENV SHIT_1170_PORT=54733
ENV RUST_LOG=shit_1170_rex_engine=info
# --- WHY --- the vault path is baked in as the default: the db lives
# under the user's home, which is /home/shit-1170 in the image, and the
# volume is mounted exactly there. the app's own default is
# `${HOME}/.shit-1170-rex-engine/shit-1170-rex-engine.db`, so it needs no
# env at all — this just says where that is.
ENV SHIT_1170_VAULT_DB_LOCATION=/home/shit-1170/.shit-1170-rex-engine/shit-1170-rex-engine.db
EXPOSE 54733

USER shit1170
ENTRYPOINT ["/usr/local/bin/shit-1170-backend"]
# ========================================