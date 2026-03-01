use leptos::prelude::*;
use leptos_meta::*;
use leptos_router::{
    components::{Route, Router, Routes},
    path,
};

use crate::components::nav::Nav;
use crate::components::footer::Footer;
use crate::pages::{
    home::HomePage,
    how_to_play::HowToPlay,
    mechanics::Mechanics,
    maps_gallery::MapsGallery,
    map_detail::MapDetail,
    settings::Settings,
    multiplayer::Multiplayer,
    history::History,
    credits::Credits,
};

pub fn shell(options: LeptosOptions) -> impl IntoView {
    view! {
        <!DOCTYPE html>
        <html lang="en">
            <head>
                <meta charset="UTF-8" />
                <meta name="viewport" content="width=device-width, initial-scale=1.0" />
                <link rel="preconnect" href="https://fonts.googleapis.com" />
                <link rel="preconnect" href="https://fonts.gstatic.com" crossorigin="" />
                <link href="https://fonts.googleapis.com/css2?family=IBM+Plex+Mono:wght@400;500;700&family=Press+Start+2P&display=swap" rel="stylesheet" />
                <link rel="icon" href="/docs/assets/favicon.ico" sizes="32x32" />
                <link rel="icon" href="/docs/assets/icon-192.png" sizes="192x192" type_="image/png" />
                <link rel="apple-touch-icon" href="/docs/assets/apple-touch-icon.png" />
                <AutoReload options=options.clone() />
                <HydrationScripts options />
                <MetaTags />
            </head>
            <body>
                <App />
            </body>
        </html>
    }
}

#[component]
pub fn App() -> impl IntoView {
    provide_meta_context();

    view! {
        <Stylesheet id="leptos" href="/pkg/docs-site.css" />
        <Title text="LiquidWar.io Documentation" />
        <Meta name="theme-color" content="#000020" />

        <Router>
            <div class="layout">
                <Nav />
                <main class="content">
                    <Routes fallback=|| view! { <p class="page">"Page not found."</p> }>
                        <Route path=path!("/docs") view=HomePage />
                        <Route path=path!("/docs/how-to-play") view=HowToPlay />
                        <Route path=path!("/docs/mechanics") view=Mechanics />
                        <Route path=path!("/docs/maps") view=MapsGallery />
                        <Route path=path!("/docs/maps/:id") view=MapDetail />
                        <Route path=path!("/docs/settings") view=Settings />
                        <Route path=path!("/docs/multiplayer") view=Multiplayer />
                        <Route path=path!("/docs/history") view=History />
                        <Route path=path!("/docs/credits") view=Credits />
                    </Routes>
                </main>
                <Footer />
            </div>
        </Router>
    }
}
