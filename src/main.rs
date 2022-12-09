#![allow(non_snake_case, unused_braces)]

use anyhow::Context;
use anyhow::{anyhow, Result};
use axum::extract::rejection::JsonRejection;
use axum::extract::FromRef;
use axum::extract::State;
use axum::response::Redirect;
use axum::routing::{get, post};
use axum::Json;
use axum::{extract::ConnectInfo, extract::Path, response::Html, Router, Server};
use axum_extra::extract::cookie::{Cookie, Key, SameSite};
use axum_extra::extract::PrivateCookieJar;
use http::StatusCode;
use leptos::*;
use std::net::SocketAddr;
use std::sync::Arc;
use surrealdb::sql::Object;
use surrealdb::sql::Value;
use surrealdb::Response;
use surrealdb::{Datastore, Session};
use uuid::Uuid;

#[component]
fn HtmlC<'a>(
    cx: Scope,
    x_data: Option<&'a str>,
    children: Box<dyn Fn() -> Vec<Element>>,
) -> Element {
    view! { cx,
        <html class="">
            <head>
                <link rel="stylesheet" href="/static/styles.css"/>
                <script src="https://unpkg.com/alpinejs@3.x.x/dist/cdn.min.js" defer init />
            </head>
            <body
                x-data=x_data
                class="bg-gray-100 h-screen font-sans dark:bg-gray-700 text-black dark:text-white"
            >
            {children}
            </body>
        </html>
    }
}

#[component]
fn ValueInput<'a>(cx: Scope, name: &'a str, ty: &'a str, var: &'a str) -> Element {
    view! {cx,
        <div class="mb-4">
            <label for={name} class="block font-bold text-gray-700 mb-2">
                {name} ": "
            </label>
            <input type={ty} id={name} x-model={var} class="w-full py-2 px-3 bg-gray-200 rounded-lg
                border border-gray-300 focus:outline-none focus:border-indigo-500" required/>
        </div>
    }
}

#[component]
fn Home(cx: Scope) -> Element {
    view! {cx,
        <HtmlC x-data="{pass: '', email: '', api_res: 'A', login: 'Login'}">
        <div class="container mx-auto px-4 py-8">
            <h1 class="text-3xl font-bold text-gray-900">"Welcome to our site!"</h1>
            <div class="mt-4">
                <h2 class="text-xl font-bold text-gray-800">"Login"</h2>
                <p
                    x-transition
                    class="font-bold text-red-500 bg-red-500 bg-opacity-20
                        rounded-lg border border-red-500 text-xs mt-4 p-4 text-center invisible"
                    x-text="api_res"
                    x-ref="api"
                >"A"</p>
                <div class="mt-4">
                    <ValueInput name="Email" ty="email" var="email"/>
                    <ValueInput name="Password" ty="password" var="pass"/>
                    <div class="flex justify-end">
                        <button class="px-4 py-2 font-bold text-white bg-indigo-500 rounded-lg
                            hover:bg-indigo-700 focus:outline-none focus:shadow-outline-indigo 
                            active:bg-indigo-800" 
                            x-on:click="
                                login = ' ... ';
                                r_text = false;
                                fetch('/api/login', {
                                        method: 'POST',
                                        headers: {
                                            'Content-Type': 'application/json',
                                        },
                                        body: JSON.stringify({ pass: pass, email: email }),
                                    })
                                    .then(response => {
                                        if (response.redirected) {
                                            window.location.href = response.url;
                                        } else {
                                            r_text = true;
                                            return response.text();
                                        }
                                    })
                                    .then(data => { 
                                        login = 'Login';
                                        if (r_text === true) {
                                            api_res = data; 
                                            $refs.api.classList.remove('invisible');
                                        }
                                    })
                            "
                            x-text="login"
                        >
                            "Login"
                        </button>
                    </div>
                </div>
            </div>
        </div>
        </HtmlC>
    }
}

pub fn render(view: impl FnOnce(Scope) -> Element + 'static) -> String {
    "<!DOCTYPE html>".to_owned()
        + &render_to_string(view)
            .replace("<!--/-->", "")
            .replace("<!--#-->", "")
}

