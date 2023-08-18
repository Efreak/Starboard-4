mod api;
pub mod behavior;
pub mod regex;
pub mod requirements;
pub mod style;

use behavior::Behavior;
use regex::Regex;
use requirements::Requirements;
use style::Style;

use leptos::*;
use leptos_router::*;
use twilight_model::id::Id;

use crate::site::{
    components::{toast, FullScreenPopup, Toast},
    routes::servers::id::components::get_flat_guild,
};

use super::get_starboard;

#[derive(Clone, Copy, PartialEq)]
pub struct StarboardId(pub i32);
pub type StarboardIdContext = Memo<Option<StarboardId>>;

#[derive(Params, PartialEq, Clone)]
struct Props {
    starboard_id: i32,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tab {
    Behavior,
    Regex,
    Requirements,
    Style,
}

impl Tab {
    pub fn as_str(&self) -> &'static str {
        match self {
            Self::Behavior => "Behavior",
            Self::Regex => "Regex",
            Self::Requirements => "Requirements",
            Self::Style => "Style",
        }
    }
}

#[component]
pub fn Starboard(cx: Scope) -> impl IntoView {
    let update_sb = create_server_action::<self::api::UpdateStarboard>(cx);
    let errs = create_memo(cx, move |_| match update_sb.value().get() {
        Some(Ok(v)) => v,
        _ => Default::default(),
    });

    let current_tab = create_rw_signal(cx, Tab::Requirements);

    let params = use_params::<Props>(cx);
    let sb_id: StarboardIdContext = create_memo(cx, move |_| {
        params.with(|p| p.as_ref().ok().map(|p| StarboardId(p.starboard_id)))
    });
    provide_context(cx, sb_id);

    let get_sb = move |cx| {
        let Ok(params) = params.get() else {
            return None;
        };
        get_starboard(cx, params.starboard_id)
    };
    let get_title = move |cx| {
        let Some(sb) = get_sb(cx) else {
            return "".to_string();
        };

        let channel = 'out: {
            let Some(guild) = get_flat_guild(cx) else {
                break 'out "unknown channel".to_string();
            };
            match guild.channels.get(&Id::new(sb.channel_id as _)) {
                Some(channel) => match &channel.name {
                    Some(n) => format!("#{n}"),
                    None => "unknown channel".to_string(),
                },
                None => "deleted channel".to_string(),
            }
        };

        format!("Starboard '{}' in {}", sb.name, channel)
    };
    let make_is_hidden = move |tab: Tab| create_memo(cx, move |_| tab != current_tab.get());

    create_effect(cx, move |_| {
        match update_sb.value().get() {
            Some(Ok(errs)) => {
                if errs.is_empty() {
                    toast(cx, Toast::info("Settings saved."));
                } else {
                    toast(
                        cx,
                        Toast::warning(
                            "Some settings were saved, but there were some errors as well.",
                        ),
                    );
                }
            }
            Some(Err(e)) => toast(cx, Toast::error(e)),
            None => (),
        };
    });

    view! { cx,
        <Suspense fallback=|| ()>
            <ActionForm action=update_sb>
                <FullScreenPopup
                    title=move || get_title(cx)
                    actions=move || {
                        view! { cx,
                            <div class="btn btn-outline btn-error">"Delete"</div>
                            <div class="flex-1"></div>
                            <A href=".." class="btn btn-ghost">
                                "Cancel"
                            </A>
                            <button
                                type="submit"
                                class="btn btn-primary"
                                class=("btn-disabled", update_sb.pending().get())
                            >
                                <Show when=move || update_sb.pending().get() fallback=|_| ()>
                                    <span class="loading loading-spinner loading-sm"></span>
                                </Show>
                                "Save"
                            </button>
                        }
                    }
                >

                    <ul class="menu menu-horizontal flex space-x-1">
                        <TabButton tab=Tab::Requirements sig=current_tab/>
                        <TabButton tab=Tab::Behavior sig=current_tab/>
                        <TabButton tab=Tab::Style sig=current_tab/>
                        <TabButton tab=Tab::Regex sig=current_tab/>
                    </ul>
                    {move || {
                        let Some(sb) = get_sb(cx) else { return None;
                    };
                        let tview =
                        view! { cx,
                            <input type="hidden" name="guild_id" value=sb.guild_id.to_string()/>
                            <input type="hidden" name="starboard_id" value=sb.id.to_string()/>

                            <Behavior errs=errs sb=sb.clone() hidden=make_is_hidden(Tab::Behavior)/>
                            <Regex errs=errs sb=sb.clone() hidden=make_is_hidden(Tab::Regex)/>
                            <Requirements
                                errs=errs
                                sb=sb.clone()
                                hidden=make_is_hidden(Tab::Requirements)
                            />
                            <Style errs=errs sb=sb.clone() hidden=make_is_hidden(Tab::Style)/>
                        };
                        Some(tview)
                    }}

                </FullScreenPopup>
            </ActionForm>
        </Suspense>
    }
}

#[component]
pub fn TabButton(cx: Scope, tab: Tab, sig: RwSignal<Tab>) -> impl IntoView {
    view! { cx,
        <li>
            <button
                on:click=move |_| sig.set(tab)
                class=move || if sig.get() == tab { "active" } else { "" }
                type="button"
            >
                {tab.as_str()}
            </button>
        </li>
    }
}
