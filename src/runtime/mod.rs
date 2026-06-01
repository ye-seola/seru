use std::{
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
};

use anyhow::{Context, Ok};

use crate::{
    assets::AssetProvider,
    core::Value,
    language::ast::{self, ComponentDeclaration},
    render::{self, registry::NativeComponentRegistry},
    runtime::{env::ScopeRef, utils::check_integer},
};

mod builtins;
mod env;
pub mod utils;

pub struct Runtime {
    root_scope: env::ScopeRef,
    components: HashMap<String, ComponentDeclaration>,
    native_component_registery: NativeComponentRegistry,
}

impl Runtime {
    pub fn new(asset_provider: Box<dyn AssetProvider>) -> Runtime {
        Runtime {
            root_scope: Rc::new(RefCell::new(env::Scope::new(None))),
            components: HashMap::new(),
            native_component_registery: NativeComponentRegistry::new(asset_provider),
        }
    }

    pub fn set(&mut self, name: &str, value: Value) {
        self.root_scope.borrow_mut().set(name, value);
    }

    pub fn get(&self, name: &str) -> Option<Value> {
        self.root_scope.borrow().get(name)
    }

    pub fn build_render_tree(
        &mut self,
        name: &str,
        args: HashMap<String, Value>,
    ) -> anyhow::Result<Option<render::RenderNode>> {
        let nodes = self.render_component(name, args, &self.root_scope)?;
        let nodes_len = nodes.len();

        if nodes_len <= 0 {
            Ok(None)
        } else if nodes_len > 1 {
            anyhow::bail!("component root must be have one node");
        } else {
            Ok(nodes.get(0).cloned())
        }
    }

    fn render_component(
        &self,
        name: &str,
        args: HashMap<String, Value>,
        scope: &ScopeRef,
    ) -> anyhow::Result<Vec<render::RenderNode>> {
        let component = self
            .components
            .get(name)
            .with_context(|| format!("undefined component: {}", name))?;

        validate_component_args(&component, &args)?;

        let scope = env::Scope::new_child(scope);
        for decl in component
            .arg_declarations
            .iter()
            .filter(|a| a.default.is_some())
        {
            let expr = decl.default.clone().unwrap();
            let default = self.eval_expr(&expr, &self.root_scope)?;

            scope.borrow_mut().set(&decl.name, default);
        }

        for (name, value) in args {
            scope.borrow_mut().set(&name, value.clone());
        }

        Ok(self.render_childnode(&component.body, &scope)?)
    }

