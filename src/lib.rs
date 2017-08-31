// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

#![feature(rustc_private)]

extern crate diff;
#[macro_use]
extern crate log;
extern crate regex;
extern crate rustc_errors as errors;
extern crate serde;
#[macro_use]
extern crate serde_derive;
extern crate serde_json;
extern crate strings;
extern crate syntax;
extern crate term;
extern crate unicode_segmentation;

use std::collections::HashMap;
use std::fmt;
use std::io::{self, stdout, Write};
use std::iter::repeat;
use std::ops::{Add, Sub};
use std::path::{Path, PathBuf};
use std::rc::Rc;

use errors::{DiagnosticBuilder, Handler};
use errors::emitter::{ColorConfig, EmitterWriter};
use strings::string_buffer::StringBuffer;
use syntax::ast;
use syntax::codemap::{CodeMap, FilePathMapping, Span};
use syntax::parse::{self, ParseSess};

use checkstyle::{output_footer, output_header};
use config::Config;
use filemap::FileMap;
use issues::{BadIssueSeeker, Issue};
use utils::{isatty, mk_sp, outer_attributes};
use visitor::FmtVisitor;

pub use self::summary::Summary;

#[macro_use]
mod utils;
pub mod config;
pub mod codemap;
pub mod filemap;
pub mod file_lines;
pub mod visitor;
mod checkstyle;
mod items;
mod missed_spans;
mod lists;
mod types;
mod expr;
mod imports;
mod issues;
mod rewrite;
mod string;
mod comment;
pub mod modules;
pub mod rustfmt_diff;
mod chains;
mod macros;
mod patterns;
mod summary;
mod vertical;

/// Spanned returns a span including attributes, if available.
pub trait Spanned {
    fn span(&self) -> Span;
}

macro_rules! span_with_attrs_lo_hi {
    ($this:ident, $lo:expr, $hi:expr) => {
        {
            let attrs = outer_attributes(&$this.attrs);
            if attrs.is_empty() {
                mk_sp($lo, $hi)
            } else {
                mk_sp(attrs[0].span.lo(), $hi)
            }
        }
    }
}

macro_rules! span_with_attrs {
    ($this:ident) => {
        span_with_attrs_lo_hi!($this, $this.span.lo(), $this.span.hi())
    }
}

macro_rules! implement_spanned {
    ($this:ty) => {
        impl Spanned for $this {
            fn span(&self) -> Span {
                span_with_attrs!(self)
            }
        }
    }
}

// Implement `Spanned` for structs with `attrs` field.
implement_spanned!(ast::Expr);
implement_spanned!(ast::Field);
implement_spanned!(ast::ForeignItem);
implement_spanned!(ast::Item);
implement_spanned!(ast::Local);

impl Spanned for ast::Stmt {
    fn span(&self) -> Span {
        match self.node {
            ast::StmtKind::Local(ref local) => mk_sp(local.span().lo(), self.span.hi()),
            ast::StmtKind::Item(ref item) => mk_sp(item.span().lo(), self.span.hi()),
            ast::StmtKind::Expr(ref expr) | ast::StmtKind::Semi(ref expr) => {
                mk_sp(expr.span().lo(), self.span.hi())
            }
            ast::StmtKind::Mac(ref mac) => {
                let (_, _, ref attrs) = **mac;
                if attrs.is_empty() {
                    self.span
                } else {
                    mk_sp(attrs[0].span.lo(), self.span.hi())
                }
            }
        }
    }
}

