use crate::errors::ElangError;
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

use crate::interpreter::env::{Environment, Value};

pub fn create_instance(
    class_name: String,
    default_fields: &HashMap<String, Value>,
    methods: &HashMap<String, Value>,
) -> Value {
    let fields = Rc::new(RefCell::new(default_fields.clone()));
    Value::Instance {
        class_name,
        fields,
        methods: methods.clone(),
    }
}

pub fn bind_method(
    method: &Value,
    instance: &Value,
) -> Result<Value, ElangError> {
    if let Value::Fn {
        name,
        params,
        body,
        is_async,
        is_pure,
        closure,
    } = method
    {
        let mut bound_closure = Environment::child(closure);
        bound_closure.declare("self".to_string(), instance.clone())?;
        Ok(Value::Fn {
            name: name.clone(),
            params: params.clone(),
            body: body.clone(),
            is_async: *is_async,
            is_pure: *is_pure,
            closure: bound_closure,
        })
    } else {
        Ok(method.clone())
    }
}
