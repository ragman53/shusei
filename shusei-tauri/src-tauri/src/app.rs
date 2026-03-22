use leptos::prelude::*;

/// Simple counter component demonstrating Leptos signals
#[component]
pub fn Counter() -> impl IntoView {
    let (count, set_count) = signal(0i32);

    let increment = move |_| {
        set_count.update(|n| *n += 1);
    };

    let decrement = move |_| {
        set_count.update(|n| *n -= 1);
    };

    view! {
        <div style="padding: 20px; font-family: sans-serif;">
            <h1>"Shusei - Tauri + Leptos"</h1>
            <p>"Counter: " {count}</p>
            <button on:click=increment style="margin-right: 10px;">"+1"</button>
            <button on:click=decrement>"-1"</button>
        </div>
    }
}

/// Main app component
#[component]
pub fn App() -> impl IntoView {
    view! {
        <Counter/>
    }
}
