use std::collections::HashMap;

use tera;
use tera::{to_value, from_value, Value};

pub fn split(value: Value, args: HashMap<String, Value>) -> Result<Value, tera::Error> {
    let arr = match tera::from_value::<Vec<Value>>(value.clone()) {
        Ok(v) => v,
        Err(_) => return Err(format!("Invalid type. Value {} is not of type Vec<Value>", &value).into())
    };
    let idx = if arr.len() % 2 == 0 { arr.len() / 2 } else { (arr.len() / 2) + 1 };
    let (l, r) = arr.split_at(idx);
    Ok(to_value(vec![l, r])?)
}