impl Spanned for ast::Pat {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for ast::Ty {
    fn span(&self) -> Span {
        self.span
    }
}

impl Spanned for ast::Arm {
    fn span(&self) -> Span {
        span_with_attrs_lo_hi!(self, self.pats[0].span.lo(), self.body.span.hi())
    }
}

impl Spanned for ast::Arg {
    fn span(&self) -> Span {
        if items::is_named_arg(self) {
            utils::mk_sp(self.pat.span.lo(), self.ty.span.hi())
        } else {
            self.ty.span
        }
    }
}

impl Spanned for ast::StructField {
    fn span(&self) -> Span {
        span_with_attrs_lo_hi!(self, self.span.lo(), self.ty.span.hi())
    }
}

impl Spanned for ast::WherePredicate {
    fn span(&self) -> Span {
        match *self {
            ast::WherePredicate::BoundPredicate(ref p) => p.span,
            ast::WherePredicate::RegionPredicate(ref p) => p.span,
            ast::WherePredicate::EqPredicate(ref p) => p.span,
        }
    }
}

impl Spanned for ast::FunctionRetTy {
    fn span(&self) -> Span {
        match *self {
            ast::FunctionRetTy::Default(span) => span,
            ast::FunctionRetTy::Ty(ref ty) => ty.span,
        }
    }
}

impl Spanned for ast::TyParam {
    fn span(&self) -> Span {
        // Note that ty.span is the span for ty.ident, not the whole item.
        let lo = if self.attrs.is_empty() {
            self.span.lo()
        } else {
            self.attrs[0].span.lo()
        };
        if let Some(ref def) = self.default {
            return mk_sp(lo, def.span.hi());
        }
        if self.bounds.is_empty() {
            return mk_sp(lo, self.span.hi());
        }
        let hi = self.bounds[self.bounds.len() - 1].span().hi();
        mk_sp(lo, hi)
    }
}

impl Spanned for ast::TyParamBound {
    fn span(&self) -> Span {
        match *self {
            ast::TyParamBound::TraitTyParamBound(ref ptr, _) => ptr.span,
            ast::TyParamBound::RegionTyParamBound(ref l) => l.span,
        }
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Indent {
    // Width of the block indent, in characters. Must be a multiple of
    // Config::tab_spaces.
    pub block_indent: usize,
    // Alignment in characters.
    pub alignment: usize,
}

impl Indent {
    pub fn new(block_indent: usize, alignment: usize) -> Indent {
        Indent {
            block_indent: block_indent,
            alignment: alignment,
        }
    }

    pub fn from_width(config: &Config, width: usize) -> Indent {
        if config.hard_tabs() {
            let tab_num = width / config.tab_spaces();
            let alignment = width % config.tab_spaces();
            Indent::new(config.tab_spaces() * tab_num, alignment)
        } else {
            Indent::new(width, 0)
        }
    }

    pub fn empty() -> Indent {
        Indent::new(0, 0)
    }

    pub fn block_only(&self) -> Indent {
        Indent {
            block_indent: self.block_indent,
            alignment: 0,
        }
    }

    pub fn block_indent(mut self, config: &Config) -> Indent {
        self.block_indent += config.tab_spaces();
        self
    }

    pub fn block_unindent(mut self, config: &Config) -> Indent {
        if self.block_indent < config.tab_spaces() {
            Indent::new(self.block_indent, 0)
        } else {
            self.block_indent -= config.tab_spaces();
            self
        }
    }

    pub fn width(&self) -> usize {
        self.block_indent + self.alignment
    }

    pub fn to_string(&self, config: &Config) -> String {
        let (num_tabs, num_spaces) = if config.hard_tabs() {
            (self.block_indent / config.tab_spaces(), self.alignment)
        } else {
            (0, self.width())
        };
        let num_chars = num_tabs + num_spaces;
        let mut indent = String::with_capacity(num_chars);
        for _ in 0..num_tabs {
            indent.push('\t')
        }
        for _ in 0..num_spaces {
            indent.push(' ')
        }
        indent
    }
}

impl Add for Indent {
    type Output = Indent;

    fn add(self, rhs: Indent) -> Indent {
        Indent {
            block_indent: self.block_indent + rhs.block_indent,
            alignment: self.alignment + rhs.alignment,
        }
    }
}

impl Sub for Indent {
    type Output = Indent;

    fn sub(self, rhs: Indent) -> Indent {
        Indent::new(
            self.block_indent - rhs.block_indent,
            self.alignment - rhs.alignment,
        )
    }
}

impl Add<usize> for Indent {
    type Output = Indent;

    fn add(self, rhs: usize) -> Indent {
        Indent::new(self.block_indent, self.alignment + rhs)
    }
}

impl Sub<usize> for Indent {
    type Output = Indent;

    fn sub(self, rhs: usize) -> Indent {
        Indent::new(self.block_indent, self.alignment - rhs)
    }
}

#[derive(Copy, Clone, Debug)]
pub struct Shape {
    pub width: usize,
    // The current indentation of code.
    pub indent: Indent,
    // Indentation + any already emitted text on the first line of the current
    // statement.
    pub offset: usize,
}

impl Shape {
    /// `indent` is the indentation of the first line. The next lines
    /// should begin with at least `indent` spaces (except backwards
    /// indentation). The first line should not begin with indentation.
    /// `width` is the maximum number of characters on the last line
    /// (excluding `indent`). The width of other lines is not limited by
    /// `width`.
    /// Note that in reality, we sometimes use width for lines other than the
    /// last (i.e., we are conservative).
    // .......*-------*
    //        |       |
    //        |     *-*
    //        *-----|
    // |<------------>|  max width
    // |<---->|          indent
    //        |<--->|    width
    pub fn legacy(width: usize, indent: Indent) -> Shape {
        Shape {
            width: width,
            indent: indent,
            offset: indent.alignment,
        }
    }

    pub fn indented(indent: Indent, config: &Config) -> Shape {
        Shape {
            width: config.max_width().checked_sub(indent.width()).unwrap_or(0),
            indent: indent,
            offset: indent.alignment,
        }
    }

    pub fn with_max_width(&self, config: &Config) -> Shape {
        Shape {
            width: config
                .max_width()
                .checked_sub(self.indent.width())
                .unwrap_or(0),
            ..*self
        }
    }

    pub fn offset(width: usize, indent: Indent, offset: usize) -> Shape {
        Shape {
            width: width,
            indent: indent,
            offset: offset,
        }
    }

    pub fn visual_indent(&self, extra_width: usize) -> Shape {
        let alignment = self.offset + extra_width;
        Shape {
            width: self.width,
            indent: Indent::new(self.indent.block_indent, alignment),
            offset: alignment,
        }
    }

    pub fn block_indent(&self, extra_width: usize) -> Shape {
        if self.indent.alignment == 0 {
            Shape {
                width: self.width,
                indent: Indent::new(self.indent.block_indent + extra_width, 0),
                offset: 0,
            }
        } else {
            Shape {
                width: self.width,
                indent: self.indent + extra_width,
                offset: self.indent.alignment + extra_width,
            }
        }
    }

    pub fn block_left(&self, width: usize) -> Option<Shape> {
        self.block_indent(width).sub_width(width)
    }

    pub fn add_offset(&self, extra_width: usize) -> Shape {
        Shape {
            offset: self.offset + extra_width,
            ..*self
        }
    }

    pub fn block(&self) -> Shape {
        Shape {
            indent: self.indent.block_only(),
            ..*self
        }
    }

    pub fn sub_width(&self, width: usize) -> Option<Shape> {
        Some(Shape {
            width: try_opt!(self.width.checked_sub(width)),
            ..*self
        })
    }

    pub fn shrink_left(&self, width: usize) -> Option<Shape> {
        Some(Shape {
            width: try_opt!(self.width.checked_sub(width)),
            indent: self.indent + width,
            offset: self.offset + width,
        })
    }

    pub fn offset_left(&self, width: usize) -> Option<Shape> {
        self.add_offset(width).sub_width(width)
    }

    pub fn used_width(&self) -> usize {
        self.indent.block_indent + self.offset
    }

    pub fn rhs_overhead(&self, config: &Config) -> usize {
        config
            .max_width()
            .checked_sub(self.used_width() + self.width)
            .unwrap_or(0)
    }
}

pub enum ErrorKind {
    // Line has exceeded character limit (found, maximum)
    LineOverflow(usize, usize),
    // Line ends in whitespace
    TrailingWhitespace,
    // TO-DO or FIX-ME item without an issue number
    BadIssue(Issue),
}

impl fmt::Display for ErrorKind {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        match *self {
            ErrorKind::LineOverflow(found, maximum) => write!(
                fmt,
                "line exceeded maximum width (maximum: {}, found: {})",
                maximum,
                found
            ),
            ErrorKind::TrailingWhitespace => write!(fmt, "left behind trailing whitespace"),
            ErrorKind::BadIssue(issue) => write!(fmt, "found {}", issue),
        }
    }
}

// Formatting errors that are identified *after* rustfmt has run.
pub struct FormattingError {
    line: usize,
    kind: ErrorKind,
    is_comment: bool,
    line_buffer: String,
}

impl FormattingError {
    fn msg_prefix(&self) -> &str {
        match self.kind {
            ErrorKind::LineOverflow(..) | ErrorKind::TrailingWhitespace => "error:",
            ErrorKind::BadIssue(_) => "WARNING:",
        }
    }

