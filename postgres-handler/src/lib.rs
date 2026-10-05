mod bindings {
    wit_bindgen::generate!({
        generate_all,
    });
}

use bindings::wasmcloud::postgres::query::query;
use bindings::wasmcloud::postgres::types::{PgValue, ResultRow};
use serde_json::{Map, Value};
use wstd::http::{Body, Method, Request, Response, StatusCode};

const SELECT_USERS: &str = "SELECT id, name, email FROM users ORDER BY id";

#[wstd::http_server]
async fn main(request: Request<Body>) -> Result<Response<Body>, wstd::http::Error> {
    match request.method() {
        &Method::GET => handle_get(),
        _ => response_factory("Method Not Allowed\n", StatusCode::METHOD_NOT_ALLOWED),
    }
}

fn handle_get() -> Result<Response<Body>, wstd::http::Error> {
    match query(SELECT_USERS, &[]) {
        Ok(rows) => {
            let users: Vec<Value> = rows.into_iter().map(row_to_json).collect();
            let body = serde_json::to_string(&users)?;
            Response::builder()
                .status(StatusCode::OK)
                .header("content-type", "application/json")
                .body(body.into())
                .map_err(Into::into)
        }
        Err(e) => response_factory(
            format!("Query failed: {e:?}\n").as_str(),
            StatusCode::INTERNAL_SERVER_ERROR,
        ),
    }
}

fn row_to_json(row: ResultRow) -> Value {
    let columns: Map<String, Value> = row
        .into_iter()
        .map(|entry| (entry.column_name, pg_value_to_json(entry.value)))
        .collect();
    Value::Object(columns)
}

fn pg_value_to_json(value: PgValue) -> Value {
    match value {
        PgValue::Null => Value::Null,
        PgValue::Serial(n) | PgValue::Serial4(n) => Value::from(n),
        PgValue::Integer(n) | PgValue::Int(n) | PgValue::Int4(n) => Value::from(n),
        PgValue::BigInt(n) | PgValue::Int8(n) => Value::from(n),
        PgValue::Text(s) | PgValue::Name(s) => Value::String(s),
        PgValue::Varchar((_, bytes)) => Value::String(String::from_utf8_lossy(&bytes).into_owned()),
        other => Value::String(format!("{other:?}")),
    }
}

fn response_factory(message: &str, code: StatusCode) -> Result<Response<Body>, wstd::http::Error> {
    Response::builder()
        .status(code)
        .body(message.into())
        .map_err(Into::into)
}
