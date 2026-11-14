use std::ffi::{CStr, CString};
use std::net::SocketAddr;
use std::os::raw::{c_char, c_int};
use std::ptr;

use axum::body::Body;
use axum::extract::{ConnectInfo, Query};
use axum::http::{header, HeaderValue, Request, StatusCode};
use axum::middleware::{self, Next};
use axum::response::{Html, IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use libsqlite3_sys as ffi;
use serde::Deserialize;
use serde_json::{json, Number, Value};

use crate::db;

const MAX_ROWS: usize = 500;

pub fn router() -> Router {
    Router::new()
        .route("/dev/db", get(index))
        .route("/dev/db/api/tables", get(tables))
        .route("/dev/db/api/table", get(table))
        .route("/dev/db/api/query", get(query))
        .layer(middleware::from_fn(loopback_only))
}

async fn loopback_only(request: Request<Body>, next: Next) -> Response {
    let allowed = request
        .extensions()
        .get::<ConnectInfo<SocketAddr>>()
        .is_some_and(|info| info.0.ip().is_loopback());
    if allowed {
        next.run(request).await
    } else {
        (
            StatusCode::FORBIDDEN,
            Json(json!({ "error": "the dev database browser only accepts loopback connections" })),
        )
            .into_response()
    }
}

async fn index() -> Response {
    let mut response = Html(PAGE).into_response();
    response
        .headers_mut()
        .insert(header::CACHE_CONTROL, HeaderValue::from_static("no-store"));
    response
}

async fn blocking<F>(work: F) -> Response
where
    F: FnOnce() -> Result<Value, String> + Send + 'static,
{
    match tokio::task::spawn_blocking(work).await {
        Ok(Ok(value)) => Json(value).into_response(),
        Ok(Err(message)) => {
            (StatusCode::BAD_REQUEST, Json(json!({ "error": message }))).into_response()
        }
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(json!({ "error": error.to_string() })),
        )
            .into_response(),
    }
}

async fn tables() -> Response {
    blocking(load_tables).await
}

async fn table(Query(params): Query<TableParams>) -> Response {
    blocking(move || load_table(params)).await
}

async fn query(Query(params): Query<QueryParams>) -> Response {
    blocking(move || load_query(params)).await
}

#[derive(Deserialize)]
struct TableParams {
    name: String,
    limit: Option<usize>,
    offset: Option<usize>,
}

#[derive(Deserialize)]
struct QueryParams {
    sql: String,
}

struct Database(*mut ffi::sqlite3);

impl Database {
    fn open_readonly(path: &str) -> Result<Self, String> {
        let path =
            CString::new(path).map_err(|_| "the database path contains a NUL byte".to_owned())?;
        let mut handle = ptr::null_mut();
        let code = unsafe {
            ffi::sqlite3_open_v2(
                path.as_ptr(),
                &mut handle,
                ffi::SQLITE_OPEN_READONLY,
                ptr::null(),
            )
        };
        let database = Self(handle);
        if code != ffi::SQLITE_OK {
            return Err(database.error(code));
        }
        unsafe {
            ffi::sqlite3_busy_timeout(handle, 5_000);
        }
        Ok(database)
    }

    fn error(&self, code: c_int) -> String {
        if self.0.is_null() {
            return format!("SQLite error {code}");
        }
        unsafe { CStr::from_ptr(ffi::sqlite3_errmsg(self.0)) }
            .to_string_lossy()
            .into_owned()
    }

