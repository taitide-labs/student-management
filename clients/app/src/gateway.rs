use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use student_management_gateway::{contract::GatewayClient, mock::MockGateway};

thread_local! {
    static ACTIVE_GATEWAY: RefCell<GatewayClient> =
        RefCell::new(GatewayClient::new(Rc::new(RefCell::new(MockGateway::seeded()))));
    static GATEWAY_CONFIGURED: Cell<bool> = const { Cell::new(false) };
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
