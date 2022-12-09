#![allow(non_snake_case, unused_braces)]

use anyhow::Result;
use axum::routing::get;
use axum::{extract::Path, response::Html, Router, Server};
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
            <body x-data=x_data class="bg-gray-100 h-screen font-sans dark:bg-gray-700 text-black dark:text-white">
            {children}
            </body>
        </html>
    }
}

#[component]
fn Hello(cx: Scope, name: String) -> Element {
    view! {cx,
        <HtmlC x-data="{value: '', count: 0, place: 'Type here...' }">
            <div class="flex flex-col items-center">
                <h1 class="text-xl">"Hola "{&name}</h1>
                <div>
                    <input type="text" x-model="value" x-bind:placeholder="place" />
                </div>
                <div class="flex">
                    <p x-text="count"></p>
                    <button class="ml-2" x-on:click="count++">"inc"</button>
                    <p x-text="value"></p>
                </div>
            </div>
        </HtmlC>
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
        <HtmlC x-data="{pass: '', email: '', api_res: ''}">
        <div class="container mx-auto px-4 py-8">
            <h1 class="text-3xl font-bold text-gray-900">"Welcome to our site!"</h1>
            <div class="mt-4">
                <h2 class="text-xl font-bold text-gray-800">"Login"</h2>
                <p
                    class="font-bold text-red-500 bg-red-500 bg-opacity-20
                        rounded-lg border border-red-500 text-xs mt-4 p-4 hidden"
                    x-text="api_res"
                    x-show="api_res.length !== 0"
                    x-init="() => {
                        $el.classList.remove('hidden'); 
                        $el.classList.add('block'); 
                    }"
                />
                <div class="mt-4">
                    <ValueInput name="Email" ty="email" var="email"/>
                    <ValueInput name="Password" ty="password" var="pass"/>
                    <div class="flex justify-end">
                        <button class="px-4 py-2 font-bold text-white bg-indigo-500 rounded-lg
                            hover:bg-indigo-700 focus:outline-none focus:shadow-outline-indigo 
                            active:bg-indigo-800" 
                            x-on:click="
                                fetch('/api/' + (email) +  '/' + (pass))
                                    .then(response => response.text())
                                    .then(data => api_res = data)
                            "
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

async fn home() -> Html<String> {
    Html(
        "<!DOCTYPE html>".to_owned()
            + &render_to_string(|cx| {
                view! {cx,
                    <Home />
                }
            })
            .replace("<!--/-->", "")
            .replace("<!--#-->", ""),
    )
}
async fn hello(Path(name): Path<String>) -> Html<String> {
    Html(
        "<!DOCTYPE html>".to_owned()
            + &render_to_string(|cx| {
                view! {cx,
                    <Hello name=name/>
                }
            })
            .replace("<!--/-->", "")
            .replace("<!--#-->", ""),
    )
}

async fn login(Path((email, pass)): Path<(String, String)>) -> String {
    format!("Logged in email {email} with pass {pass}")
}

/// Convert the Errors from ServeDir to a type that implements IntoResponse
async fn handle_file_error(err: std::io::Error) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("File Not Found: {}", err))
}

#[tokio::main]
async fn main() -> Result<()> {
    let _ds = Datastore::new("memory").await?;
    let _se = Session::for_db("test", "test");

    let static_service = {
        use axum::error_handling::HandleError;
        use tower_http::services::ServeDir;

        HandleError::new(ServeDir::new("./static"), handle_file_error)
    };

    let router = Router::new()
        .route("/hello/:name", get(hello))
        .route("/api/:email/:mail", get(login))
        .route("/", get(home))
        .nest_service(&format!("/static"), static_service);

    Ok(Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(router.into_make_service())
        .await?)
}
