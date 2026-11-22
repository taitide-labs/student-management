use student_management_infra::registered_slices;

use crate::build_app_state;

#[tokio::test]
async fn runtime_links_all_27_registered_http_slices() {
    let _state = build_app_state("sqlite::memory:")
        .await
        .expect("build runtime state");
    let slices = registered_slices();
    assert_eq!(slices.len(), 27);
}