    fn render_childnode(
        &self,
        node: &ast::ChildNode,
        scope: &ScopeRef,
    ) -> anyhow::Result<Vec<render::RenderNode>> {
        Ok(match node {
            ast::ChildNode::ComponentCall(component_call) => {
                let mut args_map: HashMap<String, Value> =
                    HashMap::with_capacity(component_call.args.len());

                for arg in &component_call.args {
                    args_map.insert(arg.name.clone(), self.eval_expr(&arg.value, scope)?);
                }

                if self
                    .native_component_registery
                    .has_component(&component_call.name.as_str())
                {
                    let mut children = Vec::with_capacity(component_call.body.len());
                    for child in &component_call.body {
                        children.extend(self.render_childnode(child, scope)?);
                    }

                    vec![self.native_component_registery.create(
                        &component_call.name,
                        args_map,
                        children,
                    )?]

                    // // TODO: 기본 component
                    // let kind = match component_call.name.as_str() {
                    //     "Center" => render::RenderNodeKind::Box,
                    //     "Column" => render::RenderNodeKind::Box,
                    //     "Row" => render::RenderNodeKind::Box,
                    //     "Box" => render::RenderNodeKind::Box,

                    //     "Text" => {
                    //         let text = args_map
                    //             .get("text")
                    //             .cloned()
                    //             .with_context(|| format!("required"))?
                    //             .into_string()?;

                    //         render::RenderNodeKind::Text {
                    //             text,
                    //             style: render::styles::TextStyle {
                    //                 ..Default::default()
                    //             },
                    //         }
                    //     }
                    //     "Image" => render::RenderNodeKind::Image {
                    //         style: render::styles::ImageStyle {
                    //             ..Default::default()
                    //         },
                    //     },
                    //     _ => unreachable!(),
                    // };

                    // vec![render::RenderNode {
                    //     kind,
                    //     children,
                    //     style: render::styles::CommonStyle {
                    //         ..Default::default()
                    //     },
                    // }]
                } else {
                    self.render_component(&component_call.name, args_map, scope)?
                }
            }
            ast::ChildNode::For(for_block) => {
                let iterable = self.eval_expr(&for_block.iterable, &scope)?;
                let for_scope = env::Scope::new_child(&scope);

                match iterable {
                    Value::String(str) => {
                        let mut nodes = Vec::with_capacity(str.len());
                        for (idx, ch) in str.char_indices() {
                            if let Some(index_name) = &for_block.index_name {
                                for_scope
                                    .borrow_mut()
                                    .set(index_name, Value::Number(idx as f32));
                            }

                            for_scope
                                .borrow_mut()
                                .set(&for_block.item_name, Value::String(ch.to_string()));

                            for node in &for_block.body {
                                nodes.extend(self.render_childnode(&node, &for_scope)?);
                            }
                        }

                        nodes
                    }
                    Value::Array(values) => {
                        let mut nodes = Vec::with_capacity(values.len());

                        for (idx, ch) in values.iter().enumerate() {
                            if let Some(index_name) = &for_block.index_name {
                                for_scope
                                    .borrow_mut()
                                    .set(index_name, Value::Number(idx as f32));
                            }

                            for_scope.borrow_mut().set(&for_block.item_name, ch.clone());

                            for node in &for_block.body {
                                nodes.extend(self.render_childnode(&node, &for_scope)?);
                            }
                        }

                        nodes
                    }
                    v => anyhow::bail!("{:?} is not iterable", v.ty()),
                }
            }
            ast::ChildNode::If(if_block) => {
                let cond = self.eval_expr(&if_block.condition, scope)?;
                let cond = value_to_bool(cond);

                if cond {
                    let mut nodes = Vec::with_capacity(if_block.body.len());
                    for node in &if_block.body {
                        nodes.extend(self.render_childnode(node, &scope)?);
                    }

                    nodes
                } else {
                    vec![]
                }
            }
        })
    }

    pub fn evaluate(&mut self, prog: &ast::SeruProgram) -> anyhow::Result<()> {
        for decl in &prog.top_declarations {
            if let ast::TopDeclaration::ConstDeclaration(const_decl) = decl {
                self.eval_const_decl(const_decl)?;
            };
        }

        for decl in &prog.top_declarations {
            if let ast::TopDeclaration::ComponentDeclaration(comp_decl) = decl {
                self.eval_comp_decl(comp_decl)?;
            };
        }

        Ok(())
    }

    fn eval_comp_decl(&mut self, comp_decl: &ast::ComponentDeclaration) -> anyhow::Result<()> {
        if self
            .native_component_registery
            .has_component(&comp_decl.name)
        {
            anyhow::bail!("already decleared component: {}", comp_decl.name)
        }

        if self.components.contains_key(&comp_decl.name) {
            // TODO: add span
            anyhow::bail!("already decleared component: {}", comp_decl.name)
        }

        self.components
            .insert(comp_decl.name.clone(), comp_decl.clone());

        Ok(())
    }

    fn eval_const_decl(&mut self, const_decl: &ast::ConstDeclaration) -> anyhow::Result<()> {
        let scope = Rc::clone(&self.root_scope);
        let value = self.eval_expr(&const_decl.value, &scope)?;
        self.root_scope.borrow_mut().set(&const_decl.name, value);

        Ok(())
    }