    fn query(&self, sql: &str, max_rows: usize) -> Result<Value, String> {
        let sql = CString::new(sql).map_err(|_| "the query contains a NUL byte".to_owned())?;
        let mut statement = ptr::null_mut();
        let mut tail: *const c_char = ptr::null();
        let code =
            unsafe { ffi::sqlite3_prepare_v2(self.0, sql.as_ptr(), -1, &mut statement, &mut tail) };
        if code != ffi::SQLITE_OK {
            return Err(self.error(code));
        }
        let statement = Statement(statement);
        if statement.0.is_null() {
            return Err("the query is empty".to_owned());
        }
        if unsafe { ffi::sqlite3_stmt_readonly(statement.0) } == 0 {
            return Err("only read-only queries are allowed".to_owned());
        }
        if !tail.is_null() {
            let remainder = unsafe { CStr::from_ptr(tail) }.to_string_lossy();
            if !remainder.trim().trim_matches(';').trim().is_empty() {
                return Err("only one statement can be run at a time".to_owned());
            }
        }

        let column_count = unsafe { ffi::sqlite3_column_count(statement.0) }.max(0) as usize;
        let columns = (0..column_count)
            .map(|column| {
                let name = unsafe { ffi::sqlite3_column_name(statement.0, column as c_int) };
                if name.is_null() {
                    String::new()
                } else {
                    unsafe { CStr::from_ptr(name) }
                        .to_string_lossy()
                        .into_owned()
                }
            })
            .collect::<Vec<_>>();

        let mut rows = Vec::new();
        let mut truncated = false;
        loop {
            let code = unsafe { ffi::sqlite3_step(statement.0) };
            if code == ffi::SQLITE_ROW {
                if rows.len() >= max_rows {
                    truncated = true;
                    break;
                }
                let row = (0..column_count)
                    .map(|column| cell(statement.0, column as c_int))
                    .collect::<Vec<_>>();
                rows.push(Value::Array(row));
            } else if code == ffi::SQLITE_DONE {
                break;
            } else {
                return Err(self.error(code));
            }
        }

        Ok(json!({
            "columns": columns,
            "rows": rows,
            "row_count": rows.len(),
            "truncated": truncated,
        }))
    }
}

impl Drop for Database {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                ffi::sqlite3_close(self.0);
            }
        }
    }
}

struct Statement(*mut ffi::sqlite3_stmt);

impl Drop for Statement {
    fn drop(&mut self) {
        if !self.0.is_null() {
            unsafe {
                ffi::sqlite3_finalize(self.0);
            }
        }
    }
}

fn cell(statement: *mut ffi::sqlite3_stmt, column: c_int) -> Value {
    match unsafe { ffi::sqlite3_column_type(statement, column) } {
        ffi::SQLITE_NULL => Value::Null,
        ffi::SQLITE_INTEGER => Value::Number(Number::from(unsafe {
            ffi::sqlite3_column_int64(statement, column)
        })),
        ffi::SQLITE_FLOAT => {
            let value = unsafe { ffi::sqlite3_column_double(statement, column) };
            Number::from_f64(value)
                .map(Value::Number)
                .unwrap_or_else(|| Value::String(value.to_string()))
        }
        ffi::SQLITE_TEXT => {
            let pointer = unsafe { ffi::sqlite3_column_text(statement, column) };
            let length = unsafe { ffi::sqlite3_column_bytes(statement, column) };
            if pointer.is_null() || length <= 0 {
                Value::String(String::new())
            } else {
                let bytes = unsafe { std::slice::from_raw_parts(pointer, length as usize) };
                Value::String(String::from_utf8_lossy(bytes).into_owned())
            }
        }
        ffi::SQLITE_BLOB => {
            let pointer = unsafe { ffi::sqlite3_column_blob(statement, column) };
            let length = unsafe { ffi::sqlite3_column_bytes(statement, column) }.max(0) as usize;
            if pointer.is_null() || length == 0 {
                Value::String("blob (0 bytes)".to_owned())
            } else {
                let bytes = unsafe { std::slice::from_raw_parts(pointer.cast::<u8>(), length) };
                let preview = bytes
                    .iter()
                    .take(32)
                    .map(|byte| format!("{byte:02x}"))
                    .collect::<String>();
                let ellipsis = if length > 32 { "…" } else { "" };
                Value::String(format!("blob 0x{preview}{ellipsis} ({length} bytes)"))
            }
        }
        _ => Value::Null,
    }
}

fn load_tables() -> Result<Value, String> {
    let database = Database::open_readonly(&db::database_path())?;
    let result = database.query(
        "SELECT type, name, sql FROM sqlite_master \
         WHERE type IN ('table', 'view') AND name NOT LIKE 'sqlite_%' \
         ORDER BY name",
        MAX_ROWS,
    )?;
    let rows = result
        .get("rows")
        .and_then(Value::as_array)
        .cloned()
        .unwrap_or_default();
    let tables = rows
        .into_iter()
        .filter_map(|row| {
            let row = row.as_array()?;
            Some(json!({
                "type": row.first().cloned().unwrap_or(Value::Null),
                "name": row.get(1).cloned().unwrap_or(Value::Null),
                "sql": row.get(2).cloned().unwrap_or(Value::Null),
            }))
        })
        .collect::<Vec<_>>();
    let path = std::fs::canonicalize(db::database_path())
        .map(|path| path.display().to_string())
        .unwrap_or_else(|_| db::database_path());
    Ok(json!({ "path": path, "tables": tables }))
}

