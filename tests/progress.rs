#[test]
fn tests() {
    let t = trybuild::TestCases::new();
    t.pass("tests/build-tests/only_modify_marked.rs");
    t.compile_fail("tests/build-tests/require_module_name.rs");
}

#[test]
fn build_abi_types() {
    let t = trybuild::TestCases::new();
    t.pass("tests/build-tests/different_abis.rs");
}