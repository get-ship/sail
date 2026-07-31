//! Typed contract glue: Json newtype, schema helpers, codegen (TS + MCP + CLI).

use schemars::JsonSchema;
use schemars::schema::{InstanceType, Schema, SchemaObject, SingleOrVec};
use schemars::schema_for;
use serde::{Deserialize, Serialize};
use serde_json::{Value, json};

/// Explicit JSON blob for truly dynamic payloads (task docs, layout trees, etc.).
/// Prefer concrete types; use this instead of bare `serde_json::Value` at contract edges.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Json(pub Value);

impl Json {
    pub fn into_inner(self) -> Value {
        self.0
    }

    pub fn as_value(&self) -> &Value {
        &self.0
    }
}

impl From<Value> for Json {
    fn from(v: Value) -> Self {
        Self(v)
    }
}

impl From<Json> for Value {
    fn from(j: Json) -> Self {
        j.0
    }
}

impl JsonSchema for Json {
    fn schema_name() -> String {
        "Json".into()
    }

    fn json_schema(_gen: &mut schemars::r#gen::SchemaGenerator) -> Schema {
        // unrestricted JSON value
        Schema::Object(SchemaObject {
            instance_type: None,
            ..Default::default()
        })
    }
}

/// Empty args object `{}`.
#[derive(Debug, Clone, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub struct EmptyArgs {}

/// One method's typed contract metadata (hand-written table lives in `contracts`).
#[derive(Debug, Clone, Copy)]
pub struct MethodContract {
    pub service: &'static str,
    pub method: &'static str,
    pub mcp_tool: &'static str,
    pub description: &'static str,
    /// JSON Schema for request body (object).
    pub input_schema: fn() -> Value,
    /// JSON Schema for response body.
    pub output_schema: fn() -> Value,
    /// TypeScript interface name for args.
    pub ts_args: &'static str,
    /// TypeScript interface name for return.
    pub ts_ret: &'static str,
    /// TypeScript type body for args (interface fields).
    pub ts_args_body: &'static str,
    /// TypeScript type body for return.
    pub ts_ret_body: &'static str,
}

impl MethodContract {
    pub fn rpc_name(&self) -> String {
        format!("{}/{}", self.service, self.method)
    }

    pub fn connect_path(&self) -> String {
        format!("/sail.ops.v1.{}/{}", self.service, self.method)
    }

    pub fn ts_fn_name(&self) -> String {
        // SailAuthoring + Surface → sailAuthoringSurface
        let mut s = String::new();
        s.push_str(&lowercase_first(self.service));
        s.push_str(self.method);
        s
    }
}

fn lowercase_first(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_lowercase().collect::<String>() + c.as_str(),
    }
}

/// Build a JSON Schema object for type `T`.
pub fn schema_value_for<T: JsonSchema>() -> Value {
    let root = schema_for!(T);
    serde_json::to_value(root).unwrap_or_else(|_| json!({"type": "object"}))
}

/// Extract the object-shaped schema suitable for MCP `inputSchema`.
///
/// Copies `properties` / `required` / `additionalProperties` **and**
/// `definitions` / `$defs` so nested `$ref`s resolve (e.g. `TaskActor`).
pub fn object_schema_for<T: JsonSchema>() -> Value {
    let full = schema_value_for::<T>();
    // schemars 0.8 emits { "$schema", "title", "type", "properties", "definitions", ... }
    if full.get("type").and_then(|t| t.as_str()) == Some("object")
        || full.get("properties").is_some()
    {
        let mut obj = serde_json::Map::new();
        obj.insert("type".into(), json!("object"));
        if let Some(p) = full.get("properties") {
            obj.insert("properties".into(), p.clone());
        } else {
            obj.insert("properties".into(), json!({}));
        }
        if let Some(r) = full.get("required") {
            obj.insert("required".into(), r.clone());
        }
        if let Some(a) = full.get("additionalProperties") {
            obj.insert("additionalProperties".into(), a.clone());
        }
        // Preserve nested type defs so `$ref: "#/definitions/…"` resolves.
        if let Some(d) = full.get("definitions") {
            obj.insert("definitions".into(), d.clone());
        }
        if let Some(d) = full.get("$defs") {
            obj.insert("$defs".into(), d.clone());
        }
        return Value::Object(obj);
    }
    // Fallback: empty object
    json!({"type": "object", "properties": {}})
}