fn load_table(params: TableParams) -> Result<Value, String> {
    if params.name.is_empty() {
        return Err("the table name is required".to_owned());
    }
    let limit = params.limit.unwrap_or(100).clamp(1, MAX_ROWS);
    let offset = params.offset.unwrap_or(0).min(1_000_000);
    let quoted = quote_identifier(&params.name);
    let database = Database::open_readonly(&db::database_path())?;

    let info = database.query(&format!("PRAGMA table_info({quoted})"), MAX_ROWS)?;
    let columns = info
        .get("rows")
        .and_then(Value::as_array)
        .map(|rows| {
            rows.iter()
                .filter_map(|row| {
                    let row = row.as_array()?;
                    Some(json!({
                        "name": row.get(1).cloned().unwrap_or(Value::Null),
                        "type": row.get(2).cloned().unwrap_or(Value::Null),
                        "not_null": row.get(3).and_then(Value::as_i64).unwrap_or(0) != 0,
                        "default": row.get(4).cloned().unwrap_or(Value::Null),
                        "primary_key": row.get(5).cloned().unwrap_or(Value::Null),
                    }))
                })
                .collect::<Vec<_>>()
        })
        .unwrap_or_default();

    let count = database.query(&format!("SELECT COUNT(*) AS total FROM {quoted}"), 1)?;
    let total = count
        .get("rows")
        .and_then(Value::as_array)
        .and_then(|rows| rows.first())
        .and_then(Value::as_array)
        .and_then(|row| row.first())
        .and_then(Value::as_i64)
        .unwrap_or(0);

    let data = database.query(
        &format!("SELECT * FROM {quoted} LIMIT {limit} OFFSET {offset}"),
        limit,
    )?;

    let schema = database
        .query(
            &format!(
                "SELECT sql FROM sqlite_master \
                 WHERE type IN ('table', 'view') AND name = {}",
                quote_literal(&params.name)
            ),
            1,
        )
        .ok()
        .and_then(|result| {
            result
                .get("rows")
                .and_then(Value::as_array)
                .and_then(|rows| rows.first())
                .and_then(Value::as_array)
                .and_then(|row| row.first())
                .and_then(Value::as_str)
                .map(str::to_owned)
        });

    let mut body = data.as_object().cloned().unwrap_or_default();
    body.insert("name".to_owned(), json!(params.name));
    body.insert("columns".to_owned(), json!(columns));
    body.insert("total".to_owned(), json!(total));
    body.insert("limit".to_owned(), json!(limit));
    body.insert("offset".to_owned(), json!(offset));
    body.insert("sql".to_owned(), json!(schema));
    Ok(Value::Object(body))
}

fn load_query(params: QueryParams) -> Result<Value, String> {
    if params.sql.trim().is_empty() {
        return Err("the query is empty".to_owned());
    }
    if params.sql.len() > 100_000 {
        return Err("the query is too long".to_owned());
    }
    let started = std::time::Instant::now();
    let database = Database::open_readonly(&db::database_path())?;
    let mut result = database.query(&params.sql, MAX_ROWS)?;
    if let Some(object) = result.as_object_mut() {
        object.insert(
            "elapsed_ms".to_owned(),
            json!(started.elapsed().as_millis() as u64),
        );
    }
    Ok(result)
}

fn quote_identifier(name: &str) -> String {
    format!("\"{}\"", name.replace('"', "\"\""))
}

fn quote_literal(value: &str) -> String {
    format!("'{}'", value.replace('\'', "''"))
}

