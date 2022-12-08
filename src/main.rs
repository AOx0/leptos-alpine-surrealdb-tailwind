#![allow(non_snake_case)]

use anyhow::Result;
use axum::routing::get;
use axum::{extract::Path, response::Html, Router, Server};
use leptos::*;
use surrealdb::{Datastore, Session};

#[component]
fn HtmlC<'a>(
    cx: Scope,
    x_data: Option<&'a str>,
    children: Box<dyn Fn() -> Vec<Element>>,
) -> Element {
    view! { cx,
        <html>
            <head>
                <script src="https://unpkg.com/alpinejs@3.x.x/dist/cdn.min.js" defer init></script>
            </head>
            <body x-data=x_data>
            {children}
            </body>
        </html>
    }
}

#[component]
fn Hello(cx: Scope, name: String) -> Element {
    view! {cx,
        <HtmlC x-data="{value: '', count: 0, place: 'Type here...' }">
            <div>
                <input type="text" x-model="value" x-bind:placeholder="place" />
            </div>
            <div>
                <p x-text="count"></p>
                <button x-on:click="count++">"inc"</button>
                <p>"Hola "{&name}</p>
                <p x-text="value"></p>
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

#[tokio::main]
async fn main() -> Result<()> {
    let _ds = Datastore::new("memory").await?;
    let _se = Session::for_db("test", "test");

    let router = Router::new().route("/hello/:name", get(hello));

    Ok(Server::bind(&"0.0.0.0:8000".parse().unwrap())
        .serve(router.into_make_service())
        .await?)
}
