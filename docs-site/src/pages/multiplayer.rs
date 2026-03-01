use leptos::prelude::*;
use crate::components::page_head::PageHead;

#[component]
pub fn Multiplayer() -> impl IntoView {
    view! {
        <PageHead
            title="Multiplayer Guide"
            description="How to play LiquidWar.io multiplayer — create rooms, join games, host matches, and compete online."
            path="/docs/multiplayer"
            breadcrumbs=&[("Home", "/docs"), ("Multiplayer", "/docs/multiplayer")]
        />
        <article class="page">
            <h1>"Multiplayer Guide"</h1>

            <p class="page-intro">
                "LiquidWar.io supports real-time online multiplayer via WebSocket. Up to 32 players "
                "can compete in a single match."
            </p>

            <section>
                <h2>"Getting Started"</h2>
                <ol>
                    <li>"Enter your name on the main menu"</li>
                    <li>"Click Multiplayer"</li>
                    <li>"Choose how you want to play: Quick Play, Create Room, Join Room, or Browse"</li>
                </ol>
            </section>

            <section>
                <h2>"Quick Play"</h2>
                <p>
                    "The fastest way to get into a game. Quick Play automatically searches for a public "
                    "room with space available. If none is found, it creates a new public room for you. "
                    "Other players can then join your room."
                </p>
            </section>

            <section>
                <h2>"Create Room"</h2>
                <p>"Host your own game with full control over settings:"</p>
                <ul>
                    <li><strong>"Map"</strong>" — Choose from 200+ maps"</li>
                    <li><strong>"Rules"</strong>" — Vanilla (balanced defaults) or Custom (adjustable sliders)"</li>
                    <li><strong>"Bots"</strong>" — Add 0-31 AI opponents to fill the game"</li>
                    <li><strong>"Public/Private"</strong>" — Public rooms appear in the browse list; private "
                        "rooms require the room code to join"</li>
                </ul>
                <p>
                    "After creating, you'll be placed in the room lobby with a 4-character room code "
                    "displayed at the top. Share this code with friends so they can join."
                </p>
            </section>

            <section>
                <h2>"Room Codes"</h2>
                <p>
                    "Each room gets a unique 4-character code like AXKW or B3NP. Codes use an "
                    "unambiguous character set that excludes easily confused characters:"
                </p>
                <p class="code-sample">"A B C D E F G H J K L M N P Q R S T U V W X Y Z 2 3 4 5 6 7 8 9"</p>
                <p>
                    "No I, O, 0, or 1 — so you'll never confuse I with 1 or O with 0."
                </p>
            </section>

            <section>
                <h2>"Join Room"</h2>
                <p>
                    "Enter a 4-character room code to join a specific game. The code is case-insensitive. "
                    "You can get room codes from friends or copy them from the browse list."
                </p>
            </section>

            <section>
                <h2>"Browse Rooms"</h2>
                <p>
                    "View all public rooms currently available. The room list shows:"
                </p>
                <ul>
                    <li>"Room code"</li>
                    <li>"Host name"</li>
                    <li>"Map name"</li>
                    <li>"Player count"</li>
                    <li>"Game mode (Vanilla/Custom)"</li>
                    <li>"Join button"</li>
                </ul>
                <p>"Click Join to enter any room that isn't full."</p>
            </section>

            <section>
                <h2>"Room Lobby"</h2>
                <p>"Once in a room, you'll see the lobby with:"</p>
                <ul>
                    <li><strong>"Player list"</strong>" — All connected players with team colors and ready status"</li>
                    <li><strong>"Room code"</strong>" — Click to copy, share with friends"</li>
                    <li><strong>"Map preview"</strong>" — The selected map (host can change)"</li>
                    <li><strong>"Rules"</strong>" — Current game settings"</li>
                </ul>

                <h3>"Host Controls"</h3>
                <p>
                    "The room host (creator) has additional controls:"
                </p>
                <ul>
                    <li>"Change the map"</li>
                    <li>"Adjust game rules"</li>
                    <li>"Add or remove bots"</li>
                    <li>"Toggle public/private"</li>
                    <li>"Start the game when ready"</li>
                </ul>

                <h3>"Ready System"</h3>
                <p>
                    "Non-host players click Ready to signal they're prepared to play. The host can see "
                    "who's ready and start the game at any time. When the game starts, everyone sees a "
                    "3-2-1 countdown before the match begins."
                </p>
            </section>

            <section>
                <h2>"During the Game"</h2>
                <p>
                    "Multiplayer games work identically to single player — move your cursor to guide your "
                    "army. The game runs at 20 ticks per second, with your cursor position sent to the "
                    "server in real-time."
                </p>
                <p>
                    "Press Escape to open the in-game menu where you can adjust audio settings or "
                    "return to the lobby."
                </p>
            </section>
        </article>
    }
}
