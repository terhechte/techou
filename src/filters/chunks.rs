use std::collections::HashMap;

use tera::{self, to_value, Filter, Result, Value};

pub struct Chunk;

impl Filter for Chunk {
    fn filter(&self, value: &Value, args: &HashMap<String, Value>) -> Result<Value> {
        let arr = tera::from_value::<Vec<Value>>(value.clone())?;
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
}
