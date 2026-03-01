use leptos::prelude::*;
use crate::components::page_head::PageHead;

#[component]
pub fn Mechanics() -> impl IntoView {
    view! {
        <PageHead
            title="Game Mechanics"
            description="Deep dive into LiquidWar.io mechanics — gradient algorithm, fighter movement, combat system, and army spawning."
            path="/docs/mechanics"
            breadcrumbs=&[("Home", "/docs"), ("Mechanics", "/docs/mechanics")]
        />
        <article class="page">
            <h1>"Game Mechanics"</h1>

            <p class="page-intro">
                "LiquidWar.io is built on the original Liquid War 5 algorithm by Thomas Colcombet. "
                "This page explains the core systems that make the game work."
            </p>

            <section>
                <h2>"The Gradient Algorithm"</h2>
                <p>
                    "The heart of Liquid War is the gradient field. Each team has a gradient — a 2D field "
                    "covering every passable pixel on the map. The gradient stores estimated distances from "
                    "each position to the team's cursor."
                </p>
                <p>
                    "When you move your cursor, the gradient starts with a high value (2,000,000) at the cursor "
                    "position. Each game tick, the gradient spreads outward in one of 12 directions. Over multiple "
                    "ticks, it fills the entire map with distance estimates. Fighters then follow the gradient "
                    "downhill — always moving toward lower values, which leads them toward the cursor."
                </p>
                <h3>"12 Directions"</h3>
                <p>
                    "Unlike a simple 4 or 8 directional system, Liquid War uses 12 directions for smoother "
                    "movement: NNE, NE, ENE, ESE, SE, SSE, SSW, SW, WSW, WNW, NW, NNW. Each direction "
                    "maps to a pixel offset — the cardinal directions move 1 pixel, while diagonals move "
                    "1 pixel in each axis."
                </p>
                <h3>"Spreading"</h3>
                <p>
                    "Each tick, only one direction is processed. This means it takes 12 ticks to update all "
                    "directions — a deliberate design choice from the original LW5 that keeps CPU usage low "
                    "while giving fighters enough information to navigate. The spreading works both forward "
                    "(SE half of the map) and backward (NW half), ensuring even coverage."
                </p>
            </section>

            <section>
                <h2>"Fighter Movement"</h2>
                <p>
                    "Each fighter has a position, health, team, and a current direction. Every tick, fighters "
                    "determine which direction has the lowest gradient value and attempt to move there."
                </p>
                <h3>"Movement Priority"</h3>
                <p>
                    "When a fighter wants to move, it tries up to 5 directions in priority order. The primary "
                    "direction is the gradient-optimal one. If that cell is blocked (wall or another fighter), "
                    "it tries alternates that fan out to the sides. Fighters alternate between clockwise and "
                    "counterclockwise preference to avoid getting stuck in symmetric patterns."
                </p>
                <h3>"Blocking"</h3>
                <p>
                    "If all 5 directions are blocked, the fighter stays put. A blocked fighter next to allies "
                    "will heal instead of moving. A blocked fighter next to enemies will take side damage. "
                    "This creates a natural front-line mechanic where armies press against each other."
                </p>
            </section>

            <section>
                <h2>"Combat System"</h2>
                <p>
                    "Combat happens automatically when fighters move into cells occupied by enemies."
                </p>

                <h3>"Front Attack"</h3>
                <p>
                    "When a fighter tries to move into a cell occupied by an enemy, it deals front attack "
                    "damage. The damage formula scales with the Attack slider and is influenced by the "
                    "attacker's team size relative to the total army (the Influence parameter). Larger teams "
                    "deal proportionally more damage per fighter."
                </p>

                <h3>"Side Attack"</h3>
                <p>
                    "Fighters also take reduced damage from enemies in adjacent cells (not directly in front). "
                    "Side attacks deal 1/16th of front attack damage. This means being surrounded is far more "
                    "dangerous than a head-on collision."
                </p>

                <h3>"Health"</h3>
                <p>
                    "Each fighter has up to 16,384 health points, displayed as 7 brightness levels. Brighter "
                    "fighters are healthier. When health drops below zero, the fighter converts to the "
                    "attacking team with health reset based on the Health slider."
                </p>

                <h3>"Healing"</h3>
                <p>
                    "Fighters that are blocked (cannot move) and adjacent to allied fighters regenerate "
                    "health. The healing rate is controlled by the Defense slider. This makes dense, "
                    "compact armies more resilient than spread-out ones."
                </p>
            </section>

            <section>
                <h2>"Team Conversion"</h2>
                <p>
                    "When an enemy fighter's health is reduced below zero, it instantly converts to the "
                    "attacking team. The converted fighter receives new health based on the Health slider "
                    "and begins following the new team's gradient toward their cursor."
                </p>
                <p>
                    "This creates a snowball effect — as you convert enemies, your army grows larger, which "
                    "increases your per-fighter damage (via Influence), making it easier to convert even more. "
                    "Conversely, a shrinking team becomes progressively weaker."
                </p>
            </section>

            <section>
                <h2>"Army Spawning"</h2>
                <p>
                    "At the start of a game, each team spawns fighters on the map. The total number of "
                    "fighters is determined by the Army Size slider, which maps to a fill percentage "
                    "of the map's passable area:"
                </p>
                <table class="info-table">
                    <thead>
                        <tr><th>"Slider"</th><th>"Fill %"</th><th>"Slider"</th><th>"Fill %"</th></tr>
                    </thead>
                    <tbody>
                        <tr><td>"0"</td><td>"1%"</td><td>"16 (default)"</td><td>"25%"</td></tr>
                        <tr><td>"4"</td><td>"5%"</td><td>"20"</td><td>"33%"</td></tr>
                        <tr><td>"8"</td><td>"10%"</td><td>"24"</td><td>"50%"</td></tr>
                        <tr><td>"12"</td><td>"18%"</td><td>"32"</td><td>"99%"</td></tr>
                    </tbody>
                </table>
                <p>
                    "Total fighters are split equally among all active teams. Fighters spawn in a spiral "
                    "pattern around each team's starting position."
                </p>

                <h3>"Spawn Positions"</h3>
                <p>
                    "The first 6 teams use fixed positions inspired by the original LW5 layout — corners "
                    "and edges of the map. Teams 7 and beyond use a \"farthest-point\" heuristic: the game "
                    "samples passable positions across the map and picks the point that maximizes the "
                    "minimum distance to all existing spawn points. This ensures fair distribution even "
                    "on complex maps with many teams."
                </p>
            </section>

            <section>
                <h2>"Influence"</h2>
                <p>
                    "The Influence slider (0-32, default 8) controls how much a team's relative size affects "
                    "its combat power. At higher influence values, a team with more fighters gets a significant "
                    "damage bonus, creating stronger snowball effects. At lower values, small teams can hold "
                    "their ground more easily."
                </p>
                <p>
                    "The formula uses a square-root-of-square-root scaling, meaning the bonus grows quickly at "
                    "first but tapers off. A team with twice as many fighters doesn't deal twice the damage — "
                    "the advantage is real but not overwhelming."
                </p>
            </section>
        </article>
    }
}
