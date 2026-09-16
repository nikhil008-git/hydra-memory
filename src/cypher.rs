pub const CREATE: &str =
    "MERGE (a {id: $agent}) CREATE (a)-[:DECIDED]->(d {id: $id, text: $text, ts: $ts})";
pub const GET: &str =
    "MATCH (d {id: $id}) RETURN d.id AS id, d.text AS text, d.ts AS ts";
pub const SUPERSEDE: &str =
    "MATCH (n {id: $new}), (o {id: $old}) CREATE (n)-[:SUPERSEDES]->(o)";
pub const LIST: &str =
    "MATCH (a {id: $agent})-[:DECIDED]->(d) RETURN d.id AS id, d.text AS text, d.ts AS ts";
pub const HOP: &str =
    "MATCH (d {id: $id})-[:SUPERSEDES]->(o) RETURN o.id AS id, o.text AS text, o.ts AS ts";