use std::collections::HashMap;
use swc_core::ecma::{
        ast::*, utils::IdentRenamer, visit::{VisitMut, VisitMutWith}
};
use rustc_hash::FxHashMap;

use crate::visitors::common::{ RenderFn, COMPILE_MODE, COMPILE_MODE_SUB_COMPONENT };
pub struct TransformProcessVisitor<'a> {
    render_fn_map: &'a HashMap<String, RenderFn>,
    in_compile_mode_jsx: bool,
}

impl<'a> TransformProcessVisitor<'a> {
    pub fn new(render_fn_map: &'a HashMap<String, RenderFn>) -> Self {
        TransformProcessVisitor {
            render_fn_map,
            in_compile_mode_jsx: false,
        }
    }
}

impl<'a> VisitMut for TransformProcessVisitor<'a> {
   fn visit_mut_jsx_element(&mut self, el: &mut JSXElement) {
        if !self.in_compile_mode_jsx {
            for attr in &mut el.opening.attrs {
                if let JSXAttrOrSpread::JSXAttr(jsx_attr) = attr {
                    if let JSXAttrName::Ident(jsx_attr_name) = &jsx_attr.name {
                        if jsx_attr_name.sym == COMPILE_MODE {
                            self.in_compile_mode_jsx = true;
                            print!("in_compile_mode_jsx\n,{:?}", el.children);
                            break;
                        }
                    }
                }
            }
        }


            el.opening.attrs.retain(|attr| match &attr {
                JSXAttrOrSpread::JSXAttr(JSXAttr {name, ..}) => {
                    match name {
                        JSXAttrName::Ident(jsx_attr_name) => {
                            jsx_attr_name.sym != COMPILE_MODE_SUB_COMPONENT
                        },
                        _=>true
                    }
                },
                _=>true
            });
        
        el.visit_mut_children_with(self);
    }
    
    fn visit_mut_expr(&mut self, n: &mut Expr) {
        if self.in_compile_mode_jsx {
            if let Expr::Call(CallExpr { callee: Callee::Expr(callee_expr), args, .. }) = n {
                    if let Expr::Ident(ident) = &**callee_expr {
                        if self.render_fn_map.contains_key(&ident.sym.to_string()) {
                            let mut jsx_ele = Expr::JSXElement(Box::new(self.render_fn_map.get(&ident.sym.to_string()).unwrap().jsx_element.clone()));
                            let mut name_map = FxHashMap::default();
                            let old_args = &self.render_fn_map.get(&ident.sym.to_string()).unwrap().params;
                            let format_args: Vec<_> = args.into_iter()
                            .map(|arg| arg.expr.clone())
                            .filter(|expr| matches!(**expr, Expr::Ident(_)))
                            .map(|expr| if let Expr::Ident(ident) = *expr { Some(ident) } else { None })
                            .flatten()
                            .collect();
                            if format_args.len() == old_args.len() {
                                for (i, arg) in format_args.iter().enumerate() {
                                    let old_arg = &old_args[i];
                                    if let Pat::Ident(old_ident) = old_arg {
                                        name_map.insert(old_ident.to_id(), arg.to_id());
                                    }
                                }
                                let mut ident_renamer_visitor = IdentRenamer::new(&name_map);
                                jsx_ele.visit_mut_with(&mut ident_renamer_visitor);
                                *n = jsx_ele;
                            }
    
                        }
                    }
            }
        }
        n.visit_mut_children_with(self);
    }
}