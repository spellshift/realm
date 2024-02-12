mod process_list_impl;

use allocative::Allocative;
use derive_more::Display;
use serde::{Serialize, Serializer};
use starlark::collections::SmallMap;
use starlark::environment::{Methods, MethodsBuilder, MethodsStatic};
use starlark::eval::Evaluator;
use starlark::values::list::UnpackList;
use starlark::values::none::NoneType;
use starlark::values::{
    starlark_value, ProvidesStaticType, StarlarkValue, UnpackValue, Value, ValueLike,
};
use starlark::{starlark_module, starlark_simple_value};

#[derive(Copy, Clone, Debug, PartialEq, Display, ProvidesStaticType, Allocative)]
#[display(fmt = "ReportLibrary")]
pub struct ReportLibrary();
starlark_simple_value!(ReportLibrary);

#[allow(non_upper_case_globals)]
#[starlark_value(type = "report_library")]
impl<'v> StarlarkValue<'v> for ReportLibrary {
    fn get_methods() -> Option<&'static Methods> {
        static RES: MethodsStatic = MethodsStatic::new();
        RES.methods(methods)
    }
}

impl Serialize for ReportLibrary {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_none()
    }
}

impl<'v> UnpackValue<'v> for ReportLibrary {
    fn expected() -> String {
        ReportLibrary::get_type_value_static().as_str().to_owned()
    }

    fn unpack_value(value: Value<'v>) -> Option<Self> {
        Some(*value.downcast_ref::<ReportLibrary>().unwrap())
    }
}

// This is where all of the "report.X" impl methods are bound
#[starlark_module]
#[rustfmt::skip]
#[allow(clippy::needless_lifetimes, clippy::type_complexity, clippy::too_many_arguments)]
fn methods(builder: &mut MethodsBuilder) {
    fn process_list(this: ReportLibrary, starlark_eval: &mut Evaluator<'v, '_>, process_list: UnpackList<SmallMap<String, Value>>) -> anyhow::Result<NoneType> {
        if false { println!("Ignore unused this var. _this isn't allowed by starlark. {:?}", this); }
        process_list_impl::process_list(starlark_eval, process_list.items)?;
        Ok(NoneType{})
    }
}

#[cfg(test)]
mod test {
    use std::collections::HashMap;

    use crate::pb::process::Status;
    use crate::pb::{Process, ProcessList, Tome};
    use anyhow::Error;

    macro_rules! process_list_tests {
        ($($name:ident: $value:expr,)*) => {
        $(
            #[tokio::test]
            async fn $name() {
                let tc: TestCase = $value;
                let mut runtime = crate::start(tc.tome).await;
                runtime.finish().await;

                let want_err_str = match tc.want_error {
                    Some(err) => err.to_string(),
                    None => "".to_string(),
                };
                let err_str = match runtime.collect_errors().pop() {
                    Some(err) => err.to_string(),
                    None => "".to_string(),
                };
                assert_eq!(want_err_str, err_str);
                assert_eq!(tc.want_output, runtime.collect_text().join(""));
                assert_eq!(Some(tc.want_proc_list), runtime.collect_process_lists().pop());
            }
        )*
        }
    }

    struct TestCase {
        pub tome: Tome,
        pub want_output: String,
        pub want_error: Option<Error>,
        pub want_proc_list: ProcessList,
    }

    process_list_tests! {
            one_process: TestCase{
                tome: Tome{
                    eldritch: String::from(r#"report.process_list([{"pid":5,"ppid":101,"name":"test","username":"root","path":"/bin/cat","env":"COOL=1","command":"cat","cwd":"/home/meow","status":"IDLE"}])"#),
                    parameters: HashMap::new(),
                    file_names: Vec::new(),
                },
                want_proc_list: ProcessList{list: vec![
                    Process{
                        pid: 5,
                        ppid: 101,
                        name: "test".to_string(),
                        principal: "root".to_string(),
                        path: "/bin/cat".to_string(),
                        env: "COOL=1".to_string(),
                        cmd: "cat".to_string(),
                        cwd: "/home/meow".to_string(),
                        status: Status::Idle.into(),
                    },
                ]},
                want_output: String::from(""),
                want_error: None,
            },
    }
}
