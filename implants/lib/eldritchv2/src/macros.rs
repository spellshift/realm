#[macro_export]
macro_rules! eldritch_module {
    (
        name: $name:expr,
        functions: {
            $($func_name:expr => $func:expr),* $(,)?
        }
    ) => {
        {
            use alloc::collections::BTreeMap;
            use alloc::rc::Rc;
            use core::cell::RefCell;
            use $crate::ast::Value;

            let mut methods = BTreeMap::new();
            $(
                methods.insert(
                    $func_name.to_string(),
                    Value::NativeFunction(
                        $func_name.to_string(),
                        |args| $crate::conversion::call_stub($func, args)
                    )
                );
            )*
            Value::Dictionary(Rc::new(RefCell::new(methods)))
        }
    };
}
