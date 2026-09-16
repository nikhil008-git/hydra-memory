# hydra-memory

Small **Rust HTTP API** that stores **agent decisions as a graph** in **HydraDB**.

This is **not** HydraDB. HydraDB (`graph-node`) is the graph database. This repo is a thin backend in front of it: JSON in, Cypher out.

Happy-path demo: you curl **this** API (`:3000`). This process talks to Hydra internally. Do not curl Hydra for the demo.

```
curl → hydra-memory (:3000)
        → HTTP Cypher + Bearer + X-Graph-Namespace
        → graph-node (HydraDB)
        → local disk store (SlateDB files)
```

**Not:** a chatbot, LLM, UI, Postgres/SQLite, Hydra SaaS, or in-place UPDATE. History is a **new Decision node + `SUPERSEDES` edge**.

---

## Where the data lives (local Hydra)

Both of these are true:

- Data is **in Hydra** — `graph-node` owns the graph (vertices, edges, properties).
- Bytes sit on **your local disk** — the node writes durable files under the path you set when starting it.

This API (`hydra-memory`) is **stateless**. Postman JSON is not saved in this repo, not in SQLite, and not only in RAM for the long term. After a successful `POST /v1/decisions`, the graph lives inside **Hydra**.

### Local layout (default from the Run section)

When you start `graph-node` with `ROOT=/tmp/sgk-local`:

| Path | Role |
|------|------|
| `/tmp/sgk-local/store` | Durable store (`LOCAL_PATH`) — graph data on disk |
| `/tmp/sgk-local/cache` | Disposable cache |
| `/tmp/sgk-local/auth-token` | Bearer token file (must match `HYDRA_TOKEN`) |

`/tmp` can be cleared on reboot. For data you want to keep, point `LOCAL_PATH` at a permanent directory when starting `graph-node`.

### Inside the engine: SlateDB

Hydra does not use Postgres. Open-source `graph-node` persists the graph through **SlateDB** (embedded LSM / object-store–friendly key-value storage: WAL, SSTs, etc.). Locally that is files under `LOCAL_PATH`. In cloud setups the same idea can sit on object storage (e.g. S3).

You never open SlateDB from this repo — only Cypher over Hydra’s HTTP API.

### Graph shape (what you query)

```
(Agent {key})-[:DECIDED]->(Decision {key, text, ts})
(Decision)-[:SUPERSEDES]->(Decision)   # only if client sent supersedes
```

Lineage walks `SUPERSEDES` (newest → older). That chain in the JSON response is read **from Hydra**, which reads **from the local SlateDB store**.

---
## Endpoints

| Method | Path | Meaning |
|--------|------|---------|
| `POST` | `/v1/decisions` | Save “agent X decided text” (optional `supersedes`) |
| `GET` | `/v1/agents/{agent}/decisions` | List that agent’s decisions |
| `GET` | `/v1/decisions/{id}/lineage` | Walk `SUPERSEDES` (newest first: this decision, then what it replaced) |
| `GET` | `/healthz` | Process up; does **not** need Hydra |

Graph: `(Agent)-[:DECIDED]->(Decision)`. If the client sends `supersedes`, also `(new)-[:SUPERSEDES]->(old)`.

---

## Example

Base: `http://127.0.0.1:3000`  
POST header: `Content-Type: application/json`

**1. Health** — `GET /healthz`  
Returns 200 even if Hydra is down. Means this process is up.

**2. Save** — `POST /v1/decisions`

```json
{ "agent": "cursor", "text": "use SQLite" }
```

Copy `id` from the response (`OLD`). Graph: cursor `DECIDED` “use SQLite”.

**3. List** — `GET /v1/agents/cursor/decisions`  
All decisions for that agent. `OLD` should be in the list.

**4. Replace + history** — `POST /v1/decisions`

```json
{ "agent": "cursor", "text": "use Hydra", "supersedes": "<OLD>" }
```

The new `id` is `NEW`. Then `GET /v1/decisions/<NEW>/lineage`  
`chain` is newest first: Hydra, then SQLite.

---

## Requirements

Two processes (two repos):

1. **`graph-node`** — HydraDB OSS database server (separate clone, **not** inside this repo)
2. **`hydra-memory`** — this API on `http://127.0.0.1:3000`

| Process | Typical ports |
|---------|----------------|
| `graph-node` query HTTP | `http://127.0.0.1:18443` |
| `graph-node` admin `/readyz` | `http://127.0.0.1:19091` |
| this API | `http://127.0.0.1:3000` |

Clone Hydra next to this project, e.g.:

```text
Developer/
  hydradb/          ← OSS graph-node (follow its AGENTS.md)
  hydra-memory/     ← this repo
```

Native deps and first-time env for Hydra are documented in **`hydradb/AGENTS.md`** (brew packages, `/tmp/sgk-env.sh`, etc.). Token ≥ 32 characters. Local only: `GRAPH_ALLOW_PLAINTEXT=true`. `X-Graph-Namespace` / `HYDRA_NAMESPACE` must match `GRAPH_NAMESPACE` (e.g. `local`).

