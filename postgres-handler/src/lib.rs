mod bindings {
    wit_bindgen::generate!({
        generate_all,
    });
}

use bindings::wasmcloud::postgres::query::query;
use bindings::wasmcloud::postgres::types::{PgValue, ResultRow};
use serde::Deserialize;
use serde_json::{Map, Value};
use wstd::http::{Body, Method, Request, Response, StatusCode};

const SELECT_USERS: &str = "SELECT id, name, email FROM users ORDER BY id";
const INSERT_USER: &str =
    "INSERT INTO users (name, email) VALUES ($1, $2) RETURNING id, name, email";
const MAX_FIELD_LEN: usize = 100;

#[derive(Deserialize)]
struct NewUser {
    name: String,
    email: String,
}

#[wstd::http_server]
async fn main(request: Request<Body>) -> Result<Response<Body>, wstd::http::Error> {
    match *request.method() {
        Method::GET => handle_get(),
        Method::POST => handle_post(request.into_body()).await,
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

async fn handle_post(mut body: Body) -> Result<Response<Body>, wstd::http::Error> {
    let Ok(user) = body.json::<NewUser>().await else {
        return response_factory(
            "Expected JSON body: {\"name\": string, \"email\": string}\n",
            StatusCode::BAD_REQUEST,
        );
    };
    let name = user.name.trim();
    let email = user.email.trim();
    if name.is_empty()
        || email.is_empty()
        || name.chars().count() > MAX_FIELD_LEN
        || email.chars().count() > MAX_FIELD_LEN
    {
        return response_factory(
            "name and email must be non-empty and at most 100 characters\n",
            StatusCode::BAD_REQUEST,
        );
    }

    let params = [
        PgValue::Text(name.to_owned()),
        PgValue::Text(email.to_owned()),
    ];
    match query(INSERT_USER, &params) {
        Ok(rows) => {
            let Some(row) = rows.into_iter().next() else {
                return response_factory(
                    "Insert returned no row\n",
                    StatusCode::INTERNAL_SERVER_ERROR,
                );
            };
            let body = serde_json::to_string(&row_to_json(row))?;
            Response::builder()
                .status(StatusCode::CREATED)
                .header("content-type", "application/json")
                .body(body.into())
                .map_err(Into::into)
        }
        Err(e) => {
            let message = format!("{e:?}");
            if message.contains("23505") || message.contains("duplicate key") {
                response_factory(
                    "A user with that email already exists\n",
                    StatusCode::CONFLICT,
                )
            } else {
                response_factory(
                    format!("Insert failed: {message}\n").as_str(),
                    StatusCode::INTERNAL_SERVER_ERROR,
                )
            }
        }
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