const PAGE: &str = r##"<!doctype html>
<html lang="en">
<head>
<meta charset="utf-8">
<meta name="viewport" content="width=device-width, initial-scale=1">
<title>persona-sanus · SQLite</title>
<style>
  :root {
    color-scheme: light dark;
    --bg: #f6f6f4;
    --surface: #ffffff;
    --surface-2: #efefec;
    --fg: #1c1c1a;
    --muted: #6b6b66;
    --border: #d7d7d2;
    --accent: #2f6f4e;
    --accent-fg: #ffffff;
    --error: #a12a2a;
    --error-bg: #fbe9e9;
    --mono: ui-monospace, SFMono-Regular, Menlo, Consolas, monospace;
  }
  @media (prefers-color-scheme: dark) {
    :root {
      --bg: #171816;
      --surface: #1f211e;
      --surface-2: #292b27;
      --fg: #e9e9e4;
      --muted: #a2a39c;
      --border: #3a3d37;
      --accent: #8fc7a8;
      --accent-fg: #14251b;
      --error: #ff9b9b;
      --error-bg: #3a2020;
    }
  }
  *, *::before, *::after { box-sizing: border-box; }
  html, body { height: 100%; }
  body {
    margin: 0;
    background: var(--bg);
    color: var(--fg);
    font: 14px/1.45 system-ui, -apple-system, BlinkMacSystemFont, "Segoe UI", sans-serif;
  }
  header {
    display: flex;
    align-items: center;
    gap: 12px;
    padding: 10px 16px;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
    position: sticky;
    top: 0;
    z-index: 5;
  }
  .brand { font-weight: 700; letter-spacing: -.01em; white-space: nowrap; }
  .brand span { color: var(--accent); }
  .path {
    flex: 1;
    min-width: 0;
    overflow: hidden;
    text-overflow: ellipsis;
    white-space: nowrap;
    font-family: var(--mono);
    font-size: 12px;
    color: var(--muted);
  }
  .badge {
    font: 600 10px/1 var(--mono);
    letter-spacing: .08em;
    padding: 5px 6px;
    border-radius: 999px;
    border: 1px solid var(--border);
    color: var(--muted);
    white-space: nowrap;
  }
  .badge.dev { color: var(--accent); border-color: var(--accent); }
  .shell { display: flex; min-height: calc(100vh - 45px); }
  aside {
    width: 240px;
    flex: 0 0 240px;
    border-right: 1px solid var(--border);
    background: var(--surface);
    padding: 12px 8px;
    overflow: auto;
  }
  .side-head {
    display: flex;
    align-items: center;
    justify-content: space-between;
    padding: 0 8px 8px;
    font: 600 11px/1 var(--mono);
    letter-spacing: .08em;
    text-transform: uppercase;
    color: var(--muted);
  }
  .table-list { list-style: none; margin: 0; padding: 0; }
  .table-list button {
    display: flex;
    align-items: center;
    gap: 8px;
    width: 100%;
    border: 0;
    background: transparent;
    color: inherit;
    font: 13px/1.2 var(--mono);
    text-align: left;
    padding: 7px 8px;
    border-radius: 6px;
    cursor: pointer;
  }
  .table-list button:hover { background: var(--surface-2); }
  .table-list button.active { background: var(--surface-2); color: var(--accent); font-weight: 600; }
  .table-list .kind { color: var(--muted); font-size: 10px; text-transform: uppercase; }
  .side-note { padding: 16px 8px; color: var(--muted); font-size: 11px; }
  main { flex: 1; min-width: 0; display: flex; flex-direction: column; }
  .tabs {
    display: flex;
    gap: 4px;
    padding: 10px 16px 0;
    border-bottom: 1px solid var(--border);
    background: var(--surface);
  }
  .tab {
    appearance: none;
    border: 0;
    border-bottom: 2px solid transparent;
    background: transparent;
    color: var(--muted);
    font: 600 12px/1 var(--mono);
    letter-spacing: .04em;
    padding: 9px 10px;
    cursor: pointer;
  }
  .tab.active { color: var(--fg); border-color: var(--accent); }
  .pane { padding: 16px; }
  .toolbar {
    display: flex;
    align-items: baseline;
    gap: 12px;
    flex-wrap: wrap;
    margin-bottom: 12px;
  }
  h2 { font-size: 17px; margin: 0; }
  .meta { color: var(--muted); font-size: 12px; }
  .pager {
    display: flex;
    align-items: center;
    gap: 8px;
    margin-bottom: 10px;
    flex-wrap: wrap;
  }
  .pager .spacer { flex: 1; }
  button {
    appearance: none;
    border: 1px solid var(--border);
    background: var(--surface);
    color: var(--fg);
    border-radius: 6px;
    padding: 5px 10px;
    font: 500 12px/1.2 system-ui, sans-serif;
    cursor: pointer;
  }
  button:hover:not(:disabled) { border-color: var(--accent); color: var(--accent); }
  button:disabled { opacity: .45; cursor: default; }
  button.primary {
    background: var(--accent);
    border-color: var(--accent);
    color: var(--accent-fg);
    font-weight: 600;
  }
  button.primary:hover:not(:disabled) { color: var(--accent-fg); filter: brightness(1.08); }
  button.icon { padding: 4px 7px; }
  select, textarea {
    font: 12px/1.4 var(--mono);
    color: var(--fg);
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 6px;
    padding: 5px 7px;
  }
  .grid {
    overflow: auto;
    border: 1px solid var(--border);
    border-radius: 8px;
    background: var(--surface);
    max-height: calc(100vh - 240px);
  }
  table { border-collapse: collapse; width: 100%; }
  th, td {
    border-bottom: 1px solid var(--border);
    border-right: 1px solid var(--border);
    padding: 6px 9px;
    text-align: left;
    vertical-align: top;
    font: 12px/1.4 var(--mono);
    max-width: 420px;
    white-space: pre-wrap;
    word-break: break-word;
  }
  th {
    position: sticky;
    top: 0;
    z-index: 1;
    background: var(--surface-2);
    font-weight: 600;
    white-space: nowrap;
  }
  th .type { display: block; font-weight: 400; font-size: 9px; letter-spacing: .04em; color: var(--muted); text-transform: uppercase; }
  th:last-child, td:last-child { border-right: 0; }
  tr:last-child td { border-bottom: 0; }
  td.number { text-align: right; font-variant-numeric: tabular-nums; }
  td.null { color: var(--muted); font-style: italic; }
  td.row-number { width: 1%; text-align: right; color: var(--muted); user-select: none; }
  .empty { padding: 28px 16px; text-align: center; color: var(--muted); font-size: 13px; }
  .error {
    padding: 10px 12px;
    border: 1px solid var(--error);
    border-radius: 8px;
    background: var(--error-bg);
    color: var(--error);
    font: 12px/1.5 var(--mono);
    white-space: pre-wrap;
  }
  .schema { margin-top: 12px; color: var(--muted); font-size: 12px; }
  .schema summary { cursor: pointer; user-select: none; }
  .schema pre {
    margin: 8px 0 0;
    padding: 12px;
    overflow: auto;
    background: var(--surface);
    border: 1px solid var(--border);
    border-radius: 8px;
    color: var(--fg);
    font: 12px/1.5 var(--mono);
  }
  #sqlPane { display: flex; flex-direction: column; min-height: 0; flex: 1; padding: 16px; }
  #sqlPane .grid { flex: 1; min-height: 140px; max-height: none; margin-top: 10px; }
  #sqlPane textarea {
    width: 100%;
    min-height: 160px;
    resize: vertical;
    font-size: 13px;
    line-height: 1.5;
  }
  .sql-actions { display: flex; align-items: center; gap: 12px; margin-top: 10px; }
  .hint { color: var(--muted); font-size: 11px; }
  [hidden] { display: none !important; }
  @media (max-width: 720px) {
    aside { width: 156px; flex-basis: 156px; }
  }
