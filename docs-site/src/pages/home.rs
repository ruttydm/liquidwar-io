use leptos::prelude::*;
use crate::components::page_head::PageHead;

#[component]
pub fn HomePage() -> impl IntoView {
    view! {
        <PageHead
            title="Documentation"
            description="LiquidWar.io documentation — learn how to play, game mechanics, maps, settings, and multiplayer guide."
            path="/docs"
            breadcrumbs=&[("Home", "/docs")]
            is_home=true
        />
        <article class="page">
            <div class="hero">
                <h1>"LIQUIDWAR.IO"</h1>
                <p class="hero-sub">"DOCUMENTATION"</p>
            </div>

            <section class="intro">
                <h2>"What is LiquidWar.io?"</h2>
                <p>
                    "LiquidWar.io is a web-based reimplementation of the classic Liquid War 5 game. "
                    "You control an army of liquid fighters using only a cursor. Your fighters automatically "
                    "chase your cursor and attack any enemies they encounter along the way. The last team "
                    "standing wins."
                </p>
                <p>
                    "Unlike traditional strategy games, you don't select units or issue commands. Instead, "
                    "the game uses a gradient descent algorithm — your entire army flows toward your cursor "
                    "like water flowing downhill. This creates unique emergent gameplay where positioning, "
                    "chokepoints, and timing are everything."
                </p>
                <p>
                    "Play solo against up to 31 AI opponents, or go online and compete with other players "
                    "in real-time multiplayer matches. Choose from over 200 community-created maps, each "
                    "offering different strategic challenges."
                </p>
            </section>

            <section class="quick-links">
                <h2>"Explore"</h2>
                <div class="link-grid">
                    <a href="/docs/how-to-play" class="link-card">
                        <h3>"How to Play"</h3>
                        <p>"Controls, objectives, and getting started"</p>
                    </a>
                    <a href="/docs/mechanics" class="link-card">
                        <h3>"Mechanics"</h3>
                        <p>"Gradient algorithm, combat, and spawning"</p>
                    </a>
                    <a href="/docs/maps" class="link-card">
                        <h3>"Maps"</h3>
                        <p>"Browse all 206 community maps"</p>
                    </a>
                    <a href="/docs/settings" class="link-card">
                        <h3>"Settings"</h3>
                        <p>"Game rules, sliders, and audio options"</p>
                    </a>
                    <a href="/docs/multiplayer" class="link-card">
                        <h3>"Multiplayer"</h3>
                        <p>"Create rooms, join games, host matches"</p>
                    </a>
                    <a href="/docs/history" class="link-card">
                        <h3>"History"</h3>
                        <p>"From MS-DOS to the modern web"</p>
                    </a>
                </div>
            </section>

            <div class="cta">
                <a href="/" class="cta-btn">"Play Now"</a>
            </div>
        </article>
    }
}
