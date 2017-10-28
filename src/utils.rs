// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::borrow::Cow;

use syntax::{abi, ptr};
use syntax::ast::{self, Attribute, CrateSugar, MetaItem, MetaItemKind, NestedMetaItem,
                  NestedMetaItemKind, Path, Visibility};
use syntax::codemap::{BytePos, Span, NO_EXPANSION};

use rewrite::RewriteContext;
use shape::Shape;

// When we get scoped annotations, we should have rustfmt::skip.
const SKIP_ANNOTATION: &'static str = "rustfmt_skip";

// Computes the length of a string's last line, minus offset.
pub fn extra_offset(text: &str, shape: Shape) -> usize {
    match text.rfind('\n') {
        // 1 for newline character
        Some(idx) => text.len()
            .checked_sub(idx + 1 + shape.used_width())
            .unwrap_or(0),
        None => text.len(),
    }
}

// Uses Cow to avoid allocating in the common cases.
pub fn format_visibility(vis: &Visibility) -> Cow<'static, str> {
    match *vis {
        Visibility::Public => Cow::from("pub "),
        Visibility::Inherited => Cow::from(""),
        Visibility::Crate(_, CrateSugar::PubCrate) => Cow::from("pub(crate) "),
        Visibility::Crate(_, CrateSugar::JustCrate) => Cow::from("crate "),
        Visibility::Restricted { ref path, .. } => {
            let Path { ref segments, .. } = **path;
            let mut segments_iter = segments.iter().map(|seg| seg.identifier.name.to_string());
            if path.is_global() {
                segments_iter
                    .next()
                    .expect("Non-global path in pub(restricted)?");
            }
            let is_keyword = |s: &str| s == "self" || s == "super";
            let path = segments_iter.collect::<Vec<_>>().join("::");
            let in_str = if is_keyword(&path) { "" } else { "in " };

            Cow::from(format!("pub({}{}) ", in_str, path))
        }
    }
}

#[inline]
pub fn format_constness(constness: ast::Constness) -> &'static str {
    match constness {
        ast::Constness::Const => "const ",
        ast::Constness::NotConst => "",
    }
}

#[inline]
pub fn format_defaultness(defaultness: ast::Defaultness) -> &'static str {
    match defaultness {
        ast::Defaultness::Default => "default ",
        ast::Defaultness::Final => "",
    }
}

#[inline]
pub fn format_unsafety(unsafety: ast::Unsafety) -> &'static str {
    match unsafety {
        ast::Unsafety::Unsafe => "unsafe ",
        ast::Unsafety::Normal => "",
    }
}

#[inline]
pub fn format_mutability(mutability: ast::Mutability) -> &'static str {
    match mutability {
        ast::Mutability::Mutable => "mut ",
        ast::Mutability::Immutable => "",
    }
}

#[inline]
pub fn format_abi(abi: abi::Abi, explicit_abi: bool, is_mod: bool) -> Cow<'static, str> {
    if abi == abi::Abi::Rust && !is_mod {
        Cow::from("")
    } else if abi == abi::Abi::C && !explicit_abi {
        Cow::from("extern ")
    } else {
        Cow::from(format!("extern {} ", abi))
    }
}

#[inline]
// Transform `Vec<syntax::ptr::P<T>>` into `Vec<&T>`
pub fn ptr_vec_to_ref_vec<T>(vec: &[ptr::P<T>]) -> Vec<&T> {
    vec.iter().map(|x| &**x).collect::<Vec<_>>()
}

#[inline]
pub fn filter_attributes(attrs: &[ast::Attribute], style: ast::AttrStyle) -> Vec<ast::Attribute> {
    attrs
        .iter()
        .filter(|a| a.style == style)
        .cloned()
        .collect::<Vec<_>>()
}

#[inline]
pub fn inner_attributes(attrs: &[ast::Attribute]) -> Vec<ast::Attribute> {
    filter_attributes(attrs, ast::AttrStyle::Inner)
}

#[inline]
pub fn outer_attributes(attrs: &[ast::Attribute]) -> Vec<ast::Attribute> {
    filter_attributes(attrs, ast::AttrStyle::Outer)
}

#[inline]
pub fn last_line_contains_single_line_comment(s: &str) -> bool {
    s.lines().last().map_or(false, |l| l.contains("//"))
}

#[inline]
pub fn is_attributes_extendable(attrs_str: &str) -> bool {
    !attrs_str.contains('\n') && !last_line_contains_single_line_comment(attrs_str)
}

// The width of the first line in s.
#[inline]
pub fn first_line_width(s: &str) -> usize {
    match s.find('\n') {
        Some(n) => n,
        None => s.len(),
    }
}

// The width of the last line in s.
#[inline]
pub fn last_line_width(s: &str) -> usize {
    match s.rfind('\n') {
        Some(n) => s.len() - n - 1,
        None => s.len(),
    }
}

// The total used width of the last line.
#[inline]
pub fn last_line_used_width(s: &str, offset: usize) -> usize {
    if s.contains('\n') {
        last_line_width(s)
    } else {
        offset + s.len()
    }
}

