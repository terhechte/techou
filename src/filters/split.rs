use std::collections::HashMap;

use tera::{self, to_value, Filter, Result, Value};

pub struct Split;

impl Filter for Split {
    fn filter(&self, value: &Value, _args: &HashMap<String, Value>) -> Result<Value> {
        let arr = tera::from_value::<Vec<Value>>(value.clone())?;
        let idx = if arr.len() % 2 == 0 {
            arr.len() / 2
        } else {
            (arr.len() / 2) + 1
        };
        let (l, r) = arr.split_at(idx);
        Ok(to_value(vec![l, r])?)
    }
}
