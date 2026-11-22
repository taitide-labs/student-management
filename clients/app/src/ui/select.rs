use nsh_base::{SelectEntry, SelectItem, select, theme};
use nsh_core::prelude::*;

use crate::ui::{platform, token};

#[derive(Debug, Clone)]
pub struct SelectOption {
    pub value: SharedString,
    pub label: SharedString,
}

impl SelectOption {
    #[must_use]
    pub fn new(value: impl Into<SharedString>, label: impl Into<SharedString>) -> Self {
        Self {
            value: value.into(),
            label: label.into(),
        }
    }
}

pub fn app_select(
    items: impl Fn() -> Vec<SelectOption> + Clone + 'static,
    selected: Signal<Option<SharedString>>,
    placeholder: &'static str,
    accessibility_label: &'static str,
) -> impl IntoElement {
    let terminal = platform::is_terminal();
    let height = if platform::is_mobile() {
        px(44.0)
    } else {
        px(32.0)
    };
    let display_items = items.clone();

    select()
        .items(move || {
            items()
                .into_iter()
                .map(|opt| SelectItem::new(opt.value, opt.label))
                .collect::<Vec<_>>()
        })
        .value(move || selected.get())
        .on_change(move |next: &Option<SharedString>| selected.set(next.clone()))
        .accessibility_label(accessibility_label)
        .offset(px(4.0))
        .popup_style(move |s| {
            s.p_1()
                .rounded(move || {
                    if terminal {
                        px(0.0)
                    } else {
                        token::radius_control()
                    }
                })
                .border_1()
                .border_color(move || theme().border)
                .bg(move || theme().popover)
                .shadow_md()
        })
        .item_class("rounded-md focus:bg-accent selected:font-medium disabled:opacity-50")
        .render_item(|item: &SelectEntry| {
            view().px_2p5().py_1p5().child(
                text(item.title.clone())
                    .text_sm()
                    .text_color(move || theme().popover_foreground),
            )
        })
        .trigger(
            view()
                .h(height)
                .px_3()
                .justify_center()
                .rounded(move || {
                    if terminal {
                        px(0.0)
                    } else {
                        token::radius_control()
                    }
                })
                .border_1()
                .border_color(move || theme().input)
                .bg(move || theme().background)
                .child(
                    text(move || {
                        let current = selected.get();
                        current
                            .and_then(|val| {
                                display_items()
                                    .into_iter()
                                    .find(|item| item.value == val)
                                    .map(|item| item.label)
                            })
                            .unwrap_or_else(|| SharedString::from(placeholder))
                    })
                    .text_sm()
                    .text_color(move || theme().foreground),
                ),
        )
}
