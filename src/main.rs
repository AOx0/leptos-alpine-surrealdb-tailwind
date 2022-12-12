#![allow(non_snake_case, unused_braces)]

use anyhow::{anyhow, Context, Result};
use axum::extract::{ConnectInfo, FromRef, State};
use axum::middleware::Next;
use axum::response::{Html, IntoResponse, Redirect};
use axum::routing::{get, post};
use axum::{Json, RequestExt, Router, Server};
use axum_extra::extract::{
    cookie::{Cookie, Key, SameSite},
    PrivateCookieJar,
};
use http::{Request, StatusCode};
use surrealdb::{
    sql::{Object, Value},
    Datastore, Response, Session,
};

mod api;
mod routes;

/// Convert the Errors from ServeDir to a type that implements IntoResponse
async fn handle_file_error(err: std::io::Error) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("File Not Found: {}", err))
}

#[derive(Clone)]
pub struct AppState(std::sync::Arc<InnerState>);

pub struct InnerState {
    db: Datastore,
    se: Session,
    ke: Key,
}

impl std::ops::Deref for AppState {
    type Target = InnerState;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// this impl tells `SignedCookieJar` how to access the key from our state
impl FromRef<AppState> for Key {
    fn from_ref(state: &AppState) -> Self {
        state.0.ke.clone()
    }
}

impl AppState {
    async fn sql(&self, sql: impl Into<String>) -> Result<Vec<Vec<Object>>> {
        let res: Vec<Response> = self.db.execute(&sql.into(), &self.se, None, false).await?;
        let mut result: Vec<Value> = vec![];
        let mut final_res = vec![];

        if res.is_empty() {
            return Err(anyhow!("The query didn't return"));
        }

        for i in res.into_iter() {
            result.push(i.result?);
        }

        if result.is_empty() {
            return Err(anyhow!("The query didn't return"));
        }

        for i in result.into_iter() {
            if let surrealdb::sql::Value::Array(surrealdb::sql::Array(val)) = i {
                let mut res = vec![];
                for j in val.into_iter() {
                    if let surrealdb::sql::Value::Object(obj) = j {
                        res.push(obj);
                    } else {
                        return Err(anyhow!("Found non obj {}", j.to_string()));
                    }
                }
                final_res.push(res);
            } else {
                return Err(anyhow!("Found non Some<Array> {}", i.to_string()));
            }
        }

        Ok(final_res)
    }

    async fn sql1_expect1(&self, sql: impl Into<String>) -> Result<Object> {
        let res = self.sql1(sql).await?;

        if res.len() > 1 {
            return Err(anyhow!("Found more than one Object in query"));
        }

        res.into_iter()
            .next()
            .context("Failed to get first Object of query response")
    }

    async fn sql1(&self, sql: impl Into<String>) -> Result<Vec<Object>> {
        let final_res = self.sql(sql).await?;
        if final_res.len() > 1 {
            return Err(anyhow!("The query returned more that one array"));
        }
        final_res
            .into_iter()
            .next()
            .context("Failed to get the first Array<Object> from query response")
    }
}

async fn handle_auth<B>(
    State(state): State<AppState>,
    mut req: Request<B>,
    next: Next<B>,
) -> axum::response::Response
where
    B: Send + 'static,
{
    let path = req.uri().path();
    if path == "/" || path.contains("public") {
        return next.run(req).await;
    }

    let logged = async {
        let jar = req
            .extract_parts_with_state::<PrivateCookieJar, _>(&state)
            .await?;

        let tok = jar.get("tok").context("Did not find the token")?;

        // This call yields an error if the sql statement did not return
        // exactly one result. Hence, if this is an error no token was found
        state
            .sql1_expect1(format!(
                "SELECT * FROM sessions WHERE token = '{}'",
                tok.value()
            ))
            .await?;

        anyhow::Ok(())
    };

    if logged.await.is_err() {
        Redirect::temporary("/").into_response()
    } else {
        next.run(req).await
    }
}

async fn hello() -> &'static str {
    "Hello!"
}

#[tokio::main]
async fn main() -> Result<()> {
    let state = AppState(std::sync::Arc::new(InnerState {
        db: Datastore::new("memory").await?,
        se: Session::for_db("test", "test"),
        ke: Key::generate(),
    }));

    state.sql(
        "
        CREATE users SET user = 'Daniel', age = 13, email = 'myemail@gmail.com', pass = crypto::argon2::generate('1234'); 
        CREATE users SET user = 'David', age = 16, email = 'msd@gmail.com', pass = crypto::argon2::generate('1234');
        ",
    )
    .await?;

    let static_service = axum::error_handling::HandleError::new(
        tower_http::services::ServeDir::new("./static"),
        handle_file_error,
    );

    let router = Router::new()
        .route("/api/public/login", post(api::login::login))
        .route("/", get(routes::login::home))
        .route("/hello", get(hello))
        .nest_service("/public/static", static_service)
        .layer(axum::middleware::from_fn_with_state(
            state.clone(),
            handle_auth,
        ))
        .with_state(state);

    Ok(Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(router.into_make_service_with_connect_info::<std::net::SocketAddr>())
        .await?)
}