    fn eval_expr(
        &self,
        expr: &ast::Expr,
        scope: &Rc<RefCell<env::Scope>>,
    ) -> anyhow::Result<Value> {
        Ok(match &expr.kind {
            ast::ExprKind::String(val) => Value::String(val.clone()),
            ast::ExprKind::Number(val) => Value::Number(val.clone()),
            ast::ExprKind::Bool(bool) => Value::Bool(*bool),
            ast::ExprKind::Null => Value::Null,
            ast::ExprKind::Array(exprs) => Value::Array(
                exprs
                    .iter()
                    .map(|e| self.eval_expr(e, scope))
                    .collect::<anyhow::Result<Vec<_>>>()?,
            ),

            ast::ExprKind::Ident(name) => scope
                .borrow()
                .get(name)
                .with_context(|| format!("undefined variable: {} {:?}", name, expr.span))?,

            ast::ExprKind::Bin {
                lhs: _,
                op: _,
                rhs: _,
            } => self.eval_bin(expr, scope)?,

            ast::ExprKind::Unary { op, expr } => {
                let val = self.eval_expr(expr, scope)?;

                match op {
                    ast::UnaryOp::Plus => match val {
                        Value::Number(num) => Value::Number(num),
                        _ => anyhow::bail!("cannot unary plus {:?}", val),
                    },
                    ast::UnaryOp::Minus => match val {
                        Value::Number(num) => Value::Number(-num),
                        _ => anyhow::bail!("cannot unary minus {:?}", val),
                    },
                    ast::UnaryOp::Not => Value::Bool(!value_to_bool(val)),
                }
            }

            ast::ExprKind::Call { callee, args } => {
                let val = self.eval_expr(callee, scope)?;

                let Value::Func(func) = val else {
                    anyhow::bail!("cannot call {:?} {:?}", val.ty(), callee.span)
                };

                let mut val_args = Vec::with_capacity(args.len());
                for arg in args {
                    val_args.push(self.eval_expr(arg, scope)?);
                }

                func(val_args)?
            }

            ast::ExprKind::StringInterpolation(parts) => {
                let mut output = String::new();

                for part in parts {
                    match &part.kind {
                        ast::ParsedStringPartKind::Text(txt) => output.push_str(txt),
                        ast::ParsedStringPartKind::Interpolation(expr) => {
                            let val = self.eval_expr(expr, scope)?;
                            let s = value_to_string(val);
                            output.push_str(&s);
                        }
                    }
                }

                Value::String(output)
            }

            ast::ExprKind::Index { target, index } => {
                let target = self.eval_expr(target, scope)?;
                let index = self.eval_expr(index, scope)?.into_number()?;

                if !check_integer(index) {
                    anyhow::bail!("index must be a integer {:?}", expr.span);
                }

                let index = index as isize;
                match target {
                    Value::String(val) => {
                        let index = calc_index(index, val.len())?;
                        Value::String(val.chars().nth(index).expect("string index").to_string())
                    }
                    Value::Array(values) => {
                        let index = calc_index(index, values.len())?;
                        values[index].clone()
                    }
                    _ => anyhow::bail!("cannot indexing {:?} {:?}", target.ty(), expr.span),
                }
            }
        })
    }

