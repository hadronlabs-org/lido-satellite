use crate::tests::helpers::instantiate_wrapper;

#[test]
fn success() {
    let (result, _deps, _env) = instantiate_wrapper("lido_satellite", "untrn");
    let _response = result.unwrap();
}
