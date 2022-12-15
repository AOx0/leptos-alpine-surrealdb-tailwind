use super::*;

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
                        <button class="px-4 py-2
                            enabled:opacity-100         disabled:opacity-50
                            enabled:hover:bg-indigo-700 disabled:hover:bg-indigo-500 
                            font-bold text-white bg-indigo-500 rounded-lg
                            focus:outline-none focus:shadow-outline-indigo 
                            active:bg-indigo-800"
                            x-data="{ benable: true }"
                            x-bind:disabled="!benable"
                            x-on:click="
                                benable = false;
                                r_text = false;
                                fetch('/api/public/login', {
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
                                        if (r_text === true) {
                                            api_res = data; 
                                            $refs.api.classList.remove('invisible');
                                        }
                                        benable = true;
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

pub async fn home() -> Html<String> {
    Html(render(|cx| {
        view! {cx, <Home /> }
    }))
}
