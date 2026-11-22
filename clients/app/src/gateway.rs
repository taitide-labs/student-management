use std::rc::Rc;

use nsh_core::prelude::*;
use student_management_gateway::{contract::GatewayClient, mock::MockGateway};

#[derive(Clone, Default)]
struct GatewaySlot(Option<GatewayClient>);

pub fn install(client: GatewayClient) {
    global::<GatewaySlot>().set(GatewaySlot(Some(client)));
}

#[must_use]
pub fn is_configured() -> bool {
    global::<GatewaySlot>().with_untracked(|slot| slot.0.is_some())
}

#[must_use]
pub fn client() -> GatewayClient {
    if let Some(existing) = global::<GatewaySlot>().with_untracked(|slot| slot.0.clone()) {
        return existing;
    }
    let fallback = GatewayClient::new(Rc::new(MockGateway::seeded()));
    install(fallback.clone());
    fallback
}