async fn home() -> Html<String> {
    Html(render(|cx| {
        view! {cx, <Home /> }
    }))
}

use serde::Deserialize;

#[derive(Deserialize)]
struct Data {
    email: String,
    pass: String,
}

async fn login(
    ConnectInfo(addr): ConnectInfo<SocketAddr>,
    State(state): State<AppState>,
    jar: PrivateCookieJar,
    result: Result<Json<Data>, JsonRejection>,
) -> Result<(PrivateCookieJar, Redirect), String> {
    let payload = if let Err(error) = result {
        return Err(format!("{}", error));
    } else {
        result.unwrap()
    };

    if payload.email.trim().is_empty() {
        return Err("Email must have a value".to_owned());
    }

    if payload.pass.trim().is_empty() {
        return Err("Password must have a value".to_owned());
    }

    let query_result = state
        .sql1_expect1(format!(
            "SELECT * FROM users WHERE email = '{}' AND pass = '{}'",
            payload.email, payload.pass
        ))
        .await;

    let response = if query_result.is_err() {
        let msg = query_result.unwrap_err().to_string();
        if msg == "Failed to get first Object of query response" {
            return Err("There is no email/password match".to_owned());
        } else {
            return Err("There was an error retreiving data from the db".to_owned());
        }
    } else {
        query_result.unwrap()
    };
    let uid = Uuid::new_v4();
    let query_result = state
        .sql1_expect1(format!(
            "CREATE sessions SET token = '{}', user = {}, ip = '{}'",
            uid,
            response.get("id").unwrap(),
            addr.ip().to_string()
        ))
        .await;

    let response = if query_result.is_err() {
        return Err("Failed to communicate with the database".to_string());
    } else {
        query_result.unwrap()
    };

    println!("{}", response.to_string());

    // We remove any existing 'tok' cookie
    let jar = {
        let cookie = jar.get("tok");
        if cookie.is_some() {
            jar.remove(cookie.unwrap())
        } else {
            jar
        }
    };

    Ok((
        jar.add({
            let mut a = Cookie::new("tok", uid.to_string());
            a.set_same_site(SameSite::Strict);
            a
        }),
        Redirect::permanent("/static/styles.css"),
    ))
}

/// Convert the Errors from ServeDir to a type that implements IntoResponse
async fn handle_file_error(err: std::io::Error) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("File Not Found: {}", err))
}

#[derive(Clone)]
struct AppState(Arc<InnerState>);

struct InnerState {
    db: Datastore,
    se: Session,
    ke: Key,
}

impl std::ops::Deref for AppState {
    type Target = InnerState;

    fn deref(&self) -> &Self::Target {
        &*self.0
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

        Ok(res
            .into_iter()
            .next()
            .context("Failed to get first Object of query response")?)
    }

    async fn sql1(&self, sql: impl Into<String>) -> Result<Vec<Object>> {
        let final_res = self.sql(sql).await?;
        if final_res.len() > 1 {
            return Err(anyhow!("The query returned more that one array"));
        }
        Ok(final_res
            .into_iter()
            .next()
            .context("Failed to get the first Array<Object> from query response")?)
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    let apps = AppState(Arc::new(InnerState {
        db: Datastore::new("memory").await?,
        se: Session::for_db("test", "test"),
        ke: Key::generate(),
    }));

    apps.sql(
        "
        CREATE users SET user = 'Daniel', age = 13, email = 'myemail@gmail.com', pass = '1234'; 
        CREATE users SET user = 'David', age = 16, email = 'msd@gmail.com', pass = '1234';
        ",
    )
    .await?;

    let static_service = {
        use axum::error_handling::HandleError;
        use tower_http::services::ServeDir;

        HandleError::new(ServeDir::new("./static"), handle_file_error)
    };

    let router = Router::new()
        .route("/api/login", post(login))
        .route("/", get(home))
        .nest_service(&format!("/static"), static_service)
        .with_state(apps);

    Ok(Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(router.into_make_service_with_connect_info::<SocketAddr>())
        .await?)
}
