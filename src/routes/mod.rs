use super::*;
use leptos::*;

pub mod login;

pub fn render(view: impl FnOnce(Scope) -> Element + 'static) -> String {
    "<!DOCTYPE html>".to_owned()
        + &render_to_string(view)
            .replace("<!--/-->", "")
            .replace("<!--#-->", "")
}

#[component]
pub fn HtmlC<'a>(
    cx: Scope,
    x_data: Option<&'a str>,
    children: Box<dyn Fn() -> Vec<Element>>,
) -> Element {
    view! { cx,
        <html class="">
            <head>
                <link rel="stylesheet" href="/public/static/styles.css"/>
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