/// Every local `$ref` in `schema` must resolve against root `definitions` / `$defs`.
pub fn schema_refs_resolve(schema: &Value) -> Result<(), String> {
    let definitions = schema.get("definitions").cloned().unwrap_or(json!({}));
    let dollar_defs = schema.get("$defs").cloned().unwrap_or(json!({}));
    fn walk(
        node: &Value,
        definitions: &Value,
        dollar_defs: &Value,
        path: &str,
    ) -> Result<(), String> {
        match node {
            Value::Object(map) => {
                if let Some(r) = map.get("$ref").and_then(|v| v.as_str()) {
                    if let Some(name) = r.strip_prefix("#/definitions/") {
                        if definitions.get(name).is_none() {
                            return Err(format!("{path}: unresolved $ref {r}"));
                        }
                    } else if let Some(name) = r.strip_prefix("#/$defs/") {
                        if dollar_defs.get(name).is_none() {
                            return Err(format!("{path}: unresolved $ref {r}"));
                        }
                    } else if r.starts_with('#') {
                        return Err(format!("{path}: unsupported local $ref form {r}"));
                    }
                }
                for (k, v) in map {
                    if k == "$ref" {
                        continue;
                    }
                    walk(v, definitions, dollar_defs, &format!("{path}.{k}"))?;
                }
                Ok(())
            }
            Value::Array(arr) => {
                for (i, v) in arr.iter().enumerate() {
                    walk(v, definitions, dollar_defs, &format!("{path}[{i}]"))?;
                }
                Ok(())
            }
            _ => Ok(()),
        }
    }
    walk(schema, &definitions, &dollar_defs, "$")
}

/// Map a JSON Schema property node to a TypeScript type expression.
fn schema_to_ts_type(node: &Value) -> String {
    // anyOf with null → optional inner
    if let Some(any) = node.get("anyOf").and_then(|a| a.as_array()) {
        let non_null: Vec<&Value> = any
            .iter()
            .filter(|v| v.get("type").and_then(|t| t.as_str()) != Some("null"))
            .collect();
        if non_null.len() == 1 {
            return schema_to_ts_type(non_null[0]);
        }
        if !non_null.is_empty() {
            let parts: Vec<String> = non_null.iter().map(|v| schema_to_ts_type(v)).collect();
            return parts.join(" | ");
        }
    }
    if let Some(r) = node.get("$ref").and_then(|v| v.as_str()) {
        if r.contains("Json") {
            return "JsonValue".into();
        }
        // Nested object refs (TaskActor, etc.)
        if let Some(name) = r.rsplit('/').next() {
            return name.to_string();
        }
    }
    // type may be string or array (["string","null"])
    let ty = node.get("type");
    if let Some(arr) = ty.and_then(|t| t.as_array()) {
        let non_null: Vec<&str> = arr
            .iter()
            .filter_map(|v| v.as_str())
            .filter(|s| *s != "null")
            .collect();
        if non_null.len() == 1 {
            return json_type_to_ts(non_null[0], node);
        }
    }
    if let Some(s) = ty.and_then(|t| t.as_str()) {
        return json_type_to_ts(s, node);
    }
    "JsonValue".into()
}

fn json_type_to_ts(t: &str, node: &Value) -> String {
    match t {
        "string" => "string".into(),
        "boolean" => "boolean".into(),
        "integer" | "number" => "number".into(),
        "array" => {
            let items = node
                .get("items")
                .map(schema_to_ts_type)
                .unwrap_or_else(|| "JsonValue".into());
            format!("{items}[]")
        }
        "object" => {
            if node.get("properties").is_some() {
                // Inline shallow object
                let body = ts_interface_body_from_schema(node);
                format!("{{\n{body}}}")
            } else {
                "JsonObject".into()
            }
        }
        _ => "JsonValue".into(),
    }
}