    fn msg_suffix(&self) -> String {
        match self.kind {
            ErrorKind::LineOverflow(..) if self.is_comment => format!(
                "use `error_on_line_overflow_comments = false` to suppress \
                 the warning against line comments\n",
            ),
            _ => String::from(""),
        }
    }

    // (space, target)
    pub fn format_len(&self) -> (usize, usize) {
        match self.kind {
            ErrorKind::LineOverflow(found, max) => (max, found - max),
            ErrorKind::TrailingWhitespace => {
                let trailing_ws_len = self.line_buffer
                    .chars()
                    .rev()
                    .take_while(|c| c.is_whitespace())
                    .count();
                (self.line_buffer.len() - trailing_ws_len, trailing_ws_len)
            }
            _ => (0, 0), // unreachable
        }
    }
}

pub struct FormatReport {
    // Maps stringified file paths to their associated formatting errors.
    file_error_map: HashMap<String, Vec<FormattingError>>,
}

impl FormatReport {
    fn new() -> FormatReport {
        FormatReport {
            file_error_map: HashMap::new(),
        }
    }

    pub fn warning_count(&self) -> usize {
        self.file_error_map
            .iter()
            .map(|(_, errors)| errors.len())
            .fold(0, |acc, x| acc + x)
    }

    pub fn has_warnings(&self) -> bool {
        self.warning_count() > 0
    }

