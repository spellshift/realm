mod kill_impl;
mod list_impl;
mod name_impl;

use starlark::environment::{Globals, Module};
use starlark::eval::Evaluator;
use starlark::syntax::{AstModule, Dialect};
use starlark::values::{Heap, StarlarkValue, Value, ValueError, ValueLike, ProvidesStaticType, NoSerialize};
use starlark::{starlark_type, starlark_simple_value};
use std::fmt::{self, Display, Write};
use allocative::Allocative;


#[derive(Debug, PartialEq, Eq, ProvidesStaticType, NoSerialize, Allocative)]
struct Complex {
    real: i32,
    imaginary: i32,
}
starlark_simple_value!(Complex);
impl Display for Complex {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{} + {}i", self.real, self.imaginary)
    }
}

impl<'v> StarlarkValue<'v> for Complex {
    starlark_type!("complex");

    // How we add them
    fn add(&self, rhs: Value<'v>, heap: &'v Heap)
            -> Option<anyhow::Result<Value<'v>>> {
        if let Some(rhs) = rhs.downcast_ref::<Self>() {
            Some(Ok(heap.alloc(Complex {
                real: self.real + rhs.real,
                imaginary: self.imaginary + rhs.imaginary,
            })))
        } else {
            None
        }
    }
}
