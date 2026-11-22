use nsh_base::{dialog, theme};
use nsh_core::{color::black, prelude::*};

use crate::ui::{motion, surface::divider, token, typography};

pub fn modal_sheet<E: IntoElement + 'static>(
    open: Signal<bool>,
    title: impl IntoReactive<SharedString> + Clone + 'static,
    build_content: impl Fn() -> E + 'static,
) -> impl IntoElement {
    when(open, move || {
        let heading = title.clone();
        dialog()
            .presentation(ModalPresentation::FormSheet)
            .on_cancel(move || open.set(false))
            .flex_1()
            .items_center()
            .justify_center()
            .bg(black().opacity(0.45))
            .p_4()
            .child(
                view()
                    .w_full()
                    .max_w(px(420.0))
                    .p_5()
                    .gap_4()
                    .rounded(token::radius_card)
                    .border_1()
                    .border_color(move || theme().border)
                    .bg(move || theme().card)
                    .shadow_lg()
                    .when(!reduce_motion(), |v| {
                        v.transition_transform()
                            .duration(motion::BASE_MS)
                            .ease_out()
                    })
                    .child(typography::section_title(heading))
                    .child(divider())
                    .child(build_content()),
            )
    })
}
