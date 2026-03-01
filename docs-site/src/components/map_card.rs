use leptos::prelude::*;

#[component]
pub fn MapCard(
    id: String,
    name: String,
    #[prop(default = String::new())] author: String,
) -> impl IntoView {
    let href = format!("/docs/maps/{}", id);
    let img_src = format!("/docs/maps/img/{}.png", id);
    let has_author = !author.is_empty();

    view! {
        <a href=href class="map-card">
            <img src=img_src alt=name.clone() loading="lazy" />
            <div class="map-card-info">
                <span class="map-card-name">{name}</span>
                {if has_author {
                    Some(view! { <span class="map-card-author">{author}</span> })
                } else {
                    None
                }}
            </div>
        </a>
    }
}
