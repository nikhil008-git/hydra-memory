# hydra-memory

Small **Rust HTTP API** that stores **agent decisions as a graph** in **HydraDB**.

This is **not** HydraDB. HydraDB (`graph-node`) is the graph database. This repo is a thin backend in front of it: JSON in, Cypher out.

Happy-path demo: you curl **this** API (`:3000`). This process talks to Hydra internally. Do not curl Hydra for the demo.

```
curl → hydra-memory (:3000)
        → HTTP Cypher + Bearer + X-Graph-Namespace
        → graph-node (HydraDB)
```

**Not:** a chatbot, LLM, UI, Postgres/SQLite, Hydra SaaS, or in-place UPDATE. History is a **new Decision node + `SUPERSEDES` edge**.

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

Two processes:

1. **`graph-node`** (Hydra OSS) — query HTTP `http://127.0.0.1:18443`, admin `http://127.0.0.1:19091`
2. **`hydra-memory`** — `http://127.0.0.1:3000`

Start Hydra first. A listening port is not enough; Hydra’s own CREATE + MATCH smoke should work (see Hydra `AGENTS.md`). Token ≥ 32 chars. `X-Graph-Namespace` must match `GRAPH_NAMESPACE` (e.g. `local`). `GRAPH_ALLOW_PLAINTEXT=true` for local.

Rust: recent stable (`edition = "2024"`). Then `cargo build` in this repo.

This API does **not** load `.env` by itself. Export vars in the same shell as `cargo run` (or `set -a; source .env; set +a` if the file is only `export KEY=...` lines).

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

`HYDRA_TOKEN` must match the token `graph-node` was started with. Missing `HYDRA_TOKEN` panics on startup.

Localhost only. No auth on `:3000` in this MVP.

---

## Run

```bash
# terminal 1: graph-node (from the hydradb tree, per its AGENTS.md)
curl -fsS http://127.0.0.1:19091/readyz && echo READY

# terminal 2: this API
cd /path/to/hydra-memory
# exports from above
cargo run
```

You should see `http://127.0.0.1:3000`.

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
