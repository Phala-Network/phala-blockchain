use super::*;

#[test]
fn make_sure_do_not_store_attestation() {
    // Make sure we don't store the attestation by accident in future refactoring.
    let mut response = InitRuntimeResponse::default();
    response.attestation = Some(Attestation::default());
    let json = serde_json::to_string(&response).unwrap();
    let response: InitRuntimeResponse = serde_json::from_str(&json).unwrap();
    assert_eq!(response.attestation, None);
}