A listening port is not enough — Hydra’s own CREATE + MATCH smoke in `AGENTS.md` should pass before you demo this API.

Rust: recent stable (`edition = "2024"`). Then `cargo build` in this repo.

This API does **not** load `.env` by itself. Export vars in the same shell as `cargo run`. If you keep a `.env` of `export KEY=...` lines only (no shell commands), you can `set -a; source .env; set +a`.

---

## Env

```bash
export BIND=127.0.0.1:3000
export HYDRA_URL=http://127.0.0.1:18443
export HYDRA_TOKEN=local-dev-auth-token-32-characters-long
export HYDRA_NAMESPACE=local
export HYDRA_GRAPH=default
export HYDRA_CELL=cell-0
```

These must match how you started `graph-node` (URL/ports, token file, namespace, graph id, cell id). Missing `HYDRA_TOKEN` panics on startup.

Localhost only. No auth on `:3000` in this MVP.

---

## Run

### Terminal 1 — start Hydra (`graph-node`)

From the **hydradb** tree (after `AGENTS.md` setup / `source /tmp/sgk-env.sh` if you use it):

```bash
cd /path/to/hydradb
# ensure brew lib paths if needed (see AGENTS.md), then:

ROOT=/tmp/sgk-local
rm -rf -- "$ROOT"
mkdir -p "$ROOT/store" "$ROOT/cache"
printf '%s\n' 'local-dev-auth-token-32-characters-long' >"$ROOT/auth-token"

export CLOUD_PROVIDER=local LOCAL_PATH="$ROOT/store"
export GRAPH_NAMESPACE=local GRAPH_ID=default
export GRAPH_CELL_ID=cell-0 GRAPH_CELLS=cell-0 GRAPH_DATA_PATH=data
export GRAPH_ALLOW_PLAINTEXT=true GRAPH_AUTH_TOKEN_FILE="$ROOT/auth-token"
export GRAPH_DATA_CACHE_BYTES=67108864 GRAPH_DATA_CACHE_DIR="$ROOT/cache"
export GRAPH_NODE_ID=node-0
export GRAPH_BOLT_ADDR=127.0.0.1:17687 GRAPH_ADVERTISED_BOLT_ADDR=127.0.0.1:17687
export GRAPH_BOLT_NODE_ADDRESSES=node-0=127.0.0.1:17687
export GRAPH_HTTP_ADDR=127.0.0.1:18443 GRAPH_ADMIN_ADDR=127.0.0.1:19091
export RUST_MIN_STACK=33554432 RUST_LOG=info

cargo build --locked --features server-runtime --bin graph-node
./target/debug/graph-node
```

Leave that terminal running. In another shell:

```bash
curl -fsS http://127.0.0.1:19091/readyz && echo READY
```

Full first-time install and CREATE+MATCH smoke: **`hydradb/AGENTS.md`**.

### Terminal 2 — start this API

```bash
cd /path/to/hydra-memory
# exports from Env above (token must match $ROOT/auth-token)
cargo run
```

You should see `http://127.0.0.1:3000`. Happy-path curls hit **this** port, not Hydra directly.

---

## Test (four curls)

**1. Health** (Hydra not required)

```bash
curl -sS -D- http://127.0.0.1:3000/healthz
```

Expect HTTP 200.

**2. Save a decision**

```bash
curl -sS -X POST http://127.0.0.1:3000/v1/decisions \
  -H 'content-type: application/json' \
  -d '{"agent":"cursor","text":"use SQLite"}'
```

Expect `{"id":"<uuid>","agent":"cursor","text":"use SQLite"}`. Copy `id` → `OLD`.

**3. List**

```bash
curl -sS http://127.0.0.1:3000/v1/agents/cursor/decisions
```

Expect that decision in `decisions` (`id`, `text`, `ts`).

**4. Replace + lineage**

```bash
curl -sS -X POST http://127.0.0.1:3000/v1/decisions \
  -H 'content-type: application/json' \
  -d '{"agent":"cursor","text":"use Hydra","supersedes":"<OLD>"}'
```

Copy the new `id` → `NEW`.

```bash
curl -sS http://127.0.0.1:3000/v1/decisions/<NEW>/lineage
```

`chain` is newest first: “use Hydra”, then “use SQLite”.

`supersedes` omitted or null = no `SUPERSEDES` edge. Hydra does not infer replacements.

---

## Errors

| Situation | Status |
|-----------|--------|
| Hydra down / Hydra 401 / bad Hydra HTTP | **502** on POST/list/lineage |
| Unknown `supersedes` id, or unknown lineage id | **400** |
| `/healthz` with Hydra down | **200** |

---

## Layout

```
src/main.rs    # env, bind, routes
src/http.rs    # JSON handlers
src/hydra.rs   # POST Cypher to graph-node
src/cypher.rs  # query strings
src/error.rs   # 400 / 502
```

---

## What this is (honest)

Local Hydra (`graph-node`). This API records agent decisions as a graph: POST A, POST B with `supersedes` A, GET lineage. OpenCypher over Hydra’s HTTP query API.

It is not Hydra, not their hosted SaaS, not a founding-engineer take-home clone of the database.
