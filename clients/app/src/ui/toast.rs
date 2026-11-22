use std::{cell::RefCell, time::Duration};

use nsh_base::{
    Anchor, ToastManager, ToastMotion, ToastOptions, ToastStackState, ToastTransitionStatus, theme,
    toast, toast_stack,
};
use nsh_core::{prelude::*, time::now};

use crate::ui::{motion, token};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ToastTone {
    Success,
    Error,
}

#[derive(Clone, PartialEq)]
struct ToastPayload {
    message: SharedString,
    tone: ToastTone,
}

#[derive(Clone, Default)]
struct PendingQueue(Vec<(SharedString, ToastTone)>);

fn queue() -> Signal<PendingQueue> {
    global::<PendingQueue>()
}

pub fn push_toast(message: impl Into<SharedString>, tone: ToastTone) {
    let msg = message.into();
    queue().update(|q| q.0.push((msg, tone)));
}

#[component]
pub fn ToastHost() -> impl IntoElement {
    let manager = stored(RefCell::new(ToastManager::<u64, ToastPayload>::new(
        ToastMotion::sonner(),
    )));
    let next_id = stored(RefCell::new(1_u64));
    let visible = signal(Vec::<(u64, ToastPayload, ToastTransitionStatus)>::new());
    let state = ToastStackState::new();

    let sync_visible = move || {
        let _ = visible.try_set(manager.with(|m| {
            m.borrow()
                .visible(3)
                .map(|(id, payload, status)| (*id, payload.clone(), status))
                .collect()
        }));
    };

    effect(move || {
        let pending = queue().with(|q| q.0.clone());
        if pending.is_empty() {
            return;
        }
        queue().update(|q| q.0.clear());
        let ts = now();
        for (message, tone) in pending {
            let id = next_id.with(|n| {
                let mut guard = n.borrow_mut();
                let current = *guard;
                *guard += 1;
                current
            });
            manager.with(|m| {
                m.borrow_mut().push(
                    id,
                    ToastPayload { message, tone },
                    ToastOptions::timeout(Some(Duration::from_secs(4))),
                    ts,
                );
            });
        }
        sync_visible();
    });

    set_interval(Duration::from_millis(80), move || {
        if manager.with(|m| m.borrow_mut().advance(now(), state.is_expanded()).changed) {
            sync_visible();
        }
    });

    overlay()
        .absolute()
        .bottom_4()
        .right_4()
        .p_safe_area()
        .child(
            toast_stack()
                .placement(Anchor::BottomRight)
                .state(state)
                .w(px(280.0))
                .child(each(
                    visible,
                    |(id, _, _)| *id,
                    move |item, _| {
                        let payload = move || item.with(|row| row.1.clone());
                        toast()
                            .transition_status(move || item.with(|row| row.2))
                            .accessibility_label(move || Some(payload().message))
                            .on_dismiss_request(move || {
                                let id = item.with_untracked(|row| row.0);
                                manager.with(|m| m.borrow_mut().dismiss(&id, now()));
                                sync_visible();
                            })
                            .rounded(token::radius_card)
                            .border_1()
                            .border_color(move || match payload().tone {
                                ToastTone::Success => token::success().opacity(0.4),
                                ToastTone::Error => theme().destructive.opacity(0.45),
                            })
                            .bg(move || theme().popover)
                            .px_3p5()
                            .py_2p5()
                            .shadow_lg()
                            .when(!reduce_motion(), |t| {
                                t.transition_transform()
                                    .duration(motion::FAST_MS)
                                    .ease_out()
                            })
                            .child(
                                text(move || payload().message)
                                    .text_sm()
                                    .text_color(move || theme().popover_foreground),
                            )
                    },
                )),
        )
}