</style>
</head>
<body>
<header>
  <div class="brand">persona<span>·</span>sanus</div>
  <div class="path" id="path">…</div>
  <span class="badge dev">DEV</span>
  <span class="badge">READ ONLY</span>
</header>
<div class="shell">
  <aside>
    <div class="side-head"><span>Tables</span><button class="icon" id="refresh" title="Reload tables">↻</button></div>
    <ul class="table-list" id="tables"></ul>
    <div class="side-note">Mounted only in debug builds. SQLite cannot write through this route.</div>
  </aside>
  <main>
    <nav class="tabs">
      <button class="tab active" data-tab="data">Data</button>
      <button class="tab" data-tab="sql">SQL</button>
    </nav>
    <section class="pane" id="dataPane">
      <div class="toolbar">
        <h2 id="tableTitle">Loading…</h2>
        <div class="meta" id="tableMeta"></div>
      </div>
      <div class="pager">
        <button id="prev" disabled>‹ Prev</button>
        <button id="next" disabled>Next ›</button>
        <span class="meta" id="range"></span>
        <span class="spacer"></span>
         <label class="meta">Rows <select id="limit">
          <option>25</option><option selected>100</option><option>250</option><option>500</option>
        </select></label>
        <button id="queryTable" disabled>Open in SQL</button>
      </div>
      <div class="grid" id="grid"><div class="empty">Choose a table on the left.</div></div>
      <details class="schema" id="schemaBox" hidden>
        <summary>Schema</summary>
        <pre id="schema"></pre>
      </details>
    </section>
    <section class="pane" id="sqlPane" hidden>
      <div class="toolbar">
        <h2>SQL</h2>
        <div class="meta">Read-only connection · ⌘/Ctrl + Enter runs</div>
      </div>
      <textarea id="sql" spellcheck="false" autocapitalize="off" autocomplete="off" placeholder="SELECT * FROM users LIMIT 20;"></textarea>
      <div class="sql-actions">
        <button class="primary" id="run">Run query</button>
        <span class="meta" id="sqlStatus"></span>
      </div>
      <div class="grid" id="sqlGrid"><div class="empty">Results appear here.</div></div>
    </section>
  </main>
