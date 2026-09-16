// Hydra's local query engine only accepts one-hop CREATE with integer `id`.
// API string ids live in `key`; `id` is a stable hash for Hydra.

pub const CREATE: &str = "CREATE (a {id: $aid, key: $agent})-[:DECIDED]->(d {id: $did, key: $id, text: $text, ts: $ts})";
pub const GET: &str =
    "MATCH (d {key: $id}) RETURN d.key AS id, d.text AS text, d.ts AS ts";
pub const SUPERSEDE: &str =
    "CREATE (n {id: $nid, key: $new, text: $new_text, ts: $new_ts})-[:SUPERSEDES]->(o {id: $oid, key: $old, text: $old_text, ts: $old_ts})";
pub const LIST: &str =
    "MATCH (a {key: $agent})-[:DECIDED]->(d) RETURN d.key AS id, d.text AS text, d.ts AS ts";
pub const HOP: &str =
    "MATCH (d {key: $id})-[:SUPERSEDES]->(o) RETURN o.key AS id, o.text AS text, o.ts AS ts";
