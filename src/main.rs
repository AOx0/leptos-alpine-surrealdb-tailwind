#![allow(non_snake_case, unused_braces)]

use anyhow::Result;
use axum::routing::get;
use axum::{extract::Path, response::Html, Router, Server};
use axum_extra::extract::cookie::{Cookie, Key, SameSite};
use axum_extra::extract::PrivateCookieJar;
use http::StatusCode;
use leptos::*;
use surrealdb::{Datastore, Session};

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
                                login = 'Loading...';
                                fetch('/api/' + (email) +  '/' + (pass))
                                    .then(response => response.text())
                                    .then(data => { 
                                        api_res = data; 
                                        login = 'Login';
                                        $refs.api.classList.remove('invisible');
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

async fn login(
    Path((email, pass)): Path<(String, String)>,
    jar: PrivateCookieJar,
) -> Result<(PrivateCookieJar, String), StatusCode> {
    Ok((
        jar.add({
            let mut a = Cookie::new("email", email.to_owned());
            a.set_same_site(SameSite::Strict);
            a
        }),
        format!("Logged in email {email} with pass {pass}"),
    ))
}

/// Convert the Errors from ServeDir to a type that implements IntoResponse
async fn handle_file_error(err: std::io::Error) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("File Not Found: {}", err))
}

struct AppState {
    db: Datastore,
    se: Session,
}

#[tokio::main]
async fn main() -> Result<()> {
    let apps = AppState {
        db: Datastore::new("memory").await?,
        se: Session::for_db("test", "test"),
    };

    let static_service = {
        use axum::error_handling::HandleError;
        use tower_http::services::ServeDir;

        HandleError::new(ServeDir::new("./static"), handle_file_error)
    };

    let router = Router::new()
        .route("/api/:email/:mail", get(login))
        .route("/", get(home))
        .nest_service(&format!("/static"), static_service)
        .with_state(Key::generate());

    Ok(Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(router.into_make_service())
        .await?)
}