    fn eval_bin(&self, expr: &ast::Expr, scope: &Rc<RefCell<env::Scope>>) -> anyhow::Result<Value> {
        Ok(match &expr.kind {
            ast::ExprKind::Bin { lhs, op, rhs } => {
                let lhs = self.eval_expr(lhs, scope)?;
                let rhs = self.eval_expr(rhs, scope)?;

                match op {
                    ast::BinOp::Plus => match (lhs, rhs) {
                        (Value::String(lhs), Value::String(rhs)) => Value::String(lhs + &rhs),
                        (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs + rhs),
                        (Value::Array(lhs), Value::Array(rhs)) => Value::Array([lhs, rhs].concat()),

                        (lhs, rhs) => anyhow::bail!(
                            "cannot plus {:?} and {:?} ({:?})",
                            lhs.ty(),
                            rhs.ty(),
                            expr.span
                        ),
                    },
                    ast::BinOp::Minus => match (lhs, rhs) {
                        (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs - rhs),

                        (lhs, rhs) => anyhow::bail!(
                            "cannot minus {:?} and {:?} ({:?})",
                            lhs.ty(),
                            rhs.ty(),
                            expr.span
                        ),
                    },
                    ast::BinOp::Multiply => match (lhs, rhs) {
                        (Value::String(lhs), Value::Number(rhs)) => {
                            Value::String(lhs.repeat(rhs as usize))
                        }
                        (Value::Array(lhs), Value::Number(rhs)) => {
                            if !check_integer(rhs) || rhs < 0.0 {
                                anyhow::bail!(
                                    "repeat count must be a non-negative integer {:?}",
                                    expr.span
                                );
                            }

                            let n = rhs as usize;
                            let mut repeated = Vec::with_capacity(lhs.len() * n);

                            for _ in 0..(n) {
                                repeated.extend_from_slice(&lhs);
                            }

                            Value::Array(repeated)
                        }
                        (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs * rhs),

                        (lhs, rhs) => anyhow::bail!(
                            "cannot multiply {:?} and {:?} ({:?})",
                            lhs.ty(),
                            rhs.ty(),
                            expr.span
                        ),
                    },

                    ast::BinOp::Divide => match (lhs, rhs) {
                        (Value::Number(lhs), Value::Number(rhs)) => Value::Number(lhs / rhs),

                        (lhs, rhs) => anyhow::bail!(
                            "cannot divide {:?} and {:?} ({:?})",
                            lhs.ty(),
                            rhs.ty(),
                            expr.span
                        ),
                    },

                    ast::BinOp::Eq => Value::Bool(lhs == rhs),

                    ast::BinOp::Neq => Value::Bool(lhs != rhs),
                }
            }
            _ => unreachable!(),
        })
    }
}

fn value_to_bool(value: Value) -> bool {
    match value {
        Value::String(val) => val.is_empty(),
        Value::Number(val) => val != 0.0,
        Value::Array(values) => !values.is_empty(),
        Value::Bool(bool) => bool,
        Value::Null => false,
        Value::Func(_) => true,
    }
}

fn calc_index(idx: isize, len: usize) -> anyhow::Result<usize> {
    let target_idx = if idx < 0 {
        (len as isize).checked_add(idx)
    } else {
        Some(idx)
    };

    if let Some(valid_idx) = target_idx {
        if valid_idx >= 0 && (valid_idx as usize) < len {
            return Ok(valid_idx as usize);
        }
    }

    anyhow::bail!("index out of bounds")
}

fn value_to_string(value: Value) -> String {
    match value {
        Value::String(val) => val.to_string(),
        Value::Number(val) => val.to_string(),
        Value::Array(values) => {
            let s = values
                .iter()
                .map(|v| value_to_string(v.clone()))
                .collect::<Vec<_>>()
                .join(", ");

            format!("[{s}]")
        }
        Value::Bool(bool) => {
            if bool {
                "true".to_string()
            } else {
                "false".to_string()
            }
        }
        Value::Null => "null".to_string(),
        Value::Func(_) => "func".to_string(),
    }
}

fn validate_component_args(
    component: &ComponentDeclaration,
    args: &HashMap<String, Value>,
) -> anyhow::Result<()> {
    // validate args
    let mut required: HashSet<&str> = component
        .arg_declarations
        .iter()
        .filter(|arg| arg.default.is_none())
        .map(|arg| arg.name.as_str())
        .collect();

    let valid_args: HashSet<&str> = component
        .arg_declarations
        .iter()
        .map(|arg| arg.name.as_str())
        .collect();

    for (name, _) in args {
        let name = name.as_str();

        if !valid_args.contains(name) {
            anyhow::bail!("unknown argument: {}", name);
        }

        required.remove(name);
    }

    if let Some(missing) = required.iter().next() {
        anyhow::bail!("missing required argument: {}", missing);
    }

    Ok(())
}