#[inline]
pub fn trimmed_last_line_width(s: &str) -> usize {
    match s.rfind('\n') {
        Some(n) => s[(n + 1)..].trim().len(),
        None => s.trim().len(),
    }
}

#[inline]
pub fn last_line_extendable(s: &str) -> bool {
    if s.ends_with("\"#") {
        return true;
    }
    for c in s.chars().rev() {
        match c {
            ')' | ']' | '}' | '?' => continue,
            '\n' => break,
            _ if c.is_whitespace() => continue,
            _ => return false,
        }
    }
    true
}

#[inline]
fn is_skip(meta_item: &MetaItem) -> bool {
    match meta_item.node {
        MetaItemKind::Word => meta_item.name == SKIP_ANNOTATION,
        MetaItemKind::List(ref l) => {
            meta_item.name == "cfg_attr" && l.len() == 2 && is_skip_nested(&l[1])
        }
        _ => false,
    }
}

#[inline]
fn is_skip_nested(meta_item: &NestedMetaItem) -> bool {
    match meta_item.node {
        NestedMetaItemKind::MetaItem(ref mi) => is_skip(mi),
        NestedMetaItemKind::Literal(_) => false,
    }
}

#[inline]
pub fn contains_skip(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|a| a.meta().map_or(false, |a| is_skip(&a)))
}

// Find the end of a TyParam
#[inline]
pub fn end_typaram(typaram: &ast::TyParam) -> BytePos {
    typaram
        .bounds
        .last()
        .map_or(typaram.span, |bound| match *bound {
            ast::RegionTyParamBound(ref lt) => lt.span,
            ast::TraitTyParamBound(ref prt, _) => prt.span,
        })
        .hi()
}

#[inline]
pub fn semicolon_for_expr(context: &RewriteContext, expr: &ast::Expr) -> bool {
    match expr.node {
        ast::ExprKind::Ret(..) | ast::ExprKind::Continue(..) | ast::ExprKind::Break(..) => {
            context.config.trailing_semicolon()
        }
        _ => false,
    }
}

#[inline]
pub fn semicolon_for_stmt(context: &RewriteContext, stmt: &ast::Stmt) -> bool {
    match stmt.node {
        ast::StmtKind::Semi(ref expr) => match expr.node {
            ast::ExprKind::While(..) |
            ast::ExprKind::WhileLet(..) |
            ast::ExprKind::Loop(..) |
            ast::ExprKind::ForLoop(..) => false,
            ast::ExprKind::Break(..) | ast::ExprKind::Continue(..) | ast::ExprKind::Ret(..) => {
                context.config.trailing_semicolon()
            }
            _ => true,
        },
        ast::StmtKind::Expr(..) => false,
        _ => true,
    }
}

#[inline]
pub fn stmt_expr(stmt: &ast::Stmt) -> Option<&ast::Expr> {
    match stmt.node {
        ast::StmtKind::Expr(ref expr) => Some(expr),
        _ => None,
    }
}

#[inline]
pub fn trim_newlines(input: &str) -> &str {
    match input.find(|c| c != '\n' && c != '\r') {
        Some(start) => {
            let end = input.rfind(|c| c != '\n' && c != '\r').unwrap_or(0) + 1;
            &input[start..end]
        }
        None => "",
    }
}

// Macro for deriving implementations of Serialize/Deserialize for enums
#[macro_export]
macro_rules! impl_enum_serialize_and_deserialize {
    ( $e:ident, $( $x:ident ),* ) => {
        impl ::serde::ser::Serialize for $e {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: ::serde::ser::Serializer
            {
                use serde::ser::Error;

                // We don't know whether the user of the macro has given us all options.
                #[allow(unreachable_patterns)]
                match *self {
                    $(
                        $e::$x => serializer.serialize_str(stringify!($x)),
                    )*
                    _ => {
                        Err(S::Error::custom(format!("Cannot serialize {:?}", self)))
                    }
                }
            }
        }

        impl<'de> ::serde::de::Deserialize<'de> for $e {
            fn deserialize<D>(d: D) -> Result<Self, D::Error>
                    where D: ::serde::Deserializer<'de> {
                use std::ascii::AsciiExt;
                use serde::de::{Error, Visitor};
                use std::marker::PhantomData;
                use std::fmt;
                struct StringOnly<T>(PhantomData<T>);
                impl<'de, T> Visitor<'de> for StringOnly<T>
                        where T: ::serde::Deserializer<'de> {
                    type Value = String;
                    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
                        formatter.write_str("string")
                    }
                    fn visit_str<E>(self, value: &str) -> Result<String, E> {
                        Ok(String::from(value))
                    }
                }
                let s = d.deserialize_string(StringOnly::<D>(PhantomData))?;
                $(
                    if stringify!($x).eq_ignore_ascii_case(&s) {
                      return Ok($e::$x);
                    }
                )*
                static ALLOWED: &'static[&str] = &[$(stringify!($x),)*];
                Err(D::Error::unknown_variant(&s, ALLOWED))
            }
        }

        impl ::std::str::FromStr for $e {
            type Err = &'static str;

            fn from_str(s: &str) -> Result<Self, Self::Err> {
                use std::ascii::AsciiExt;
                $(
                    if stringify!($x).eq_ignore_ascii_case(s) {
                        return Ok($e::$x);
                    }
                )*
                Err("Bad variant")
            }
        }

        impl ::config::ConfigType for $e {
            fn doc_hint() -> String {
                let mut variants = Vec::new();
                $(
                    variants.push(stringify!($x));
                )*
                format!("[{}]", variants.join("|"))
            }
        }
    };
}

