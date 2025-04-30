#[derive(Debug)]
struct FooBuilder<Bar, Baz> {
    bar: Option<bool>,
    baz: Option<Vec<u8>>,
}
impl Default
for FooBuilder<
    ::derive_tagged_builder::UnspecifiedProperty,
    ::derive_tagged_builder::UnspecifiedProperty,
> {
    fn default() -> Self {
        Self { bar: None, baz: None }
    }
}
impl FooBuilder<
    ::derive_tagged_builder::UnspecifiedProperty,
    ::derive_tagged_builder::UnspecifiedProperty,
> {
    pub fn new() -> Self {
        Self::default()
    }
}
impl Foo {
    pub fn builder() -> FooBuilder<
        ::derive_tagged_builder::UnspecifiedProperty,
        ::derive_tagged_builder::UnspecifiedProperty,
    > {
        FooBuilder::new()
    }
}
impl FooBuilder<
    ::derive_tagged_builder::SpecifiedProperty,
    ::derive_tagged_builder::SpecifiedProperty,
> {
    pub fn build(self) -> Foo {
        Foo {
            bar: self
                .bar
                .expect(
                    concat!(
                        "Should not be able to call build when ", stringify!(bar),
                        " has not been set!"
                    ),
                ),
            baz: self
                .baz
                .expect(
                    concat!(
                        "Should not be able to call build when ", stringify!(baz),
                        " has not been set!"
                    ),
                ),
        }
    }
}
impl<Baz> FooBuilder<::derive_tagged_builder::UnspecifiedProperty, Baz> {
    pub fn bar(
        self,
        value: impl Into<bool>,
    ) -> FooBuilder<::derive_tagged_builder::SpecifiedProperty, Baz> {
        FooBuilder {
            baz,
            bar: Some(value.into()),
        }
    }
    pub fn set_bar(
        &mut self,
        value: impl Into<bool>,
    ) -> FooBuilder<::derive_tagged_builder::SpecifiedProperty, Baz> {
        FooBuilder {
            baz,
            bar: Some(value.into()),
        }
    }
    pub fn try_set_bar<ValueInto: TryInto<bool>>(
        &mut self,
        value: ValueInto,
    ) -> Result<(), ValueInto::Err> {
        Ok(FooBuilder {
            baz,
            bar: Some(value.try_into()?),
        })
    }
}
impl<Bar> FooBuilder<Bar, ::derive_tagged_builder::UnspecifiedProperty> {
    pub fn baz(
        self,
        value: impl Into<Vec<u8>>,
    ) -> FooBuilder<Bar, ::derive_tagged_builder::SpecifiedProperty> {
        FooBuilder {
            bar,
            baz: Some(value.into()),
        }
    }
    pub fn set_baz(
        &mut self,
        value: impl Into<Vec<u8>>,
    ) -> FooBuilder<Bar, ::derive_tagged_builder::SpecifiedProperty> {
        FooBuilder {
            bar,
            baz: Some(value.into()),
        }
    }
    pub fn try_set_baz<ValueInto: TryInto<Vec<u8>>>(
        &mut self,
        value: ValueInto,
    ) -> Result<(), ValueInto::Err> {
        Ok(FooBuilder {
            bar,
            baz: Some(value.try_into()?),
        })
    }
}
