use std::borrow::{Borrow, BorrowMut};
use std::collections::hash_map::RandomState;
use std::collections::HashMap;
use std::fs::File;
use std::io::BufReader;
use std::path::Path;

use jgraph::{transform_json, CustomizerFn, CustomizerType};
use serde_json::{from_reader, to_string_pretty, Map, Value};

const TEST_FILE_PATH: &str = "./test/entry.json";
const OTHER_FILE_PATH: &str = "./test/other_file.json";

fn parse_ref(v: Value) -> (Value, bool) {
    match v {
        Value::Object(mut o) => {
            if o.contains_key("$ref") && o.get("$ref").unwrap().is_string() {
                let ref_file = File::open(o.remove("$ref").unwrap().to_string());
                if ref_file.is_err() {
                    (Value::Object(o), true)
                } else {
                    let other_file = open_json_file(OTHER_FILE_PATH);
                    if other_file.is_some() {
                        println!("{:?}", other_file.clone().unwrap());
                        o.remove("$ref");
                        o.append(&mut other_file.unwrap());
                    }
                    (Value::Object(o), false)
                }
            } else {
                (Value::Object(o), false)
            }
        }
        _ => (Value::from(v), false),
    }
}

fn build_customizer_map_helper(
    fn_arr: Vec<CustomizerFn>,
) -> HashMap<CustomizerType, Vec<CustomizerFn>, RandomState> {
    let mut customizer_map = HashMap::new();
    customizer_map.insert(CustomizerType::Object, fn_arr);
    customizer_map
}

fn main() {
    let entry_file = open_json_file(TEST_FILE_PATH)
        .unwrap_or_else(|| panic!("An error occurred when trying to retrieve the json file"));
    let other_file = open_json_file(OTHER_FILE_PATH)
        .unwrap_or_else(|| panic!("An error occurred when trying to retrieve the json file"));
    let fn_arr = vec![CustomizerFn::new(parse_ref)];
    let customizer_map = build_customizer_map_helper(fn_arr);
    let mapped_node = transform_json(Value::Object(entry_file), &customizer_map);
    println!("{}", to_string_pretty(&other_file).unwrap());
    println!("{}", to_string_pretty(&mapped_node).unwrap());
}

fn open_json_file<P: AsRef<Path>>(path: P) -> Option<Map<String, Value>> {
    let file = File::open(path).unwrap_or_else(|error| {
        panic!("An error occurred: {:?}", error);
    });
    let reader = BufReader::new(file);

    from_reader(reader).unwrap_or_else(|_| None)
}
