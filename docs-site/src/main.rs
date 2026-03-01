#[cfg(feature = "ssr")]
#[tokio::main]
async fn main() {
    use axum::Router;
    use axum::response::IntoResponse;
    use leptos::prelude::*;
    use leptos_axum::{generate_route_list, LeptosRoutes};
    use docs_site::app::*;
    use tower_http::services::ServeDir;

    let conf = get_configuration(None).unwrap();
    let leptos_options = conf.leptos_options;
    let addr = leptos_options.site_addr;
    let routes = generate_route_list(App);

    // Serve map thumbnail images from the main project's public/maps/ directory
    let maps_dir = std::path::PathBuf::from(
        std::env::var("MAPS_DIR").unwrap_or_else(|_| {
            // cargo leptos runs from the crate dir, so ../public/maps works;
            // but also try the workspace root in case we're run from there.
            let crate_relative = std::path::PathBuf::from("../public/maps");
            if crate_relative.exists() {
                return crate_relative.to_string_lossy().into_owned();
            }
            "public/maps".to_string()
        }),
    );

    // Resolve public dir for static SEO files
    let public_dir = std::path::PathBuf::from(
        std::env::var("PUBLIC_DIR").unwrap_or_else(|_| {
            let leptos_public = std::path::PathBuf::from("target/site");
            if leptos_public.join("sitemap.xml").exists() {
                return leptos_public.to_string_lossy().into_owned();
            }
            "public".to_string()
        }),
    );

    let pub_dir = public_dir.clone();
    let serve_sitemap = move || {
        let path = pub_dir.join("sitemap.xml");
        async move {
            match tokio::fs::read_to_string(&path).await {
                Ok(content) => (
                    [(http::header::CONTENT_TYPE, "application/xml")],
                    content,
                ).into_response(),
                Err(_) => (
                    http::StatusCode::NOT_FOUND,
                    "sitemap.xml not found",
                ).into_response(),
            }
        }
    };

    let pub_dir2 = public_dir.clone();
    let serve_robots = move || {
        let path = pub_dir2.join("robots.txt");
        async move {
            match tokio::fs::read_to_string(&path).await {
                Ok(content) => (
                    [(http::header::CONTENT_TYPE, "text/plain")],
                    content,
                ).into_response(),
                Err(_) => (
                    http::StatusCode::NOT_FOUND,
                    "robots.txt not found",
                ).into_response(),
            }
        }
    };

    let app = Router::new()
        .route("/docs/sitemap.xml", axum::routing::get(serve_sitemap))
        .route("/docs/robots.txt", axum::routing::get(serve_robots))
        .nest_service("/docs/maps/img", ServeDir::new(&maps_dir))
        .nest_service("/docs/assets", ServeDir::new(&public_dir))
        .leptos_routes(&leptos_options, routes, {
            let leptos_options = leptos_options.clone();
            move || shell(leptos_options.clone())
        })
        .fallback(leptos_axum::file_and_error_handler(shell))
        .with_state(leptos_options);

    let listener = tokio::net::TcpListener::bind(&addr).await.unwrap();
    println!("Docs site listening on http://{}", addr);
    axum::serve(listener, app.into_make_service()).await.unwrap();
}

#[cfg(not(feature = "ssr"))]
pub fn main() {}
