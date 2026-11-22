use std::{cell::RefCell, rc::Rc};

use student_management_gateway::{contract::GatewayClient, mock::MockGateway};

thread_local! {
    static ACTIVE_GATEWAY: RefCell<GatewayClient> =
        RefCell::new(GatewayClient::new(Rc::new(RefCell::new(MockGateway::seeded()))));
}

pub fn install(client: GatewayClient) {
    ACTIVE_GATEWAY.with(|slot| *slot.borrow_mut() = client);
}

#[must_use]
pub fn client() -> GatewayClient {
    ACTIVE_GATEWAY.with(|slot| slot.borrow().clone())
}
