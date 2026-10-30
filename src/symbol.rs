use std::collections::{HashMap, HashSet};

use crate::ast::Type;

#[derive(Debug, Clone, PartialEq)]
pub struct Symbol {
    pub name: String,
    pub ty: Type,
    pub stack_offset: i32,
}

#[derive(Debug)]
pub struct SymbolTable {
    scopes: Vec<HashMap<String, Symbol>>,
    current_offset: i32,
    reserved_builtins: HashSet<String>,
}

impl SymbolTable {
    pub fn new() -> Self {
        let mut reserved = HashSet::new();
        for builtin in &["putint", "putstr", "put", "getint", "getstr", "get"] {
            reserved.insert(builtin.to_string());
        }

        Self {
            scopes: vec![HashMap::new()],
            current_offset: -4,
            reserved_builtins: reserved,
        }
    }

    pub fn enter_scope(&mut self) {
        self.scopes.push(HashMap::new());
    }

    pub fn exit_scope(&mut self) {
        if self.scopes.len() > 1 {
            self.scopes.pop();
        }
    }

    pub fn is_builtin(&self, name: &str) -> bool {
        self.reserved_builtins.contains(name)
    }

    pub fn declare(&mut self, name: &str, ty: Type) -> Result<Symbol, String> {
        if self.is_builtin(name) {
            return Err(format!(
                "Cannot redeclare reserved built-in function '{}'",
                name
            ));
        }

        let current = self.scopes.last_mut().unwrap();
        if current.contains_key(name) {
            return Err(format!(
                "Identifier '{}' already declared in this scope",
                name
            ));
        }

        let offset = self.current_offset;
        self.current_offset -= 4;

        let sym = Symbol {
            name: name.to_string(),
            ty,
            stack_offset: offset,
        };

        current.insert(name.to_string(), sym.clone());
        Ok(sym)
    }

    pub fn lookup(&self, name: &str) -> Option<&Symbol> {
        for scope in self.scopes.iter().rev() {
            if let Some(sym) = scope.get(name) {
                return Some(sym);
            }
        }
        None
    }

    pub fn total_stack_bytes(&self) -> usize {
        (-self.current_offset - 4).unsigned_abs() as usize
    }

    pub fn reset_offsets(&mut self) {
        self.current_offset = -4;
    }
}
