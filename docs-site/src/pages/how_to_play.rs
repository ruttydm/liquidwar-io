use leptos::prelude::*;
use crate::components::page_head::PageHead;

#[component]
pub fn HowToPlay() -> impl IntoView {
    view! {
        <PageHead
            title="How to Play"
            description="Learn how to play LiquidWar.io — controls, game modes, objectives, and beginner tips."
            path="/docs/how-to-play"
            breadcrumbs=&[("Home", "/docs"), ("How to Play", "/docs/how-to-play")]
        />
        <article class="page">
            <h1>"How to Play"</h1>

            <section>
                <h2>"Objective"</h2>
                <p>
                    "Your goal is simple: be the last team standing. You control an army of liquid fighters "
                    "using only your cursor. Your fighters automatically flow toward your cursor position "
                    "and attack any enemy fighters they encounter. When an enemy fighter's health drops to "
                    "zero, it converts to your team. Eliminate all opposing teams to win."
                </p>
            </section>

            <section>
                <h2>"Controls"</h2>
                <table class="info-table">
                    <thead>
                        <tr><th>"Input"</th><th>"Action"</th></tr>
                    </thead>
                    <tbody>
                        <tr><td>"Arrow Keys"</td><td>"Move cursor up/down/left/right"</td></tr>
                        <tr><td>"W A S D"</td><td>"Alternative cursor movement"</td></tr>
                        <tr><td>"Mouse Click + Drag"</td><td>"Move cursor (click to anchor, drag to steer)"</td></tr>
                        <tr><td>"Virtual Joystick"</td><td>"Mobile touch control (bottom-right)"</td></tr>
                        <tr><td>"Escape"</td><td>"Open in-game menu"</td></tr>
                    </tbody>
                </table>
                <p>
                    "The mouse uses a \"gap\" system: click and hold to set an anchor point, then move the "
                    "mouse away from that point. Moving 18+ pixels in any direction steers your cursor "
                    "that way. This lets you steer precisely without lifting your finger."
                </p>
            </section>

            <section>
                <h2>"Game Modes"</h2>

                <h3>"Single Player"</h3>
                <p>
                    "From the main menu, click Single Player. You'll see a two-panel setup screen:"
                </p>
                <ul>
                    <li>"Left panel: Configure teams (up to 32). Set each slot to Human, CPU, or Off. "
                        "Rename teams and adjust game rules (Attack, Defense, Health, Influence, Army Size)."</li>
                    <li>"Right panel: Select a map from the grid. Click a map to highlight it, then click "
                        "Start Game to begin."</li>
                </ul>
                <p>
                    "Choose Vanilla for standard balanced rules (8/8/8/8/16) or Custom to adjust "
                    "each parameter individually."
                </p>

                <h3>"Multiplayer"</h3>
                <p>
                    "Click Multiplayer from the main menu. You can:"
                </p>
                <ul>
                    <li><strong>"Quick Play"</strong>" — Automatically join or create a public room"</li>
                    <li><strong>"Create Room"</strong>" — Host a game with your own settings"</li>
                    <li><strong>"Join Room"</strong>" — Enter a 4-character room code"</li>
                    <li><strong>"Browse Rooms"</strong>" — See all public rooms and join one"</li>
                </ul>
            </section>

            <section>
                <h2>"Your First Game"</h2>
                <ol>
                    <li>"Enter your player name on the main menu"</li>
                    <li>"Click Single Player"</li>
                    <li>"Leave the default 2 teams (you + 1 CPU)"</li>
                    <li>"Pick any map from the grid"</li>
                    <li>"Click Start Game"</li>
                    <li>"After the 3-2-1 countdown, use arrow keys to move your cursor"</li>
                    <li>"Your fighters (colored army) will flow toward your cursor"</li>
                    <li>"Guide them into enemy territory to convert their fighters"</li>
                    <li>"Win by converting all enemy fighters to your color"</li>
                </ol>
            </section>

            <section>
                <h2>"Tips for Beginners"</h2>
                <ul>
                    <li><strong>"Use chokepoints"</strong>" — Narrow passages on the map create natural "
                        "defensive positions. Funnel enemies through tight spaces where your concentrated "
                        "army has the advantage."</li>
                    <li><strong>"Don't spread too thin"</strong>" — Moving your cursor too far from your "
                        "army stretches your fighters out. A compact army deals more damage than a scattered one."</li>
                    <li><strong>"Flank from behind"</strong>" — Front attacks deal full damage, but moving "
                        "your cursor around to attack from the enemy's rear is devastating."</li>
                    <li><strong>"Watch the info bar"</strong>" — The bottom bar shows each team's fighter "
                        "count. If you're ahead, play aggressively. If behind, defend and pick your battles."</li>
                    <li><strong>"Army size matters"</strong>" — Larger armies deal more damage per fighter "
                        "thanks to the influence system. Converting even a few enemies can snowball quickly."</li>
                </ul>
            </section>
        </article>
    }
}
