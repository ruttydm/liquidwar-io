use leptos::prelude::*;
use crate::components::page_head::PageHead;
use crate::components::map_card::MapCard;
use crate::data::maps;

#[component]
pub fn MapsGallery() -> impl IntoView {
    let all = maps::all_maps();
    let authors = maps::all_authors();

    let (search, set_search) = signal(String::new());
    let (author_filter, set_author_filter) = signal(String::new());

    let filtered = Memo::new(move |_| {
        let q = search.get().to_lowercase();
        let af = author_filter.get();
        all.iter()
            .filter(|m| {
                if !q.is_empty() {
                    let matches_name = m.name.to_lowercase().contains(&q);
                    let matches_id = m.id.to_lowercase().contains(&q);
                    let matches_author = m.author.as_ref().map_or(false, |a| a.to_lowercase().contains(&q));
                    if !(matches_name || matches_id || matches_author) {
                        return false;
                    }
                }
                if !af.is_empty() {
                    if m.author.as_deref() != Some(af.as_str()) {
                        return false;
                    }
                }
                true
            })
            .cloned()
            .collect::<Vec<_>>()
    });

    view! {
        <PageHead
            title="Maps"
            description="Browse all 206 maps available in LiquidWar.io — community-created maps from Liquid War 3, 5, and 6."
            path="/docs/maps"
            breadcrumbs=&[("Home", "/docs"), ("Maps", "/docs/maps")]
        />
        <article class="page">
            <h1>"Maps"</h1>
            <p class="page-intro">
                "Browse all community-created maps. Each map offers unique terrain, chokepoints, "
                "and strategic challenges."
            </p>

            <div class="gallery-controls">
                <input
                    type="text"
                    placeholder="Search maps..."
                    class="gallery-search"
                    on:input=move |ev| set_search.set(event_target_value(&ev))
                />
                <select
                    class="gallery-filter"
                    on:change=move |ev| set_author_filter.set(event_target_value(&ev))
                >
                    <option value="">"All Authors"</option>
                    {authors.iter().map(|(name, count)| {
                        let label = format!("{} ({})", name, count);
                        let val = name.to_string();
                        view! { <option value=val>{label}</option> }
                    }).collect_view()}
                </select>
            </div>

            <p class="gallery-count">
                {move || format!("{} maps", filtered.get().len())}
            </p>

            <div class="map-gallery-grid">
                {move || filtered.get().into_iter().map(|m| {
                    view! {
                        <MapCard
                            id=m.id.clone()
                            name=m.name.clone()
                            author=m.author.clone().unwrap_or_default()
                        />
                    }
                }).collect_view()}
            </div>
        </article>
    }
}
