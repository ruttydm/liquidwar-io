use leptos::prelude::*;
use crate::components::page_head::PageHead;

#[component]
pub fn Settings() -> impl IntoView {
    view! {
        <PageHead
            title="Settings & Rules"
            description="Complete reference for LiquidWar.io game settings — attack, defense, health, influence, army size, cursor speed, and audio."
            path="/docs/settings"
            breadcrumbs=&[("Home", "/docs"), ("Settings", "/docs/settings")]
        />
        <article class="page">
            <h1>"Settings & Rules"</h1>

            <p class="page-intro">
                "LiquidWar.io offers extensive customization through game rule sliders and audio settings. "
                "In Vanilla mode, all rules are locked to balanced defaults. In Custom mode, each parameter "
                "can be adjusted individually."
            </p>

            <section>
                <h2>"Game Rules"</h2>
                <p>
                    "These sliders control the core combat and army parameters. Each ranges from 0 to 32."
                </p>

                <table class="info-table">
                    <thead>
                        <tr>
                            <th>"Parameter"</th>
                            <th>"Range"</th>
                            <th>"Default"</th>
                            <th>"Description"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>"Attack"</td>
                            <td>"0 - 32"</td>
                            <td>"8"</td>
                            <td>"Damage dealt when a fighter attacks an enemy. Higher values make "
                                "combat more lethal and games faster."</td>
                        </tr>
                        <tr>
                            <td>"Defense"</td>
                            <td>"0 - 32"</td>
                            <td>"8"</td>
                            <td>"Healing rate for blocked fighters near allies. Higher values make "
                                "armies more resilient and harder to break through."</td>
                        </tr>
                        <tr>
                            <td>"Health"</td>
                            <td>"0 - 32"</td>
                            <td>"8"</td>
                            <td>"Initial health of newly converted fighters. Higher values mean "
                                "converted fighters are immediately strong."</td>
                        </tr>
                        <tr>
                            <td>"Influence"</td>
                            <td>"0 - 32"</td>
                            <td>"8"</td>
                            <td>"How much team size affects combat power. Higher values create "
                                "stronger snowball effects — large teams become dominant faster."</td>
                        </tr>
                        <tr>
                            <td>"Army Size"</td>
                            <td>"0 - 32"</td>
                            <td>"16"</td>
                            <td>"Controls map fill percentage. Default 16 = 25% fill. Higher values "
                                "create more crowded maps with more fighters."</td>
                        </tr>
                    </tbody>
                </table>
            </section>

            <section>
                <h2>"Army Size Fill Table"</h2>
                <p>
                    "The Army Size slider maps to a percentage of the map's passable area that will be "
                    "filled with fighters. The mapping is non-linear — low values give fine control, while "
                    "high values fill the map rapidly."
                </p>
                <table class="info-table compact">
                    <thead>
                        <tr>
                            <th>"Slider"</th>
                            <th>"0"</th><th>"2"</th><th>"4"</th><th>"6"</th><th>"8"</th>
                            <th>"10"</th><th>"12"</th><th>"14"</th><th>"16"</th><th>"18"</th>
                            <th>"20"</th><th>"22"</th><th>"24"</th><th>"26"</th><th>"28"</th>
                            <th>"30"</th><th>"32"</th>
                        </tr>
                    </thead>
                    <tbody>
                        <tr>
                            <th>"Fill %"</th>
                            <td>"1"</td><td>"3"</td><td>"5"</td><td>"8"</td><td>"10"</td>
                            <td>"14"</td><td>"18"</td><td>"22"</td><td>"25"</td><td>"29"</td>
                            <td>"33"</td><td>"40"</td><td>"50"</td><td>"60"</td><td>"70"</td>
                            <td>"90"</td><td>"99"</td>
                        </tr>
                    </tbody>
                </table>
            </section>

            <section>
                <h2>"Vanilla vs Custom"</h2>
                <p>
                    "In both Single Player and Multiplayer, you can choose between two rule modes:"
                </p>
                <ul>
                    <li><strong>"Vanilla"</strong>" — All rules locked to defaults: Attack 8, Defense 8, "
                        "Health 8, Influence 8, Army Size 16. This ensures balanced, fair games."</li>
                    <li><strong>"Custom"</strong>" — All sliders unlocked. Experiment with extreme settings: "
                        "max attack for instant kills, max defense for near-immortal armies, or max army size "
                        "for chaotic battles."</li>
                </ul>
            </section>

            <section>
                <h2>"Player Settings"</h2>
                <table class="info-table">
                    <thead>
                        <tr><th>"Setting"</th><th>"Range"</th><th>"Default"</th><th>"Description"</th></tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>"Cursor Speed"</td>
                            <td>"1 - 5"</td>
                            <td>"1"</td>
                            <td>"Pixels per tick the cursor moves. Higher speeds cover more ground "
                                "but reduce precision."</td>
                        </tr>
                        <tr>
                            <td>"Player Name"</td>
                            <td>"1-12 chars"</td>
                            <td>"Player"</td>
                            <td>"Your display name in lobbies and the game-over screen."</td>
                        </tr>
                    </tbody>
                </table>
            </section>

            <section>
                <h2>"Audio Settings"</h2>
                <table class="info-table">
                    <thead>
                        <tr><th>"Setting"</th><th>"Range"</th><th>"Default"</th><th>"Description"</th></tr>
                    </thead>
                    <tbody>
                        <tr>
                            <td>"Music Volume"</td>
                            <td>"0 - 100%"</td>
                            <td>"70%"</td>
                            <td>"MIDI music from the original LW5 soundtrack."</td>
                        </tr>
                        <tr>
                            <td>"SFX Volume"</td>
                            <td>"0 - 100%"</td>
                            <td>"50%"</td>
                            <td>"Sound effects for game events."</td>
                        </tr>
                        <tr>
                            <td>"Water Volume"</td>
                            <td>"0 - 100%"</td>
                            <td>"15%"</td>
                            <td>"Ambient water/wave sounds during gameplay."</td>
                        </tr>
                    </tbody>
                </table>
                <p>
                    "All settings are automatically saved to your browser's local storage and restored "
                    "when you return."
                </p>
            </section>
        </article>
    }
}
