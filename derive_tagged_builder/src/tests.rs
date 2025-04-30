use expect_test::expect;

use crate::TaggedBuilder;

#[test]
fn test_derive() {
    #[derive(Debug, TaggedBuilder)]
    #[builder(derive(Debug))]
    struct Foo {
        #[builder(Bar)]
        bar: bool,
        #[builder(Baz)]
        baz: Vec<u8>,
    }

    expect![""].assert_debug_eq(&FooBuilder::new().bar(true).baz(Vec::new()).build())
}