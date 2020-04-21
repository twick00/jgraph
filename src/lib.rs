use serde_json::{Map, Value};
use std::any::Any;
use std::collections::HashMap;

#[derive(PartialEq, Eq, Hash)]
pub enum CustomizerType {
    String,
    Null,
    Bool,
    Number,
    Array,
    Object,
}

pub struct CustomizerFn {
    mutation: fn(Value) -> (Value, bool),
}

impl CustomizerFn {
    pub fn new(mutation: fn(Value) -> (Value, bool)) -> CustomizerFn {
        CustomizerFn { mutation }
    }
    pub fn mutate(&self, v: Value) -> (Value, bool) {
        (self.mutation)(v)
    }
}

pub fn transform_json(
    mut node: Value,
    customizer_arr: &HashMap<CustomizerType, Vec<CustomizerFn>>,
) -> Value {
    let mut counter = 1;
    wrapped_transform_json(node, customizer_arr, counter)
}

fn match_customizer(v: Value, customizer_arr: &[CustomizerFn]) -> Value {
    let mut value = v;
    for customizer in customizer_arr {
        let (new_value, has_mutated) = (customizer.mutation)(value);
        value = new_value
    }
    value
}

fn wrapped_transform_json(
    mut node: Value,
    customizer_map: &HashMap<CustomizerType, Vec<CustomizerFn>>,
    mut loop_counter: i32,
) -> Value {
    let v = match node {
        Value::Null => {
            let customizer_arr = customizer_map.get(&CustomizerType::Null);
            if customizer_arr.is_some() {
                match_customizer(Value::Null, customizer_arr.unwrap())
            } else {
                Value::Null
            }
        }
        Value::Bool(b) => {
            let customizer_arr = customizer_map.get(&CustomizerType::Bool);
            if customizer_arr.is_some() {
                match_customizer(Value::Bool(b), customizer_arr.unwrap())
            } else {
                Value::Bool(b)
            }
        }
        Value::Number(n) => {
            let customizer_arr = customizer_map.get(&CustomizerType::Number);
            if customizer_arr.is_some() {
                match_customizer(Value::Number(n), customizer_arr.unwrap())
            } else {
                Value::Number(n)
            }
        }
        Value::String(s) => {
            let customizer_arr = customizer_map.get(&CustomizerType::String);
            if customizer_arr.is_some() {
                match_customizer(Value::String(s), customizer_arr.unwrap())
            } else {
                Value::String(s)
            }
        }
        Value::Array(a) => {
            let customizer_arr = customizer_map.get(&CustomizerType::Array);
            let new_value = match customizer_arr {
                Some(customizer_arr) => match_customizer(Value::Array(a), customizer_arr),
                None => Value::Array(a),
            };
            if let Value::Array(v) = new_value {
                let mut new_arr = vec![];
                for value in v {
                    new_arr.push(wrapped_transform_json(value, customizer_map, loop_counter))
                }
                Value::Array(new_arr)
            } else {
                wrapped_transform_json(new_value, customizer_map, loop_counter)
            }
        }
        Value::Object(o) => {
            let customizer_arr = customizer_map.get(&CustomizerType::Object);
            let new_value = match customizer_arr {
                Some(customizer_arr) => match_customizer(Value::Object(o), customizer_arr),
                None => Value::Object(o),
            };
            if let Value::Object(o) = new_value {
                let mut new_map = Map::new();
                for (k, v) in o {
                    new_map.insert(k, wrapped_transform_json(v, customizer_map, loop_counter));
                }
                Value::Object(new_map)
            } else {
                wrapped_transform_json(new_value, customizer_map, loop_counter)
            }
        }
    };

    v
}
