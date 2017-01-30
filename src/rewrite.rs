// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// A generic trait to abstract the rewriting of an element (of the AST).

use syntax::codemap::{CodeMap, Span};
use syntax::parse::ParseSess;

use {Indent, Shape};
use config::Config;

pub trait Rewrite {
    /// Rewrite self into shape.
    fn rewrite(&self, context: &RewriteContext, shape: Shape) -> Option<String>;
}

#[derive(Clone)]
pub struct RewriteContext<'a> {
    pub parse_session: &'a ParseSess,
    pub codemap: &'a CodeMap,
    pub config: &'a Config,
    // Indentation due to nesting of blocks.
    pub block_indent: Indent,
}

impl<'a> RewriteContext<'a> {
    pub fn nested_context(&self) -> RewriteContext<'a> {
        RewriteContext {
            parse_session: self.parse_session,
            codemap: self.codemap,
            config: self.config,
            block_indent: self.block_indent.block_indent(self.config),
        }
    }

    pub fn snippet(&self, span: Span) -> String {
        self.codemap.span_to_snippet(span).unwrap()
    }
}
