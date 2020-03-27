use rustc_span::Symbol;
use syntax::ast;
use syntax::visit::Visitor;

use crate::attr::MetaVisitor;
use crate::syntux::parser::Parser;
use crate::syntux::session::ParseSess;

pub(crate) struct ModItem {
    pub(crate) item: ast::Item,
}

/// Traverse `cfg_if!` macro and fetch modules.
pub(crate) struct CfgIfVisitor<'a> {
    parse_sess: &'a ParseSess,
    mods: Vec<ModItem>,
}

impl<'a> CfgIfVisitor<'a> {
    pub(crate) fn new(parse_sess: &'a ParseSess) -> CfgIfVisitor<'a> {
        CfgIfVisitor {
            mods: vec![],
            parse_sess,
        }
    }

    pub(crate) fn mods(self) -> Vec<ModItem> {
        self.mods
    }
}

impl<'a, 'ast: 'a> Visitor<'ast> for CfgIfVisitor<'a> {
    fn visit_mac(&mut self, mac: &'ast ast::MacCall) {
        match self.visit_mac_inner(mac) {
            Ok(()) => (),
            Err(e) => debug!("{}", e),
        }
    }
}

impl<'a, 'ast: 'a> CfgIfVisitor<'a> {
    fn visit_mac_inner(&mut self, mac: &'ast ast::MacCall) -> Result<(), &'static str> {
        // Support both:
        // ```
        // extern crate cfg_if;
        // cfg_if::cfg_if! {..}
        // ```
        // And:
        // ```
        // #[macro_use]
        // extern crate cfg_if;
        // cfg_if! {..}
        // ```
        match mac.path.segments.first() {
            Some(first_segment) => {
                if first_segment.ident.name != Symbol::intern("cfg_if") {
                    return Err("Expected cfg_if");
                }
            }
            None => {
                return Err("Expected cfg_if");
            }
        };

        let items = Parser::parse_cfg_if(self.parse_sess, mac)?;
        self.mods
            .append(&mut items.into_iter().map(|item| ModItem { item }).collect());

        Ok(())
    }
}

/// Extracts `path = "foo.rs"` from attributes.
#[derive(Default)]
pub(crate) struct PathVisitor {
    /// A list of path defined in attributes.
    paths: Vec<String>,
}

impl PathVisitor {
    pub(crate) fn paths(self) -> Vec<String> {
        self.paths
    }
}

impl<'ast> MetaVisitor<'ast> for PathVisitor {
    fn visit_meta_name_value(&mut self, meta_item: &'ast ast::MetaItem, lit: &'ast ast::Lit) {
        if meta_item.check_name(Symbol::intern("path")) && lit.kind.is_str() {
            self.paths.push(lit_to_str(lit));
        }
    }
}

#[cfg(not(windows))]
fn lit_to_str(lit: &ast::Lit) -> String {
    match lit.kind {
        ast::LitKind::Str(symbol, ..) => symbol.to_string(),
        _ => unreachable!(),
    }
}

#[cfg(windows)]
fn lit_to_str(lit: &ast::Lit) -> String {
    match lit.kind {
        ast::LitKind::Str(symbol, ..) => symbol.as_str().replace("/", "\\"),
        _ => unreachable!(),
    }
}