fn prop_is_required(schema: &Value, name: &str) -> bool {
    schema
        .get("required")
        .and_then(|r| r.as_array())
        .map(|arr| arr.iter().any(|v| v.as_str() == Some(name)))
        .unwrap_or(false)
}

fn prop_allows_null(node: &Value) -> bool {
    if node.get("type").and_then(|t| t.as_str()) == Some("null") {
        return true;
    }
    if let Some(arr) = node.get("type").and_then(|t| t.as_array()) {
        if arr.iter().any(|v| v.as_str() == Some("null")) {
            return true;
        }
    }
    if let Some(any) = node.get("anyOf").and_then(|a| a.as_array()) {
        return any
            .iter()
            .any(|v| v.get("type").and_then(|t| t.as_str()) == Some("null"));
    }
    false
}

/// Emit TS interface field lines from an object JSON Schema (from schemars).
pub fn ts_interface_body_from_schema(schema: &Value) -> String {
    let Some(props) = schema.get("properties").and_then(|p| p.as_object()) else {
        return String::new();
    };
    let mut names: Vec<&String> = props.keys().collect();
    names.sort();
    let mut out = String::new();
    for name in names {
        let node = &props[name];
        let required = prop_is_required(schema, name);
        let optional = !required || prop_allows_null(node);
        let ts_ty = schema_to_ts_type(node);
        let opt = if optional { "?" } else { "" };
        out.push_str(&format!("  {name}{opt}: {ts_ty}\n"));
    }
    out
}

/// Ret body: use schema properties when present; else hand fallback / open index.
fn ts_ret_body_from_schema(schema: &Value, fallback: &str) -> String {
    if schema
        .get("properties")
        .and_then(|p| p.as_object())
        .is_some_and(|m| !m.is_empty())
    {
        return ts_interface_body_from_schema(schema);
    }
    // Flatten Json / empty object → open bag or explicit fallback
    if !fallback.trim().is_empty() {
        let mut s = fallback.to_string();
        if !s.ends_with('\n') {
            s.push('\n');
        }
        return s;
    }
    "  [key: string]: JsonValue | undefined\n".into()
}

/// Nested named types referenced from args (e.g. TaskActor) — emit once.
pub fn collect_named_defs(contracts: &[MethodContract]) -> Vec<(String, Value)> {
    let mut out = Vec::new();
    let mut seen = std::collections::BTreeSet::new();
    for c in contracts {
        let input = (c.input_schema)();
        if let Some(defs) = input.get("definitions").and_then(|d| d.as_object()) {
            for (name, schema) in defs {
                if seen.insert(name.clone()) {
                    out.push((name.clone(), schema.clone()));
                }
            }
        }
    }
    out
}

/// Top-level property names declared in an object schema (for TS parity).
pub fn schema_property_names(schema: &Value) -> Vec<String> {
    schema
        .get("properties")
        .and_then(|p| p.as_object())
        .map(|m| {
            let mut keys: Vec<String> = m.keys().cloned().collect();
            keys.sort();
            keys
        })
        .unwrap_or_default()
}

/// Parse `export interface Name { ... }` field names from generated TS.
pub fn ts_interface_field_names(ts: &str, iface: &str) -> Vec<String> {
    let needle = format!("export interface {iface} {{");
    let Some(start) = ts.find(&needle) else {
        return Vec::new();
    };
    let rest = &ts[start + needle.len()..];
    let Some(end) = rest.find('}') else {
        return Vec::new();
    };
    let body = &rest[..end];
    let mut fields = Vec::new();
    for line in body.lines() {
        let line = line.trim();
        if line.is_empty() || line.starts_with("//") || line.starts_with("/**") {
            continue;
        }
        // `field?: type` or `field: type` or index signature `[key: string]: …`
        if line.starts_with('[') {
            continue;
        }
        let name = line
            .split(':')
            .next()
            .unwrap_or("")
            .trim()
            .trim_end_matches('?');
        if !name.is_empty() && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_') {
            fields.push(name.to_string());
        }
    }
    fields.sort();
    fields
}

