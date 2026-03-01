use leptos::prelude::*;
use leptos_meta::*;

const BASE_URL: &str = "https://liquidwar.io";

#[component]
pub fn PageHead(
    title: &'static str,
    description: &'static str,
    path: &'static str,
    #[prop(default = &[])] breadcrumbs: &'static [(&'static str, &'static str)],
    #[prop(default = false)] is_home: bool,
) -> impl IntoView {
    let full_title = format!("{} — LiquidWar.io Docs", title);
    let canonical = format!("{}{}", BASE_URL, path);
    let og_image = format!("{}/og-image.png", BASE_URL);

    // Build BreadcrumbList JSON-LD
    let breadcrumb_ld = if !breadcrumbs.is_empty() {
        let items: Vec<String> = breadcrumbs
            .iter()
            .enumerate()
            .map(|(i, (name, url))| {
                format!(
                    r#"{{"@type":"ListItem","position":{},"name":"{}","item":"{}{}"}}"#,
                    i + 1,
                    name,
                    BASE_URL,
                    url
                )
            })
            .collect();
        Some(format!(
            r#"<script type="application/ld+json">{{"@context":"https://schema.org","@type":"BreadcrumbList","itemListElement":[{}]}}</script>"#,
            items.join(",")
        ))
    } else {
        None
    };

    // WebSite schema for home page only
    let website_ld = if is_home {
        Some(format!(
            r#"<script type="application/ld+json">{{"@context":"https://schema.org","@type":"WebSite","name":"LiquidWar.io Docs","url":"{}/docs","description":"{}"}}</script>"#,
            BASE_URL, description
        ))
    } else {
        None
    };

    view! {
        <Title text=full_title.clone() />
        <Meta name="description" content=description />
        <Meta name="robots" content="index, follow" />
        <Link rel="canonical" href=canonical.clone() />

        // Open Graph
        <Meta property="og:type" content="website" />
        <Meta property="og:site_name" content="LiquidWar.io" />
        <Meta property="og:locale" content="en_US" />
        <Meta property="og:title" content=full_title.clone() />
        <Meta property="og:description" content=description />
        <Meta property="og:url" content=canonical />
        <Meta property="og:image" content=og_image.clone() />
        <Meta property="og:image:width" content="1200" />
        <Meta property="og:image:height" content="630" />

        // Twitter Card
        <Meta name="twitter:card" content="summary_large_image" />
        <Meta name="twitter:title" content=full_title />
        <Meta name="twitter:description" content=description />
        <Meta name="twitter:image" content=og_image />

        // Structured data (injected as raw HTML)
        {breadcrumb_ld.map(|ld| view! { <div inner_html=ld style="display:none" /> })}
        {website_ld.map(|ld| view! { <div inner_html=ld style="display:none" /> })}
    }
}
