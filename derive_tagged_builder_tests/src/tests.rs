use derive_tagged_builder::TaggedBuilder;
use expect_test::expect;

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

    expect![[r#"
        Foo {
            bar: true,
            baz: [],
        }
    "#]]
    .assert_debug_eq(&FooBuilder::new().bar(true).baz(Vec::new()).build())
}
