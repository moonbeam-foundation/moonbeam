#[test]
fn tests() {
	let t = trybuild::TestCases::new();
	t.pass("tests/01-parse-valid-toto.rs");
	//t.compile_fail("tests/02-parse-non-lit.rs");
}
