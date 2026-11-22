use nsh_base::{nav_page, nav_stack};
use nsh_core::prelude::*;

pub fn stacked_shell<D>(
    root_title: impl IntoReactive<SharedString>,
    root_body: impl IntoElement,
    detail_stack: impl Fn() -> Vec<String> + 'static,
    detail_title: impl Fn(&str) -> String + Clone + 'static,
    on_dismiss_depth: impl Fn(usize) + Clone + 'static,
    render_detail: impl Fn(String) -> D + Clone + 'static,
) -> impl IntoElement
where
    D: IntoElement + 'static,
{
    nav_stack()
        .flex_1()
        .child(nav_page().title(root_title).child(root_body))
        .child(each(
            detail_stack,
            |key: &String| key.clone(),
            move |key, depth| {
                let target = key.get_untracked();
                let dismiss = on_dismiss_depth.clone();
                nav_page()
                    .title(detail_title(&target))
                    .on_dismiss(move || dismiss(depth.get_untracked()))
                    .child(render_detail(target))
            },
        ))
}
