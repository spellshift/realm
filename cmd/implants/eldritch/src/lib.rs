#[cfg(test)]
mod tests {
    use derive_more::Display;

    use starlark::environment::{GlobalsBuilder, Methods, MethodsBuilder, MethodsStatic};
    use starlark::values::{StarlarkValue, Value, UnpackValue, ValueLike};
    use starlark::{starlark_type, starlark_simple_value, starlark_module};
    use starlark::assert::Assert;


    #[test]
    fn test_value_attributes() {
        #[derive(Copy, Clone, Debug, PartialEq, Display)]
        #[display(fmt = "Sys")]
        struct Sys();
        starlark_simple_value!(Sys);

        impl<'v> StarlarkValue<'v> for Sys {
            starlark_type!("sys");

            fn get_methods(&self) -> Option<&'static Methods> {
                static RES: MethodsStatic = MethodsStatic::new();
                RES.methods(methods)
            }
        }

        impl<'v> UnpackValue<'v> for Sys {
            fn expected() -> String {
                Sys::get_type_value_static().as_str().to_owned()
            }

            fn unpack_value(value: Value<'v>) -> Option<Self> {
                Some(*value.downcast_ref::<Sys>().unwrap())
            }
        }

        #[starlark_module]
        fn globals(builder: &mut GlobalsBuilder) {
            const sys: Sys = Sys();
        }

        #[starlark_module]
        fn methods(builder: &mut MethodsBuilder) {
            fn exec(_this: Sys, _t: String) -> String {
                Ok("root".to_owned())
            }
        }

        let mut a = Assert::new();
        a.globals_add(globals);
        a.all_true(
            r#"
sys.exec("whoami") == "root"
"#,
        );
    }
}