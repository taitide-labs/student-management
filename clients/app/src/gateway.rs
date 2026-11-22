use std::{
    cell::{Cell, RefCell},
    rc::Rc,
    sync::RwLock,
};

use student_management_gateway::{contract::GatewayClient, mock::MockGateway};

static REMOTE_BASE_URL: RwLock<Option<String>> = RwLock::new(None);

thread_local! {
    static ACTIVE_GATEWAY: RefCell<GatewayClient> =
        RefCell::new(GatewayClient::new(Rc::new(RefCell::new(MockGateway::seeded()))));
    static GATEWAY_CONFIGURED: Cell<bool> = const { Cell::new(false) };
}

pub fn set_remote_base_url(base_url: &str) {
    if let Ok(mut guard) = REMOTE_BASE_URL.write() {
        *guard = Some(base_url.trim().to_owned());
    }
}

#[must_use]
pub fn remote_base_url() -> Option<String> {
    REMOTE_BASE_URL
        .read()
        .ok()
        .and_then(|guard| guard.clone())
        .filter(|url| !url.is_empty())
}

pub fn install(client: GatewayClient) {
    ACTIVE_GATEWAY.with(|slot| *slot.borrow_mut() = client);
    GATEWAY_CONFIGURED.with(|flag| flag.set(true));
}

#[must_use]
pub fn is_configured() -> bool {
    GATEWAY_CONFIGURED.with(Cell::get)
}

#[must_use]
pub fn client() -> GatewayClient {
    ACTIVE_GATEWAY.with(|slot| slot.borrow().clone())
}
