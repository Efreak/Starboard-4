pub mod id;
pub mod server_list;

use leptos::*;
use leptos_router::*;
#[cfg(feature = "ssr")]
use std::sync::Arc;

#[cfg(feature = "ssr")]
use crate::auth::context::Guilds;

use super::UserRes;

#[cfg(feature = "ssr")]
pub async fn get_manageable_guilds(cx: Scope) -> Option<Arc<Guilds>> {
    use std::collections::HashMap;

    use twilight_model::guild::Permissions;

    use crate::auth::context::AuthContext;

    let acx = AuthContext::get(cx)?;

    if let Some(guilds) = acx.guilds.lock().unwrap().clone() {
        return Some(guilds);
    }

    let guilds: Arc<HashMap<_, _>> = Arc::new(
        acx.http
            .current_user_guilds()
            .await
            .ok()?
            .models()
            .await
            .ok()?
            .into_iter()
            .filter(|g| g.permissions.contains(Permissions::ADMINISTRATOR))
            .map(|g| (g.id, g))
            .collect(),
    );

    *acx.guilds.lock().unwrap() = Some(guilds.clone());

    Some(guilds)
}

#[component]
pub fn Servers(cx: Scope) -> impl IntoView {
    let user = expect_context::<UserRes>(cx);

    let red = move || {
        user.with(cx, |u| {
            if u.is_err() {
                create_effect(cx, |_| {
                    window().location().assign("/api/redirect").unwrap();
                })
            }
        });
    };
    view! { cx,
        <Suspense fallback=|| ()>{red}</Suspense>
        <Outlet/>
    }
}
