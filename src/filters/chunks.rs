use std::collections::HashMap;

use tera;
use tera::{to_value, Value};

pub fn chunk(value: Value, args: HashMap<String, Value>) -> Result<Value, tera::Error> {
    let arr = match tera::from_value::<Vec<Value>>(value.clone()) {
        Ok(v) => v,
        Err(_) => {
            return Err(format!("Invalid type. Value {} is not of type Vec<Value>", &value).into())
        }
    };
    let size = match args.get("size") {
        Some(val) => match tera::from_value::<usize>(val.clone()) {
            Ok(v) => v,
            Err(_) => 2,
        },
        None => 2,
    };

    // Convert all the values to strings before we join them together.
    let rendered = arr.chunks(size).collect::<Vec<_>>();
    Ok(to_value(rendered)?)
}
