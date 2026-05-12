use crate::ast::*;
use crate::errors::ElangError;
use crate::interpreter::env::{Control, Environment, Value};
use std::cell::RefCell;
use std::collections::HashMap;
use std::rc::Rc;

fn execute_block(stmts: &[Statement], env: &mut Environment) -> Result<Control, ElangError> {
    for stmt in stmts {
        let ctrl = super::env::exec_stmt(stmt, env)?;
        match ctrl {
            Control::Normal => {}
            other => return Ok(other),
        }
    }
    Ok(Control::Normal)
}