macro_rules! msg {
    ($($arg:tt)*) => (
        match writeln!(&mut ::std::io::stderr(), $($arg)* ) {
            Ok(_) => {},
            Err(x) => panic!("Unable to write to stderr: {}", x),
        }
    )
}

// For format_missing and last_pos, need to use the source callsite (if applicable).
// Required as generated code spans aren't guaranteed to follow on from the last span.
macro_rules! source {
    ($this:ident, $sp: expr) => {
        $sp.source_callsite()
    }
}

pub fn mk_sp(lo: BytePos, hi: BytePos) -> Span {
    Span::new(lo, hi, NO_EXPANSION)
}

// Return true if the given span does not intersect with file lines.
macro_rules! out_of_file_lines_range {
    ($self:ident, $span:expr) => {
        !$self.config
            .file_lines()
            .intersects(&$self.codemap.lookup_line_range($span))
    }
}

macro_rules! skip_out_of_file_lines_range {
    ($self:ident, $span:expr) => {
        if out_of_file_lines_range!($self, $span) {
            return None;
        }
    }
}

macro_rules! skip_out_of_file_lines_range_visitor {
    ($self:ident, $span:expr) => {
        if out_of_file_lines_range!($self, $span) {
            $self.push_rewrite($span, None);
            return;
        }
    }
}

// Wraps String in an Option. Returns Some when the string adheres to the
// Rewrite constraints defined for the Rewrite trait and else otherwise.
pub fn wrap_str(s: String, max_width: usize, shape: Shape) -> Option<String> {
    if is_valid_str(&s, max_width, shape) {
        Some(s)
    } else {
        None
    }
}

fn is_valid_str(snippet: &str, max_width: usize, shape: Shape) -> bool {
    if !snippet.is_empty() {
        // First line must fits with `shape.width`.
        if first_line_width(snippet) > shape.width {
            return false;
        }
        // If the snippet does not include newline, we are done.
        if first_line_width(snippet) == snippet.len() {
            return true;
        }
        // The other lines must fit within the maximum width.
        if snippet.lines().skip(1).any(|line| line.len() > max_width) {
            return false;
        }
        // A special check for the last line, since the caller may
        // place trailing characters on this line.
        if last_line_width(snippet) > shape.used_width() + shape.width {
            return false;
        }
    }
    true
}

#[inline]
pub fn colon_spaces(before: bool, after: bool) -> &'static str {
    match (before, after) {
        (true, true) => " : ",
        (true, false) => " :",
        (false, true) => ": ",
        (false, false) => ":",
    }
}

#[inline]
pub fn paren_overhead(context: &RewriteContext) -> usize {
    if context.config.spaces_within_parens() {
        4
    } else {
        2
    }
}

pub fn left_most_sub_expr(e: &ast::Expr) -> &ast::Expr {
    match e.node {
        ast::ExprKind::InPlace(ref e, _) |
        ast::ExprKind::Call(ref e, _) |
        ast::ExprKind::Binary(_, ref e, _) |
        ast::ExprKind::Cast(ref e, _) |
        ast::ExprKind::Type(ref e, _) |
        ast::ExprKind::Assign(ref e, _) |
        ast::ExprKind::AssignOp(_, ref e, _) |
        ast::ExprKind::Field(ref e, _) |
        ast::ExprKind::TupField(ref e, _) |
        ast::ExprKind::Index(ref e, _) |
        ast::ExprKind::Range(Some(ref e), _, _) |
        ast::ExprKind::Try(ref e) => left_most_sub_expr(e),
        _ => e,
    }
}

// isatty shamelessly adapted from cargo.
#[cfg(unix)]
pub fn isatty() -> bool {
    extern crate libc;

    unsafe { libc::isatty(libc::STDOUT_FILENO) != 0 }
}
#[cfg(windows)]
pub fn isatty() -> bool {
    extern crate kernel32;
    extern crate winapi;

    unsafe {
        let handle = kernel32::GetStdHandle(winapi::winbase::STD_OUTPUT_HANDLE);
        let mut out = 0;
        kernel32::GetConsoleMode(handle, &mut out) != 0
    }
}

pub fn starts_with_newline(s: &str) -> bool {
    s.starts_with('\n') || s.starts_with("\r\n")
}
