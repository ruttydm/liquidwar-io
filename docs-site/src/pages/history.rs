use leptos::prelude::*;
use crate::components::page_head::PageHead;

#[component]
pub fn History() -> impl IntoView {
    view! {
        <PageHead
            title="History"
            description="The history of Liquid War — from the original MS-DOS game to the modern LiquidWar.io web port."
            path="/docs/history"
            breadcrumbs=&[("Home", "/docs"), ("History", "/docs/history")]
        />
        <article class="page">
            <h1>"History"</h1>

            <p class="page-intro">
                "Liquid War has a long history spanning three decades. From a student project on MS-DOS "
                "to a modern web game, the core concept has remained the same: control an army of liquid "
                "with nothing but a cursor."
            </p>

            <section class="timeline">
                <div class="timeline-entry">
                    <h2>"Liquid War 1-3"</h2>
                    <span class="timeline-date">"~1995 - 2002"</span>
                    <p>
                        "Created by Thomas Colcombet as a student project, the original Liquid War ran on "
                        "MS-DOS. The concept was revolutionary: instead of selecting and commanding individual "
                        "units, you control an entire army that flows like liquid toward your cursor."
                    </p>
                    <p>
                        "The early versions established the core gradient-descent algorithm that would define "
                        "the series. Players quickly discovered the game's surprising strategic depth — "
                        "positioning, chokepoints, and timing mattered far more than raw clicking speed."
                    </p>
                    <p>
                        "Liquid War 3 supported up to 6 players on a single computer, each using different "
                        "keys. The game came with a collection of hand-drawn maps, many of which are still "
                        "available in LiquidWar.io today as \"legacy\" maps."
                    </p>
                </div>

                <div class="timeline-entry">
                    <h2>"Liquid War 5"</h2>
                    <span class="timeline-date">"2002"</span>
                    <p>
                        "The definitive version. Built with the Allegro game library, LW5 ran on Linux, "
                        "Windows, and Mac OS X. It featured polished graphics, MIDI music, an expanded map "
                        "collection, and a refined version of the gradient algorithm."
                    </p>
                    <p>
                        "LW5 supported up to 6 teams with configurable AI, custom rules (attack, defense, "
                        "health, influence, army size), and dozens of maps contributed by the community. "
                        "Key contributors include Kasper Hviid (66 maps), Christian Mauduit (59 maps), "
                        "and Valerie Mauduit (24 maps)."
                    </p>
                    <p>
                        "Released under the GNU GPL v2 license, LW5 became part of many Linux distributions "
                        "and developed a dedicated following. Its source code served as the reference "
                        "implementation for all future versions."
                    </p>
                </div>

                <div class="timeline-entry">
                    <h2>"Liquid War 6"</h2>
                    <span class="timeline-date">"2005 - 2015"</span>
                    <p>
                        "Christian Mauduit (ufoot) undertook an ambitious rewrite as an official GNU project. "
                        "LW6 aimed to modernize the game with 3D graphics, network multiplayer, and a "
                        "flexible map system with layers, textures, and metadata."
                    </p>
                    <p>
                        "While LW6 never reached the playability of LW5, its contribution to the map "
                        "ecosystem was significant. The LW6 extra maps collection includes original maps "
                        "from community creators, legacy ports from LW3 and LW5, and experimental designs "
                        "with advanced features."
                    </p>
                    <p>
                        "The LW6 metadata format documented map authors, descriptions, and licenses — "
                        "information preserved in LiquidWar.io's map gallery."
                    </p>
                </div>

                <div class="timeline-entry">
                    <h2>"LiquidWar.io"</h2>
                    <span class="timeline-date">"2025 - Present"</span>
                    <p>
                        "A modern web-based reimplementation faithful to the LW5 game engine. Built with:"
                    </p>
                    <ul>
                        <li><strong>"Rust"</strong>" — Core game engine (gradient algorithm, combat, AI)"</li>
                        <li><strong>"WebAssembly"</strong>" — Game engine compiled to WASM for client-side play"</li>
                        <li><strong>"TypeScript"</strong>" — Frontend UI, rendering, and input handling"</li>
                        <li><strong>"WebSocket"</strong>" — Real-time multiplayer server in Rust"</li>
                    </ul>
                    <p>
                        "LiquidWar.io expands on LW5 with support for up to 32 teams, online multiplayer "
                        "with room codes, 200+ maps from the combined LW3/LW5/LW6 collections, mobile "
                        "support with a virtual joystick, and the original MIDI soundtrack."
                    </p>
                    <p>
                        "The game runs entirely in the browser — no downloads, no plugins, no accounts. "
                        "Just open the page and play."
                    </p>
                </div>
            </section>
        </article>
    }
}
