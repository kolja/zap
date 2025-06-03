use std::collections::HashMap;
use tera::{Function as TeraFunction, Result as TeraResult, Value, to_value};

struct ShoutFunction;
impl TeraFunction for ShoutFunction {
    fn call(&self, args: &HashMap<String, Value>) -> TeraResult<Value> {
        let input_val = args.get("input")
            .ok_or_else(|| tera::Error::msg("Function `shout` requires an `input` argument"))?;
        let input_str = input_val.as_str()
            .ok_or_else(|| tera::Error::msg("`input` argument for `shout` must be a string"))?;
        Ok(to_value(format!("{}!!!", input_str.to_uppercase()))
            .map_err(|e| tera::Error::chain("Failed to convert result to Value", e))?)
    }
    fn is_safe(&self) -> bool {
        true
    }
}

#[unsafe(no_mangle)]
pub unsafe extern "C" fn register_tera_custom_functions(tera: &mut tera::Tera) {
    tera.register_function("shout", *Box::new(ShoutFunction));
    // tera.register_function("another_one", Box::new(AnotherFunc));
}
