use leptos::prelude::*;

#[component]
pub fn Footer() -> impl IntoView {
    view! {
        <footer class="docs-footer">
            <p>"LiquidWar.io — Based on Liquid War 5 by Thomas Colcombet"</p>
            <p class="footer-license">"Licensed under GNU GPL v2"</p>
        </footer>
    }
}