/// MCP `tools/list` descriptors generated from method contracts.
pub fn mcp_descriptors_from(contracts: &[MethodContract]) -> Value {
    let tools: Vec<Value> = contracts
        .iter()
        .map(|c| {
            json!({
                "name": c.mcp_tool,
                "description": c.description,
                "inputSchema": (c.input_schema)(),
            })
        })
        .collect();
    Value::Array(tools)
}

/// Full schema catalog served at `/sail.ops.v1.schema`.
pub fn schema_catalog(contracts: &[MethodContract]) -> Value {
    let methods: Vec<Value> = contracts
        .iter()
        .map(|c| {
            json!({
                "service": c.service,
                "method": c.method,
                "mcp_tool": c.mcp_tool,
                "description": c.description,
                "path": c.connect_path(),
                "inputSchema": (c.input_schema)(),
                "outputSchema": (c.output_schema)(),
            })
        })
        .collect();
    json!({
        "package": "sail.ops.v1",
        "methods": methods,
    })
}

/// CLI `--help`-style summary from contracts.
pub fn cli_help_text(contracts: &[MethodContract]) -> String {
    let mut out = String::from("sail-ops methods (Service/Method):\n");
    for c in contracts {
        out.push_str(&format!(
            "  {}/{}  ({})\n    {}\n",
            c.service, c.method, c.mcp_tool, c.description
        ));
    }
    out
}

