use leptos::prelude::*;
use leptos_router::hooks::use_params_map;
use crate::components::page_head::PageHead;
use crate::data::maps;

#[component]
pub fn MapDetail() -> impl IntoView {
    let params = use_params_map();

    let map_data = Memo::new(move |_| {
        let id = params.get().get("id").unwrap_or_default();
        maps::get_map(&id).cloned()
    });

    let nav_info = Memo::new(move |_| {
        let all = maps::all_maps();
        let id = params.get().get("id").unwrap_or_default();
        let idx = all.iter().position(|m| m.id == id);
        let prev = idx.and_then(|i| if i > 0 { Some(all[i - 1].id.clone()) } else { None });
        let next = idx.and_then(|i| if i + 1 < all.len() { Some(all[i + 1].id.clone()) } else { None });
        (prev, next)
    });

    view! {
        {move || {
            if let Some(m) = map_data.get() {
                let img_src = format!("/docs/maps/img/{}.png", m.id);
                let dims = format!("{} x {}", m.width, m.height);
                let title_str: &'static str = Box::leak(format!("Map: {}", m.name).into_boxed_str());
                let desc_str: &'static str = Box::leak(
                    format!("{} — a {}x{} map for LiquidWar.io", m.name, m.width, m.height).into_boxed_str()
                );
                let path_str: &'static str = Box::leak(format!("/docs/maps/{}", m.id).into_boxed_str());
                let name_str: &'static str = Box::leak(m.name.clone().into_boxed_str());
                let breadcrumbs: &'static [(&'static str, &'static str)] = Box::leak(
                    vec![("Home", "/docs"), ("Maps", "/docs/maps"), (name_str, path_str)].into_boxed_slice()
                );
                let (prev, next) = nav_info.get();

                view! {
                    <PageHead title=title_str description=desc_str path=path_str breadcrumbs=breadcrumbs />
                    <article class="page">
                        <div class="map-detail-nav">
                            <a href="/docs/maps" class="back-link">"Back to Maps"</a>
                            <div class="prev-next">
                                {prev.map(|p| view! { <a href=format!("/docs/maps/{}", p) class="nav-arrow">"Prev"</a> })}
                                {next.map(|n| view! { <a href=format!("/docs/maps/{}", n) class="nav-arrow">"Next"</a> })}
                            </div>
                        </div>

                        <h1>{m.name.clone()}</h1>

                        <div class="map-detail-preview">
                            <img src=img_src alt=m.name.clone() />
                        </div>

                        <table class="info-table">
                            <tbody>
                                <tr><td class="label">"ID"</td><td>{m.id.clone()}</td></tr>
                                <tr><td class="label">"Dimensions"</td><td>{dims}</td></tr>
                                {m.author.as_ref().map(|a| view! {
                                    <tr><td class="label">"Author"</td><td>{a.clone()}</td></tr>
                                })}
                                {m.description.as_ref().map(|d| view! {
                                    <tr><td class="label">"Description"</td><td>{d.clone()}</td></tr>
                                })}
                                {m.license.as_ref().map(|l| view! {
                                    <tr><td class="label">"License"</td><td>{l.clone()}</td></tr>
                                })}
                            </tbody>
                        </table>

                        <div class="map-detail-actions">
                            <a href="/" class="cta-btn">"Play This Map"</a>
                        </div>
                    </article>
                }.into_any()
            } else {
                view! {
                    <article class="page">
                        <h1>"Map Not Found"</h1>
                        <p>"The requested map could not be found."</p>
                        <a href="/docs/maps">"Back to Maps"</a>
                    </article>
                }.into_any()
            }
        }}
    }
}
