use pyo3::{
    types::{PyDict, PyList},
    Python,
};
use serde::Serialize;
use shared::ecu_hal::EcuConfig;

pub fn list_from_array<T: Serialize>(py: Python, list: T) -> &PyList {
    let binding = serde_json::to_value(&list).expect("Failed to serialize list");
    let values = binding.as_array().unwrap();

    let list = PyList::empty(py);
    for value in values {
        if value.is_object() {
            list.append(dict_from_obj(py, value)).unwrap();
        } else if value.is_array() {
            list.append(list_from_array(py, value)).unwrap();
        } else if value.is_i64() {
            list.append(value.as_i64().unwrap()).unwrap();
        } else if value.is_number() {
            list.append(value.as_f64().unwrap()).unwrap();
        } else if value.is_string() {
            list.append(value.as_str().unwrap()).unwrap();
        } else if value.is_boolean() {
            list.append(value.as_bool().unwrap()).unwrap();
        } else if !value.is_null() {
            panic!("Unsupported type {:?} for list ({:?}", value, list);
        }
    }

    list
}

pub fn dict_from_obj<T: Serialize>(py: Python, obj: T) -> &PyDict {
    let binding = serde_json::to_value(&obj).expect("Failed to serialize object");
    let values = binding.as_object().unwrap();

    let dict = PyDict::new(py);
    for (key, value) in values {
        if value.is_object() {
            dict.set_item(key, dict_from_obj(py, value)).unwrap();
        } else if value.is_array() {
            dict.set_item(key, list_from_array(py, value)).unwrap();
        } else if value.is_i64() {
            dict.set_item(key, value.as_i64().unwrap()).unwrap();
        } else if value.is_number() {
            dict.set_item(key, value.as_f64().unwrap()).unwrap();
        } else if value.is_string() {
            dict.set_item(key, value.as_str().unwrap()).unwrap();
        } else if value.is_boolean() {
            dict.set_item(key, value.as_bool().unwrap()).unwrap();
        } else if !value.is_null() {
            panic!(
                "Unsupported type {:?} for key {:?} ({:?})",
                value, key, dict
            );
        }
    }

    dict
}

pub fn obj_from_dict<T: serde::de::DeserializeOwned>(dict: &PyDict) -> T {
    println!("This one: {:?}", serde_json::to_string(&EcuConfig::default()).unwrap());

    let json_str = dict.to_string()
        .replace("'", "\"")
        .replace(": True", ": true")
        .replace(": False", ": false")
        .replace("None", "null");

    println!("{:?}", dict);
    serde_json::from_str(&json_str).expect("Failed to deserialize object")
}