    pub fn print_warnings_fancy(
        &self,
        mut t: Box<term::Terminal<Output = io::Stderr>>,
    ) -> Result<(), term::Error> {
        for (file, errors) in &self.file_error_map {
            for error in errors {
                let prefix_space_len = error.line.to_string().len();
                let prefix_spaces: String = repeat(" ").take(1 + prefix_space_len).collect();

                // First line: the overview of error
                t.fg(term::color::RED)?;
                t.attr(term::Attr::Bold)?;
                write!(t, "{} ", error.msg_prefix())?;
                t.reset()?;
                t.attr(term::Attr::Bold)?;
                write!(t, "{}\n", error.kind)?;

                // Second line: file info
                write!(t, "{}--> ", &prefix_spaces[1..])?;
                t.reset()?;
                write!(t, "{}:{}\n", file, error.line)?;

                // Third to fifth lines: show the line which triggered error, if available.
                if !error.line_buffer.is_empty() {
                    let (space_len, target_len) = error.format_len();
                    t.attr(term::Attr::Bold)?;
                    write!(t, "{}|\n{} | ", prefix_spaces, error.line)?;
                    t.reset()?;
                    write!(t, "{}\n", error.line_buffer)?;
                    t.attr(term::Attr::Bold)?;
                    write!(t, "{}| ", prefix_spaces)?;
                    t.fg(term::color::RED)?;
                    write!(t, "{}\n", target_str(space_len, target_len))?;
                    t.reset()?;
                }

                // The last line: show note if available.
                let msg_suffix = error.msg_suffix();
                if !msg_suffix.is_empty() {
                    t.attr(term::Attr::Bold)?;
                    write!(t, "{}= note: ", prefix_spaces)?;
                    t.reset()?;
                    write!(t, "{}\n", error.msg_suffix())?;
                } else {
                    write!(t, "\n")?;
                }
                t.reset()?;
            }
        }

        if !self.file_error_map.is_empty() {
            t.attr(term::Attr::Bold)?;
            write!(t, "warning: ")?;
            t.reset()?;
            write!(
                t,
                "rustfmt may have failed to format. See previous {} errors.\n\n",
                self.warning_count(),
            )?;
        }

        Ok(())
    }
}

fn target_str(space_len: usize, target_len: usize) -> String {
    let empty_line: String = repeat(" ").take(space_len).collect();
    let overflowed: String = repeat("^").take(target_len).collect();
    empty_line + &overflowed
}

impl fmt::Display for FormatReport {
    // Prints all the formatting errors.
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for (file, errors) in &self.file_error_map {
            for error in errors {
                let prefix_space_len = error.line.to_string().len();
                let prefix_spaces: String = repeat(" ").take(1 + prefix_space_len).collect();

                let error_line_buffer = if error.line_buffer.is_empty() {
                    String::from(" ")
                } else {
                    let (space_len, target_len) = error.format_len();
                    format!(
                        "{}|\n{} | {}\n{}| {}",
                        prefix_spaces,
                        error.line,
                        error.line_buffer,
                        prefix_spaces,
                        target_str(space_len, target_len)
                    )
                };

                let error_info = format!("{} {}", error.msg_prefix(), error.kind);
                let file_info = format!("{}--> {}:{}", &prefix_spaces[1..], file, error.line);
                let msg_suffix = error.msg_suffix();
                let note = if msg_suffix.is_empty() {
                    String::new()
                } else {
                    format!("{}note= ", prefix_spaces)
                };

                write!(
                    fmt,
                    "{}\n{}\n{}\n{}{}\n",
                    error_info,
                    file_info,
                    error_line_buffer,
                    note,
                    error.msg_suffix()
                )?;
            }
        }
        if !self.file_error_map.is_empty() {
            write!(
                fmt,
                "warning: rustfmt may have failed to format. See previous {} errors.\n",
                self.warning_count(),
            )?;
        }
        Ok(())
    }
}

// Formatting which depends on the AST.
fn format_ast<F>(
    krate: &ast::Crate,
    parse_session: &mut ParseSess,
    main_file: &Path,
    config: &Config,
    codemap: &Rc<CodeMap>,
    mut after_file: F,
) -> Result<(FileMap, bool), io::Error>
where
    F: FnMut(&str, &mut StringBuffer) -> Result<bool, io::Error>,
{
    let mut result = FileMap::new();
    // diff mode: check if any files are differing
    let mut has_diff = false;

    // We always skip children for the "Plain" write mode, since there is
    // nothing to distinguish the nested module contents.
    let skip_children = config.skip_children() || config.write_mode() == config::WriteMode::Plain;
    for (path, module) in modules::list_files(krate, parse_session.codemap()) {
        if skip_children && path.as_path() != main_file {
            continue;
        }
        let path_str = path.to_str().unwrap();
        if config.verbose() {
            println!("Formatting {}", path_str);
        }
        {
            let mut visitor = FmtVisitor::from_codemap(parse_session, config);
            let filemap = visitor.codemap.lookup_char_pos(module.inner.lo()).file;
            // Format inner attributes if available.
            if !krate.attrs.is_empty() && path == main_file {
                visitor.visit_attrs(&krate.attrs, ast::AttrStyle::Inner);
            } else {
                visitor.last_pos = filemap.start_pos;
            }
            visitor.format_separate_mod(module, &*filemap);

            has_diff |= after_file(path_str, &mut visitor.buffer)?;

            result.push((path_str.to_owned(), visitor.buffer));
        }
        // Reset the error count.
        if parse_session.span_diagnostic.has_errors() {
            let silent_emitter = Box::new(EmitterWriter::new(
                Box::new(Vec::new()),
                Some(codemap.clone()),
            ));
            parse_session.span_diagnostic = Handler::with_emitter(true, false, silent_emitter);
        }
    }

    Ok((result, has_diff))
}

// Formatting done on a char by char or line by line basis.
// FIXME(#209) warn on bad license
// FIXME(#20) other stuff for parity with make tidy
fn format_lines(text: &mut StringBuffer, name: &str, config: &Config, report: &mut FormatReport) {
    // Iterate over the chars in the file map.
    let mut trims = vec![];
    let mut last_wspace: Option<usize> = None;
    let mut line_len = 0;
    let mut cur_line = 1;
    let mut newline_count = 0;
    let mut errors = vec![];
    let mut issue_seeker = BadIssueSeeker::new(config.report_todo(), config.report_fixme());
    let mut prev_char: Option<char> = None;
    let mut is_comment = false;
    let mut line_buffer = String::with_capacity(config.max_width() * 2);

    for (c, b) in text.chars() {
        if c == '\r' {
            continue;
        }

        let format_line = config.file_lines().contains_line(name, cur_line as usize);

        if format_line {
            // Add warnings for bad todos/ fixmes
            if let Some(issue) = issue_seeker.inspect(c) {
                errors.push(FormattingError {
                    line: cur_line,
                    kind: ErrorKind::BadIssue(issue),
                    is_comment: false,
                    line_buffer: String::new(),
                });
            }
        }

        if c == '\n' {
            if format_line {
                // Check for (and record) trailing whitespace.
                if let Some(lw) = last_wspace {
                    trims.push((cur_line, lw, b, line_buffer.clone()));
                    line_len -= 1;
                }

                // Check for any line width errors we couldn't correct.
                let report_error_on_line_overflow = config.error_on_line_overflow() &&
                    (config.error_on_line_overflow_comments() || !is_comment);
                if report_error_on_line_overflow && line_len > config.max_width() {
                    errors.push(FormattingError {
                        line: cur_line,
                        kind: ErrorKind::LineOverflow(line_len, config.max_width()),
                        is_comment: is_comment,
                        line_buffer: line_buffer.clone(),
                    });
                }
            }

            line_len = 0;
            cur_line += 1;
            newline_count += 1;
            last_wspace = None;
            prev_char = None;
            is_comment = false;
            line_buffer.clear();
        } else {
            newline_count = 0;
            line_len += 1;
            if c.is_whitespace() {
                if last_wspace.is_none() {
                    last_wspace = Some(b);
                }
            } else if c == '/' {
                match prev_char {
                    Some('/') => is_comment = true,
                    _ => (),
                }
                last_wspace = None;
            } else {
                last_wspace = None;
            }
            prev_char = Some(c);
            line_buffer.push(c);
        }
    }

    if newline_count > 1 {
        debug!("track truncate: {} {}", text.len, newline_count);
        let line = text.len - newline_count + 1;
        text.truncate(line);
    }

    for &(l, _, _, ref b) in &trims {
        errors.push(FormattingError {
            line: l,
            kind: ErrorKind::TrailingWhitespace,
            is_comment: false,
            line_buffer: b.clone(),
        });
    }

    report.file_error_map.insert(name.to_owned(), errors);
}

fn parse_input(
    input: Input,
    parse_session: &ParseSess,
) -> Result<ast::Crate, Option<DiagnosticBuilder>> {
    let result = match input {
        Input::File(file) => {
            let mut parser = parse::new_parser_from_file(parse_session, &file);
            parser.cfg_mods = false;
            parser.parse_crate_mod()
        }
        Input::Text(text) => {
            let mut parser =
                parse::new_parser_from_source_str(parse_session, "stdin".to_owned(), text);
            parser.cfg_mods = false;
            parser.parse_crate_mod()
        }
    };

    match result {
        Ok(c) => {
            if parse_session.span_diagnostic.has_errors() {
                // Bail out if the parser recovered from an error.
                Err(None)
            } else {
                Ok(c)
            }
        }
        Err(e) => Err(Some(e)),
    }
}

pub fn format_input<T: Write>(
    input: Input,
    config: &Config,
    mut out: Option<&mut T>,
) -> Result<(Summary, FileMap, FormatReport), (io::Error, Summary)> {
    let mut summary = Summary::new();
    if config.disable_all_formatting() {
        return Ok((summary, FileMap::new(), FormatReport::new()));
    }
    let codemap = Rc::new(CodeMap::new(FilePathMapping::empty()));

    let tty_handler =
        Handler::with_tty_emitter(ColorConfig::Auto, true, false, Some(codemap.clone()));
    let mut parse_session = ParseSess::with_span_handler(tty_handler, codemap.clone());

    let main_file = match input {
        Input::File(ref file) => file.clone(),
        Input::Text(..) => PathBuf::from("stdin"),
    };

    let krate = match parse_input(input, &parse_session) {
        Ok(krate) => krate,
        Err(diagnostic) => {
            if let Some(mut diagnostic) = diagnostic {
                diagnostic.emit();
            }
            summary.add_parsing_error();
            return Ok((summary, FileMap::new(), FormatReport::new()));
        }
    };

    if parse_session.span_diagnostic.has_errors() {
        summary.add_parsing_error();
    }

    // Suppress error output after parsing.
    let silent_emitter = Box::new(EmitterWriter::new(
        Box::new(Vec::new()),
        Some(codemap.clone()),
    ));
    parse_session.span_diagnostic = Handler::with_emitter(true, false, silent_emitter);

    let mut report = FormatReport::new();

    match format_ast(
        &krate,
        &mut parse_session,
        &main_file,
        config,
        &codemap,
        |file_name, file| {
            // For some reason, the codemap does not include terminating
            // newlines so we must add one on for each file. This is sad.
            filemap::append_newline(file);

            format_lines(file, file_name, config, &mut report);

            if let Some(ref mut out) = out {
                return filemap::write_file(file, file_name, out, config);
            }
            Ok(false)
        },
    ) {
        Ok((file_map, has_diff)) => {
            if report.has_warnings() {
                summary.add_formatting_error();
            }

            if has_diff {
                summary.add_diff();
            }

            Ok((summary, file_map, report))
        }
        Err(e) => Err((e, summary)),
    }
}

#[derive(Debug)]
pub enum Input {
    File(PathBuf),
    Text(String),
}

pub fn run(input: Input, config: &Config) -> Summary {
    let out = &mut stdout();
    output_header(out, config.write_mode()).ok();
    match format_input(input, config, Some(out)) {
        Ok((summary, _, report)) => {
            output_footer(out, config.write_mode()).ok();

            if report.has_warnings() {
                match term::stderr() {
                    Some(ref t) if isatty() && t.supports_color() => {
                        match report.print_warnings_fancy(term::stderr().unwrap()) {
                            Ok(..) => (),
                            Err(..) => panic!("Unable to write to stderr: {}", report),
                        }
                    }
                    _ => msg!("{}", report),
                }
            }

            summary
        }
        Err((msg, mut summary)) => {
            msg!("Error writing files: {}", msg);
            summary.add_operational_error();
            summary
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn indent_add_sub() {
        let indent = Indent::new(4, 8) + Indent::new(8, 12);
        assert_eq!(12, indent.block_indent);
        assert_eq!(20, indent.alignment);

        let indent = indent - Indent::new(4, 4);
        assert_eq!(8, indent.block_indent);
        assert_eq!(16, indent.alignment);
    }

    #[test]
    fn indent_add_sub_alignment() {
        let indent = Indent::new(4, 8) + 4;
        assert_eq!(4, indent.block_indent);
        assert_eq!(12, indent.alignment);

        let indent = indent - 4;
        assert_eq!(4, indent.block_indent);
        assert_eq!(8, indent.alignment);
    }

    #[test]
    fn indent_to_string_spaces() {
        let config = Config::default();
        let indent = Indent::new(4, 8);

        // 12 spaces
        assert_eq!("            ", indent.to_string(&config));
    }

    #[test]
    fn indent_to_string_hard_tabs() {
        let mut config = Config::default();
        config.set().hard_tabs(true);
        let indent = Indent::new(8, 4);

        // 2 tabs + 4 spaces
        assert_eq!("\t\t    ", indent.to_string(&config));
    }

    #[test]
    fn shape_visual_indent() {
        let config = Config::default();
        let indent = Indent::new(4, 8);
        let shape = Shape::legacy(config.max_width(), indent);
        let shape = shape.visual_indent(20);

        assert_eq!(config.max_width(), shape.width);
        assert_eq!(4, shape.indent.block_indent);
        assert_eq!(28, shape.indent.alignment);
        assert_eq!(28, shape.offset);
    }

    #[test]
    fn shape_block_indent_without_alignment() {
        let config = Config::default();
        let indent = Indent::new(4, 0);
        let shape = Shape::legacy(config.max_width(), indent);
        let shape = shape.block_indent(20);

        assert_eq!(config.max_width(), shape.width);
        assert_eq!(24, shape.indent.block_indent);
        assert_eq!(0, shape.indent.alignment);
        assert_eq!(0, shape.offset);
    }

    #[test]
    fn shape_block_indent_with_alignment() {
        let config = Config::default();
        let indent = Indent::new(4, 8);
        let shape = Shape::legacy(config.max_width(), indent);
        let shape = shape.block_indent(20);

        assert_eq!(config.max_width(), shape.width);
        assert_eq!(4, shape.indent.block_indent);
        assert_eq!(28, shape.indent.alignment);
        assert_eq!(28, shape.offset);
    }
}
