use leptos::prelude::*;

#[component]
pub fn Nav() -> impl IntoView {
    let (open, set_open) = signal(false);

    let nav_links: Vec<(&str, &str)> = vec![
        ("/docs", "Home"),
        ("/docs/how-to-play", "How to Play"),
        ("/docs/mechanics", "Mechanics"),
        ("/docs/maps", "Maps"),
        ("/docs/settings", "Settings"),
        ("/docs/multiplayer", "Multiplayer"),
        ("/docs/history", "History"),
        ("/docs/credits", "Credits"),
    ];

    view! {
        <button class="nav-hamburger" on:click=move |_| set_open.update(|o| *o = !*o)>
            {move || if open.get() { "X" } else { "=" }}
        </button>
        <nav class="sidebar" class:open=move || open.get()>
            <div class="sidebar-header">
                <a href="/docs" class="sidebar-title">"LIQUIDWAR.IO"</a>
                <span class="sidebar-subtitle">"DOCS"</span>
            </div>
            <ul class="sidebar-links">
                {nav_links.into_iter().map(|(href, label)| {
                    view! {
                        <li>
                            <a href=href on:click=move |_| set_open.set(false)>{label}</a>
                        </li>
                    }
                }).collect_view()}
            </ul>
            <div class="sidebar-footer">
                <a href="/" class="play-link">"Play Game"</a>
                <a href="https://github.com/ruttydm/liquidwar-io" target="_blank" rel="noopener" class="github-link">"GitHub"</a>
            </div>
        </nav>
        <div class="nav-overlay" class:open=move || open.get()
             on:click=move |_| set_open.set(false)></div>
    }
}
