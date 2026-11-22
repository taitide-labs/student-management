use axum::Router;

use crate::state::AppState;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SliceStatus {
    Planned,
    Active,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
}

pub struct SliceDescriptor {
    pub context: &'static str,
    pub slice: &'static str,
    pub method: HttpMethod,
    pub path: &'static str,
    pub status: SliceStatus,
    pub route: fn() -> Router<AppState>,
}

inventory::collect!(SliceDescriptor);

#[must_use]
pub fn registered_slices() -> Vec<&'static SliceDescriptor> {
    inventory::iter::<SliceDescriptor>().collect()
}

pub fn build_context_router(context: &str, state: AppState) -> Router {
    let mut router = Router::new();
    for descriptor in inventory::iter::<SliceDescriptor> {
        if descriptor.context == context {
            router = router.merge((descriptor.route)());
        }
    }
    router.with_state(state)
}
