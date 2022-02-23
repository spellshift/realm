#[cfg(test)]
mod tests {
    use derive_more::Display;
    use gazebo::prelude::*;

    use starlark::environment::{GlobalsBuilder, Methods, MethodsBuilder, MethodsStatic};
    use starlark::values::{StarlarkValue, Value, UnpackValue, ValueLike};
    use starlark::{starlark_type, starlark_simple_value, starlark_module};
    use starlark::assert::Assert;


    #[test]
    fn test_value_attributes() {
        #[derive(Copy, Clone, Debug, Dupe, PartialEq, Display)]
        #[display(fmt = "{}", _0)]
        struct Bool2(bool);
        starlark_simple_value!(Bool2);

        impl<'v> StarlarkValue<'v> for Bool2 {
            starlark_type!("bool2");

            fn get_methods(&self) -> Option<&'static Methods> {
                static RES: MethodsStatic = MethodsStatic::new();
                RES.methods(methods)
            }
        }

        impl<'v> UnpackValue<'v> for Bool2 {
            fn expected() -> String {
                Bool2::get_type_value_static().as_str().to_owned()
            }

            fn unpack_value(value: Value<'v>) -> Option<Self> {
                Some(*value.downcast_ref::<Bool2>().unwrap())
            }
        }

        #[starlark_module]
        fn globals(builder: &mut GlobalsBuilder) {
            const True2: Bool2 = Bool2(true);
        }

        #[starlark_module]
        fn methods(builder: &mut MethodsBuilder) {
            fn invert1(_this: Bool2) -> String {
                Ok("blah".to_owned())
            }
        }

        let mut a = Assert::new();
        a.globals_add(globals);
        a.all_true(
            r#"
True2.invert1() == "blah"
"#,
        );
    }
}