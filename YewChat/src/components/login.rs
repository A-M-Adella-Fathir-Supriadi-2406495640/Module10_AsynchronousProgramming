use web_sys::HtmlInputElement;
use yew::functional::*;
use yew::prelude::*;
use yew_router::prelude::*;

use crate::Route;
use crate::User;

#[function_component(Login)]
pub fn login() -> Html {
    let username = use_state(|| String::new());
    let user = use_context::<User>().expect("No context found.");

    let oninput = {
        let current_username = username.clone();
        Callback::from(move |e: InputEvent| {
            let input: HtmlInputElement = e.target_unchecked_into();
            current_username.set(input.value());
        })
    };

    let onclick = {
        let username = username.clone();
        let user = user.clone();
        Callback::from(move |_| {
            *user.username.borrow_mut() = (*username).clone();
        })
    };

    let char_left = if username.len() < 2 { 2 - username.len() } else { 0 };

    html! {
        <div class="min-h-screen w-screen flex items-center justify-center bg-gray-900">
            <div class="flex flex-col items-center w-full max-w-sm px-8">

                // Logo & branding
                <div class="text-center mb-8">
                    <div class="text-7xl mb-3">{"🦀"}</div>
                    <h1 class="text-4xl font-extrabold text-white tracking-tight">{"RustChat"}</h1>
                    <p class="text-gray-400 mt-2 text-sm">{"Real-time chat — compiled to WebAssembly"}</p>
                </div>

                // Card
                <div class="w-full bg-gray-800 rounded-2xl p-6 shadow-2xl border border-gray-700">
                    <label class="block text-gray-400 text-xs font-semibold uppercase tracking-widest mb-2">
                        {"Your username"}
                    </label>
                    <input
                        {oninput}
                        type="text"
                        maxlength="20"
                        placeholder="e.g. rustacean42"
                        class="w-full px-4 py-3 bg-gray-700 border border-gray-600 rounded-xl text-white placeholder-gray-500 focus:outline-none focus:border-orange-500 focus:ring-1 focus:ring-orange-500 transition-colors"
                    />

                    // hint
                    <p class="text-gray-500 text-xs mt-2 ml-1">
                        {
                            if char_left > 0 {
                                format!("{} more character{} needed", char_left, if char_left == 1 { "" } else { "s" })
                            } else {
                                format!("✓  {} characters — looking good!", username.len())
                            }
                        }
                    </p>

                    <Link<Route> to={Route::Chat}>
                        <button
                            {onclick}
                            disabled={username.len() < 2}
                            class="mt-4 w-full py-3 bg-orange-500 hover:bg-orange-400 disabled:bg-gray-700 disabled:text-gray-500 disabled:cursor-not-allowed text-white font-bold rounded-xl transition-all duration-200 text-sm tracking-wide uppercase"
                        >
                            {"🚀  Launch into Chat"}
                        </button>
                    </Link<Route>>
                </div>

                // Footer quote
                <p class="text-gray-600 text-xs mt-6 text-center italic">
                    {"\"Memory safety without a garbage collector.\" — The Rust Book"}
                </p>
            </div>
        </div>
    }
}
