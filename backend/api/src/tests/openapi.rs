use crate::export_openapi_pretty;

#[test]
fn openapi_schema_matches_frozen_contracts_file() {
    let generated = export_openapi_pretty().expect("serialize openapi");
    let committed = include_str!("../../../../contracts/openapi.json").replace("\r\n", "\n");
    assert_eq!(generated.trim(), committed.trim());
}