/// Emit the committed TypeScript client source.
pub fn generate_ts_client(contracts: &[MethodContract]) -> String {
    let mut out = String::new();
    out.push_str("/**\n");
    out.push_str(" * GENERATED by `sail op gen` from sail-ops contracts — do not edit by hand.\n");
    out.push_str(" * Source: crates/sail/ops (MethodContract table).\n");
    out.push_str(" */\n\n");

    out.push_str("export type JsonValue =\n");
    out.push_str("  | string\n");
    out.push_str("  | number\n");
    out.push_str("  | boolean\n");
    out.push_str("  | null\n");
    out.push_str("  | JsonObject\n");
    out.push_str("  | JsonValue[]\n\n");
    out.push_str("/** Object-shaped JSON (Connect unary responses). */\n");
    out.push_str("export type JsonObject = { [key: string]: JsonValue | undefined }\n\n");

    out.push_str("export type WatchKind = 'model' | 'file' | 'log' | 'task'\n\n");

    out.push_str("export interface WatchEvent {\n");
    out.push_str("  kind: WatchKind\n");
    out.push_str("  path: string\n");
    out.push_str("  rev: number\n");
    out.push_str("  payload: JsonValue\n");
    out.push_str("}\n\n");

    out.push_str("export interface WatchFilter {\n");
    out.push_str("  kinds?: WatchKind[]\n");
    out.push_str("  pathPrefix?: string\n");
    out.push_str("}\n\n");

    out.push_str("export interface ClientOptions {\n");
    out.push_str("  /** Origin or absolute base (default: '') relative to page. */\n");
    out.push_str("  baseUrl?: string\n");
    out.push_str("  fetch?: typeof fetch\n");
    out.push_str("}\n\n");

    out.push_str("export interface HealthRet {\n");
    out.push_str("  ok: boolean\n");
    out.push_str("  package: string\n");
    out.push_str("  workspace_root: string\n");
    out.push_str("  project_root: string\n");
    out.push_str("  apply_root: string\n");
    out.push_str("  methods: string[]\n");
    out.push_str("  [key: string]: JsonValue | undefined\n");
    out.push_str("}\n\n");

    // Nested defs (TaskActor, …) from schemars definitions — before method ifaces.
    for (name, schema) in collect_named_defs(contracts) {
        if name == "Json" {
            continue;
        }
        let body = ts_interface_body_from_schema(&schema);
        out.push_str(&format!("export interface {name} {{\n"));
        out.push_str(&body);
        out.push_str("}\n\n");
    }

    // Per-method types — args fields derived from schemars inputSchema (T-I4).
    // Ret: prefer concrete schema properties; fall back to open JsonObject.
    for c in contracts {
        let input = (c.input_schema)();
        let args_body = ts_interface_body_from_schema(&input);
        out.push_str(&format!("export interface {} {{\n", c.ts_args));
        out.push_str(&args_body);
        out.push_str("}\n\n");

        let output = (c.output_schema)();
        let ret_body = ts_ret_body_from_schema(&output, c.ts_ret_body);
        out.push_str(&format!("export interface {} {{\n", c.ts_ret));
        out.push_str(&ret_body);
        out.push_str("}\n\n");
    }

    out.push_str("function resolveFetch(opts?: ClientOptions): typeof fetch {\n");
    out.push_str("  return opts?.fetch ?? globalThis.fetch.bind(globalThis)\n");
    out.push_str("}\n\n");

    out.push_str("function resolveBase(opts?: ClientOptions): string {\n");
    out.push_str("  const b = opts?.baseUrl ?? ''\n");
    out.push_str("  return b.endsWith('/') ? b.slice(0, -1) : b\n");
    out.push_str("}\n\n");

    out.push_str("async function postJson<T>(\n");
    out.push_str("  path: string,\n");
    out.push_str("  body: object,\n");
    out.push_str("  opts?: ClientOptions,\n");
    out.push_str("): Promise<T> {\n");
    out.push_str("  const f = resolveFetch(opts)\n");
    out.push_str("  const url = `${resolveBase(opts)}${path}`\n");
    out.push_str("  const res = await f(url, {\n");
    out.push_str("    method: 'POST',\n");
    out.push_str(
        "    headers: { 'content-type': 'application/json', accept: 'application/json' },\n",
    );
    out.push_str("    body: JSON.stringify(body ?? {}),\n");
    out.push_str("    cache: 'no-store',\n");
    out.push_str("  })\n");
    out.push_str("  const json = (await res.json().catch(() => ({}))) as {\n");
    out.push_str("    code?: string\n");
    out.push_str("    message?: string\n");
    out.push_str("  } & T\n");
    out.push_str("  if (!res.ok || (json as { code?: string }).code) {\n");
    out.push_str("    const msg =\n");
    out.push_str("      (typeof (json as { message?: string }).message === 'string' &&\n");
    out.push_str("        (json as { message?: string }).message) ||\n");
    out.push_str("      `Connect ${path} failed (${res.status})`\n");
    out.push_str("    throw new Error(msg)\n");
    out.push_str("  }\n");
    out.push_str("  return json as T\n");
    out.push_str("}\n\n");

    out.push_str("/** Typed unary call by service/method (generated transport). */\n");
    out.push_str("export async function callRpc(\n");
    out.push_str("  service: string,\n");
    out.push_str("  method: string,\n");
    out.push_str("  body: object = {},\n");
    out.push_str("  opts?: ClientOptions,\n");
    out.push_str("): Promise<JsonObject> {\n");
    out.push_str(
        "  return postJson<JsonObject>(`/sail.ops.v1.${service}/${method}`, body, opts)\n",
    );
    out.push_str("}\n\n");

    out.push_str(
        "/** Alias kept for session call-sites / tests (`ops(service, method, body)`). */\n",
    );
    out.push_str("export const ops = callRpc\n\n");

    out.push_str("/** Health is a bare GET endpoint (not unary Service/Method). */\n");
    out.push_str("export async function opsHealth(opts?: ClientOptions): Promise<HealthRet> {\n");
    out.push_str("  const f = resolveFetch(opts)\n");
    out.push_str(
        "  const res = await f(`${resolveBase(opts)}/sail.ops.v1.health`, { cache: 'no-store' })\n",
    );
    out.push_str("  const json = (await res.json().catch(() => ({}))) as HealthRet\n");
    out.push_str("  if (!res.ok) throw new Error(`health failed (${res.status})`)\n");
    out.push_str("  return json\n");
    out.push_str("}\n\n");

    for c in contracts {
        let fn_name = c.ts_fn_name();
        // Use // comments — descriptions may contain `*/` (e.g. path globs).
        out.push_str(&format!(
            "// {} — {}\n",
            c.rpc_name(),
            c.description.replace('\n', " ")
        ));
        out.push_str(&format!(
            "export async function {fn_name}(\n  args: {},\n  opts?: ClientOptions,\n): Promise<{}> {{\n",
            c.ts_args, c.ts_ret
        ));
        out.push_str(&format!(
            "  return postJson<{}>('{}', args ?? {{}}, opts)\n",
            c.ts_ret,
            c.connect_path()
        ));
        out.push_str("}\n\n");
    }

    // watch via SSE
    out.push_str("/**\n");
    out.push_str(" * Subscribe to ops watch channel (SSE `GET /sail.ops.v1.watch`).\n");
    out.push_str(" * Returns an unsubscribe function.\n");
    out.push_str(" */\n");
    out.push_str("export function watch(\n");
    out.push_str("  filter: WatchFilter,\n");
    out.push_str("  cb: (ev: WatchEvent) => void,\n");
    out.push_str("  opts?: ClientOptions,\n");
    out.push_str("): () => void {\n");
    out.push_str("  const params = new URLSearchParams()\n");
    out.push_str("  if (filter.kinds?.length) params.set('kinds', filter.kinds.join(','))\n");
    out.push_str("  if (filter.pathPrefix) params.set('pathPrefix', filter.pathPrefix)\n");
    out.push_str("  const q = params.toString()\n");
    out.push_str("  const url = `${resolveBase(opts)}/sail.ops.v1.watch${q ? `?${q}` : ''}`\n");
    out.push_str("  const es = new EventSource(url)\n");
    out.push_str("  const onMessage = (msg: MessageEvent) => {\n");
    out.push_str("    try {\n");
    out.push_str("      const ev = JSON.parse(String(msg.data)) as WatchEvent\n");
    out.push_str("      if (filter.kinds?.length && !filter.kinds.includes(ev.kind)) return\n");
    out.push_str("      if (filter.pathPrefix && !ev.path.startsWith(filter.pathPrefix)) return\n");
    out.push_str("      cb(ev)\n");
    out.push_str("    } catch {\n");
    out.push_str("      /* ignore malformed */\n");
    out.push_str("    }\n");
    out.push_str("  }\n");
    out.push_str("  es.addEventListener('message', onMessage)\n");
    out.push_str("  es.onmessage = onMessage\n");
    out.push_str("  return () => {\n");
    out.push_str("    es.removeEventListener('message', onMessage)\n");
    out.push_str("    es.close()\n");
    out.push_str("  }\n");
    out.push_str("}\n");

    out
}

