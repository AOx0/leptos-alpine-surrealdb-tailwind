#![allow(non_snake_case)]

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
        <html class="dark">
            <head>
                <link href="/static/styles.css" rel="stylesheet"/>
                <script src="https://unpkg.com/alpinejs@3.x.x/dist/cdn.min.js" defer init></script>
            </head>
            <body x-data=x_data class="bg-white dark:bg-black text-black dark:text-white">
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
                    <button class="ml-5" x-on:click="count++">"inc"</button>
                    <p x-text="value"></p>
                </div>
            </div>
        </HtmlC>
    }
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

/// Convert the Errors from ServeDir to a type that implements IntoResponse
async fn handle_file_error(err: std::io::Error) -> (StatusCode, String) {
    (StatusCode::NOT_FOUND, format!("File Not Found: {}", err))
}

#[tokio::main]
async fn main() -> Result<()> {
    const STATIC_DIR: &'static str = "static";
    let _ds = Datastore::new("memory").await?;
    let _se = Session::for_db("test", "test");

    let static_service = {
        use axum::error_handling::HandleError;
        use tower_http::services::ServeDir;

        HandleError::new(ServeDir::new(STATIC_DIR), handle_file_error)
    };

    let router = Router::new()
        .route("/hello/:name", get(hello))
        .nest_service(&format!("/{}", STATIC_DIR), static_service);

    Ok(Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(router.into_make_service())
        .await?)
}
