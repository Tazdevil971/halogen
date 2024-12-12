use std::collections::HashMap;

use heck::{
    ToLowerCamelCase as _, ToShoutySnakeCase as _, ToSnakeCase as _, ToUpperCamelCase as _,
};
use tera::Tera;

#[cfg(feature = "backend-c")]
pub mod c;
#[cfg(feature = "backend-cpp")]
pub mod cpp;
#[cfg(feature = "backend-rust")]
pub mod rust;

fn tera() -> Tera {
    let mut tera = Tera::default();

    fn upper_camel_case(
        v: &tera::Value,
        _args: &HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let s = tera::try_get_value!("upper_camel_case", "value", String, v);
        Ok(s.to_upper_camel_case().into())
    }

    fn lower_camel_case(
        v: &tera::Value,
        _args: &HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let s = tera::try_get_value!("lower_camel_case", "value", String, v);
        Ok(s.to_lower_camel_case().into())
    }

    fn shouty_snake_case(
        v: &tera::Value,
        _args: &HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let s = tera::try_get_value!("shouty_snake_case", "value", String, v);
        Ok(s.to_shouty_snake_case().into())
    }

    fn snake_case(
        v: &tera::Value,
        _args: &HashMap<String, tera::Value>,
    ) -> tera::Result<tera::Value> {
        let s = tera::try_get_value!("snake_case", "value", String, v);
        Ok(s.to_snake_case().into())
    }

    fn hex(v: &tera::Value, _args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        let n = tera::try_get_value!("hex", "value", u64, v);
        Ok(format!("{n:#x}").into())
    }

    fn mask(v: &tera::Value, _args: &HashMap<String, tera::Value>) -> tera::Result<tera::Value> {
        let n = tera::try_get_value!("mask", "value", u64, v);
        Ok(((1u64 << n) - 1).into())
    }

    tera.register_filter("upper_camel_case", upper_camel_case);
    tera.register_filter("lower_camel_case", lower_camel_case);
    tera.register_filter("shouty_snake_case", shouty_snake_case);
    tera.register_filter("snake_case", snake_case);
    tera.register_filter("hex", hex);
    tera.register_filter("mask", mask);

    tera
}