</div>
<script>
  const state = { tables: [], name: null, limit: 100, offset: 0, total: 0 };

  const byId = (id) => document.getElementById(id);
  const pathEl = byId("path");
  const tablesEl = byId("tables");
  const titleEl = byId("tableTitle");
  const metaEl = byId("tableMeta");
  const gridEl = byId("grid");
  const schemaBoxEl = byId("schemaBox");
  const schemaEl = byId("schema");
  const prevEl = byId("prev");
  const nextEl = byId("next");
  const rangeEl = byId("range");
  const limitEl = byId("limit");
  const queryTableEl = byId("queryTable");
  const refreshEl = byId("refresh");
  const dataPane = byId("dataPane");
  const sqlPane = byId("sqlPane");
  const sqlEl = byId("sql");
  const runEl = byId("run");
  const sqlStatusEl = byId("sqlStatus");
  const sqlGridEl = byId("sqlGrid");

  function element(tag, className, text) {
    const node = document.createElement(tag);
    if (className) node.className = className;
    if (text !== undefined) node.textContent = text;
    return node;
  }

  async function api(path) {
    const response = await fetch(path, { headers: { Accept: "application/json" } });
    const text = await response.text();
    let data = null;
    try {
      data = text ? JSON.parse(text) : null;
    } catch {
      throw new Error(text || "HTTP " + response.status);
    }
    if (!response.ok) {
      throw new Error((data && data.error) || "HTTP " + response.status);
    }
    if (data && data.error) {
      throw new Error(data.error);
    }
    return data;
  }

  function renderError(container, message) {
    container.replaceChildren();
    container.append(element("div", "error", message));
  }

  function renderGrid(container, result) {
    container.replaceChildren();
    const allColumns = (result.columns || []).map((column) =>
      typeof column === "string"
        ? { name: column, type: "" }
        : { name: column.name, type: column.type || "" }
    );
    if (allColumns.length === 0 || !result.rows || result.rows.length === 0) {
      container.append(element("div", "empty", "No rows."));
      return;
    }
    const table = element("table");
    const head = document.createElement("thead");
    const headRow = document.createElement("tr");
    headRow.append(element("th", "row-number", "#"));
    allColumns.forEach((column) => {
      const th = element("th", "", column.name);
      if (column.type) {
        th.append(element("span", "type", column.type));
      }
      headRow.append(th);
    });
    head.append(headRow);
    table.append(head);
    const body = document.createElement("tbody");
    result.rows.forEach((row, index) => {
      const tr = document.createElement("tr");
      const number = element("td", "row-number", String(index + 1));
      tr.append(number);
      row.forEach((value, columnIndex) => {
        const td = document.createElement("td");
        const column = allColumns[columnIndex] || { name: "" };
        if (value === null) {
          td.className = "null";
          td.textContent = "NULL";
        } else if (typeof value === "number") {
          td.className = "number";
          td.textContent = String(value);
        } else if (typeof value === "object") {
          td.textContent = JSON.stringify(value);
        } else {
          td.textContent = String(value);
        }
        tr.append(td);
      });
      body.append(tr);
    });
    table.append(body);
    container.append(table);
    if (result.truncated) {
      container.append(element("div", "empty", "Showing the first " + result.row_count + " rows. Add LIMIT or narrow the query."));
    }
  }

  function markActive() {
    tablesEl.querySelectorAll("button").forEach((button) => {
      button.classList.toggle("active", button.dataset.name === state.name);
    });
  }

  async function loadTables() {
    try {
      const data = await api("/dev/db/api/tables");
      state.tables = data.tables || [];
      pathEl.textContent = data.path;
      pathEl.title = data.path;
      tablesEl.replaceChildren();
      state.tables.forEach((table) => {
        const item = document.createElement("li");
        const button = element("button");
        button.type = "button";
        button.dataset.name = table.name;
        button.append(element("span", "", table.name));
        button.append(element("span", "kind", table.type === "view" ? "view" : "table"));
        button.addEventListener("click", () => openTable(table.name));
        item.append(button);
        tablesEl.append(item);
      });
      if (state.name && state.tables.some((table) => table.name === state.name)) {
        markActive();
      } else if (state.tables.length > 0) {
        const preferred = state.tables.find((table) => table.name === "users") || state.tables[0];
        openTable(preferred.name);
      } else {
        titleEl.textContent = "No tables";
      }
    } catch (error) {
      renderError(tablesEl, error.message);
    }
  }

  async function openTable(name) {
    state.name = name;
    state.offset = 0;
    state.limit = Number(limitEl.value) || 100;
    titleEl.textContent = name;
    queryTableEl.disabled = false;
    markActive();
    await loadTable();
  }

  async function loadTable() {
    if (!state.name) return;
    metaEl.textContent = "Loading…";
    try {
      const data = await api(
        "/dev/db/api/table?name=" + encodeURIComponent(state.name) +
        "&limit=" + state.limit + "&offset=" + state.offset
      );
      state.total = data.total;
      metaEl.textContent = data.total + (data.total === 1 ? " row" : " rows") +
        " · " + data.columns.length + (data.columns.length === 1 ? " column" : " columns");
      renderGrid(gridEl, data);
      const shown = data.rows.length;
      rangeEl.textContent = shown === 0
        ? "0–0 of " + data.total
        : (state.offset + 1) + "–" + (state.offset + shown) + " of " + data.total;
      prevEl.disabled = state.offset <= 0;
      nextEl.disabled = state.offset + shown >= data.total;
      schemaBoxEl.hidden = !data.sql;
      schemaEl.textContent = data.sql || "";
    } catch (error) {
      metaEl.textContent = "";
      rangeEl.textContent = "";
      prevEl.disabled = true;
      nextEl.disabled = true;
      renderError(gridEl, error.message);
    }
  }

  async function runSql() {
    const sql = sqlEl.value.trim();
    if (!sql) {
      sqlEl.focus();
      return;
    }
    runEl.disabled = true;
    sqlStatusEl.textContent = "Running…";
    try {
      const data = await api("/dev/db/api/query?sql=" + encodeURIComponent(sql));
      sqlStatusEl.textContent = data.row_count + (data.row_count === 1 ? " row" : " rows") +
        " · " + data.elapsed_ms + " ms" + (data.truncated ? " · truncated" : "");
      renderGrid(sqlGridEl, data);
    } catch (error) {
      sqlStatusEl.textContent = "";
      renderError(sqlGridEl, error.message);
    } finally {
      runEl.disabled = false;
    }
  }

  function showTab(name) {
    document.querySelectorAll(".tab").forEach((tab) => {
      tab.classList.toggle("active", tab.dataset.tab === name);
    });
    dataPane.hidden = name !== "data";
    sqlPane.hidden = name !== "sql";
  }

  prevEl.addEventListener("click", () => {
    state.offset = Math.max(0, state.offset - state.limit);
    loadTable();
  });
  nextEl.addEventListener("click", () => {
    state.offset += state.limit;
    loadTable();
  });
  limitEl.addEventListener("change", () => {
    state.limit = Number(limitEl.value);
    state.offset = 0;
    loadTable();
  });
  refreshEl.addEventListener("click", loadTables);
  runEl.addEventListener("click", runSql);
  queryTableEl.addEventListener("click", () => {
    if (!state.name) return;
    const quoted = '"' + state.name.replaceAll('"', '""') + '"';
    sqlEl.value = "SELECT * FROM " + quoted + " LIMIT 100;";
    showTab("sql");
    sqlEl.focus();
  });
  document.querySelectorAll(".tab").forEach((tab) => {
    tab.addEventListener("click", () => showTab(tab.dataset.tab));
  });
  sqlEl.addEventListener("keydown", (event) => {
    if ((event.metaKey || event.ctrlKey) && event.key === "Enter") {
      event.preventDefault();
      runSql();
    }
  });
  loadTables();
</script>
</body>
</html>
"##;