/// Parse JSON args into a typed struct (contract edge).
pub fn parse_args<T: for<'de> Deserialize<'de>>(args: &Value) -> Result<T, crate::error::OpError> {
    serde_json::from_value(args.clone())
        .map_err(|e| crate::error::OpError::InvalidArgument(format!("args deserialize: {e}")))
}

/// Serialize a typed return to JSON Value.
pub fn to_value<T: Serialize>(ret: T) -> Result<Value, crate::error::OpError> {
    serde_json::to_value(ret)
        .map_err(|e| crate::error::OpError::Internal(format!("serialize: {e}")))
}

/// Helper used by snapshot tests: stable pretty schema for one method.
pub fn method_schema_snapshot(c: &MethodContract) -> String {
    let v = json!({
        "service": c.service,
        "method": c.method,
        "mcp_tool": c.mcp_tool,
        "inputSchema": (c.input_schema)(),
        "outputSchema": (c.output_schema)(),
    });
    serde_json::to_string_pretty(&v).unwrap_or_default()
}

/// Ensure an empty-object schema when T has no fields.
#[allow(dead_code)]
pub fn empty_object_schema() -> Schema {
    Schema::Object(SchemaObject {
        instance_type: Some(SingleOrVec::Single(Box::new(InstanceType::Object))),
        object: Some(Box::default()),
        ..Default::default()
    })
}
