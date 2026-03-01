use leptos::prelude::*;
use crate::components::page_head::PageHead;
use crate::data::maps;

#[component]
pub fn Credits() -> impl IntoView {
    let authors = maps::all_authors();
    let total_maps = maps::all_maps().len();

    view! {
        <PageHead
            title="Credits"
            description="Credits for LiquidWar.io — original game creator, map authors, technology, and licenses."
            path="/docs/credits"
            breadcrumbs=&[("Home", "/docs"), ("Credits", "/docs/credits")]
        />
        <article class="page">
            <h1>"Credits"</h1>

            <section>
                <h2>"Original Game"</h2>
                <p>
                    "Liquid War was created by "<strong>"Thomas Colcombet"</strong>" and released under the "
                    "GNU General Public License v2. The original game's innovative gradient-descent algorithm "
                    "and unique gameplay have inspired ports and reimplementations for over 25 years."
                </p>
            </section>

            <section>
                <h2>"Map Authors"</h2>
                <p>
                    {format!("LiquidWar.io includes {} maps contributed by the community across ", total_maps)}
                    "Liquid War 3, 5, and 6."
                </p>
                <table class="info-table">
                    <thead>
                        <tr><th>"Author"</th><th>"Maps"</th></tr>
                    </thead>
                    <tbody>
                        {authors.into_iter().map(|(name, count)| {
                            view! {
                                <tr>
                                    <td>{name.to_string()}</td>
                                    <td>{count.to_string()}</td>
                                </tr>
                            }
                        }).collect_view()}
                    </tbody>
                </table>
                <p class="note">
                    {format!("{} maps have identified authors. The remaining maps are from the original ", total_maps)}
                    "Liquid War 5 distribution."
                </p>
            </section>

            <section>
                <h2>"Technology"</h2>
                <ul>
                    <li><strong>"Rust"</strong>" — Game engine, multiplayer server, and this documentation site"</li>
                    <li><strong>"WebAssembly"</strong>" — Client-side game engine via wasm-pack"</li>
                    <li><strong>"TypeScript"</strong>" — Frontend UI and input handling"</li>
                    <li><strong>"Three.js / WebGL"</strong>" — Game rendering"</li>
                    <li><strong>"WebSocket"</strong>" — Real-time multiplayer protocol"</li>
                    <li><strong>"Leptos"</strong>" — This documentation site (Rust SSR framework)"</li>
                    <li><strong>"Vite"</strong>" — Frontend build tooling"</li>
                </ul>
            </section>

            <section>
                <h2>"Licenses"</h2>
                <ul>
                    <li>"Game engine and server: GNU GPL v2"</li>
                    <li>"Maps: Various (GPL v2+, GPL v3+, CC BY-SA 3.0) — see individual map pages for details"</li>
                    <li>"Original LW5 MIDI music: GNU GPL v2"</li>
                </ul>
            </section>

            <section>
                <h2>"Special Thanks"</h2>
                <ul>
                    <li>"Thomas Colcombet — for creating Liquid War"</li>
                    <li>"Christian Mauduit (ufoot) — for the LW6 project and map collection infrastructure"</li>
                    <li>"Kasper Hviid — for contributing 65+ maps to the Liquid War ecosystem"</li>
                    <li>"The Liquid War community — for keeping the game alive across decades"</li>
                </ul>
            </section>
        </article>
    }
}
