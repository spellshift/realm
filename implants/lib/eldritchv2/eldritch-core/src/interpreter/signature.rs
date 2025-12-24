use alloc::string::String;
use alloc::vec::Vec;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ParameterSignature {
    pub name: String,
    pub type_name: Option<String>,
    pub is_optional: bool,
    pub is_variadic: bool, // for *args
    pub is_kwargs: bool,   // for **kwargs
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct MethodSignature {
    pub name: String,
    pub params: Vec<ParameterSignature>,
    pub return_type: Option<String>,
}
