// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

// Formatting top-level items - functions, structs, enums, traits, impls.

use Indent;
use codemap::SpanUtils;
use utils::{format_mutability, format_visibility, contains_skip, end_typaram, wrap_str,
            last_line_width, format_unsafety, trim_newlines, stmt_expr, semicolon_for_expr};
use lists::{write_list, itemize_list, ListItem, ListFormatting, SeparatorTactic,
            DefinitiveListTactic, ListTactic, definitive_tactic, format_item_list};
use expr::{is_empty_block, is_simple_block_stmt, rewrite_assign_rhs, type_annotation_separator};
use comment::{FindUncommented, contains_comment};
use visitor::FmtVisitor;
use rewrite::{Rewrite, RewriteContext};
use config::{Config, BlockIndentStyle, Density, ReturnIndent, BraceStyle, FnArgLayoutStyle};
use itertools::Itertools;

use syntax::{ast, abi, ptr, codemap};
use syntax::codemap::{Span, BytePos, mk_sp};
use syntax::parse::token;
use syntax::ast::ImplItem;

// Statements of the form
// let pat: ty = init;
impl Rewrite for ast::Local {
    fn rewrite(&self, context: &RewriteContext, width: usize, offset: Indent) -> Option<String> {
        let mut result = "let ".to_owned();
        let pattern_offset = offset + result.len();
        // 1 = ;
        let pattern_width = try_opt!(width.checked_sub(pattern_offset.width() + 1));

        let pat_str = try_opt!(self.pat.rewrite(&context, pattern_width, pattern_offset));
        result.push_str(&pat_str);

        // String that is placed within the assignment pattern and expression.
        let infix = {
            let mut infix = String::new();

            if let Some(ref ty) = self.ty {
                let separator = type_annotation_separator(context.config);
                let indent = offset + last_line_width(&result) + separator.len();
                // 1 = ;
                let budget = try_opt!(width.checked_sub(indent.width() + 1));
                let rewrite = try_opt!(ty.rewrite(context, budget, indent));

                infix.push_str(separator);
                infix.push_str(&rewrite);
            }

            if self.init.is_some() {
                infix.push_str(" =");
            }

            infix
        };

        result.push_str(&infix);

        if let Some(ref ex) = self.init {
            let budget = try_opt!(width.checked_sub(context.block_indent.width() + 1));

            // 1 = trailing semicolon;
            result =
                try_opt!(rewrite_assign_rhs(&context, result, ex, budget, context.block_indent));
        }

        result.push(';');
        Some(result)
    }
}

impl<'a> FmtVisitor<'a> {
    pub fn format_foreign_mod(&mut self, fm: &ast::ForeignMod, span: Span) {
        let abi_str = ::utils::format_abi(fm.abi, self.config.force_explicit_abi);
        self.buffer.push_str(&abi_str);

        let snippet = self.snippet(span);
        let brace_pos = snippet.find_uncommented("{").unwrap();

        self.buffer.push_str("{");
        if !fm.items.is_empty() || contains_comment(&snippet[brace_pos..]) {
            // FIXME: this skips comments between the extern keyword and the opening
            // brace.
            self.last_pos = span.lo + BytePos(brace_pos as u32 + 1);
            self.block_indent = self.block_indent.block_indent(self.config);

            if fm.items.is_empty() {
                self.format_missing_no_indent(span.hi - BytePos(1));
                self.block_indent = self.block_indent.block_unindent(self.config);

                self.buffer.push_str(&self.block_indent.to_string(self.config));
            } else {
                for item in &fm.items {
                    self.format_foreign_item(&*item);
                }

                self.block_indent = self.block_indent.block_unindent(self.config);
                self.format_missing_with_indent(span.hi - BytePos(1));
            }
        }

        self.buffer.push_str("}");
        self.last_pos = span.hi;
    }

    fn format_foreign_item(&mut self, item: &ast::ForeignItem) {
        self.format_missing_with_indent(item.span.lo);
        // Drop semicolon or it will be interpreted as comment.
        // FIXME: this may be a faulty span from libsyntax.
        let span = mk_sp(item.span.lo, item.span.hi - BytePos(1));

        match item.node {
            ast::ForeignItemKind::Fn(ref fn_decl, ref generics) => {
                let indent = self.block_indent;
                let rewrite = rewrite_fn_base(&self.get_context(),
                                              indent,
                                              item.ident,
                                              fn_decl,
                                              generics,
                                              ast::Unsafety::Normal,
                                              ast::Constness::NotConst,
                                              ast::Defaultness::Final,
                                              // These are not actually rust functions,
                                              // but we format them as such.
                                              abi::Abi::Rust,
                                              &item.vis,
                                              span,
                                              false,
                                              false);

                match rewrite {
                    Some((new_fn, _)) => {
                        self.buffer.push_str(&new_fn);
                        self.buffer.push_str(";");
                    }
                    None => self.format_missing(item.span.hi),
                }
            }
            ast::ForeignItemKind::Static(ref ty, is_mutable) => {
                // FIXME(#21): we're dropping potential comments in between the
                // function keywords here.
                let vis = format_visibility(&item.vis);
                let mut_str = if is_mutable { "mut " } else { "" };
                let prefix = format!("{}static {}{}: ", vis, mut_str, item.ident);
                let offset = self.block_indent + prefix.len();
                // 1 = ;
                let width = self.config.max_width - offset.width() - 1;
                let rewrite = ty.rewrite(&self.get_context(), width, offset);

                match rewrite {
                    Some(result) => {
                        self.buffer.push_str(&prefix);
                        self.buffer.push_str(&result);
                        self.buffer.push_str(";");
                    }
                    None => self.format_missing(item.span.hi),
                }
            }
        }

        self.last_pos = item.span.hi;
    }

    pub fn rewrite_fn(&mut self,
                      indent: Indent,
                      ident: ast::Ident,
                      fd: &ast::FnDecl,
                      generics: &ast::Generics,
                      unsafety: ast::Unsafety,
                      constness: ast::Constness,
                      defaultness: ast::Defaultness,
                      abi: abi::Abi,
                      vis: &ast::Visibility,
                      span: Span,
                      block: &ast::Block)
                      -> Option<String> {
        let mut newline_brace = newline_for_brace(self.config, &generics.where_clause);
        let context = self.get_context();

        let block_snippet = self.snippet(codemap::mk_sp(block.span.lo, block.span.hi));
        let has_body = !block_snippet[1..block_snippet.len() - 1].trim().is_empty() ||
                       !context.config.fn_empty_single_line;

        let (mut result, force_newline_brace) = try_opt!(rewrite_fn_base(&context,
                                                                         indent,
                                                                         ident,
                                                                         fd,
                                                                         generics,
                                                                         unsafety,
                                                                         constness,
                                                                         defaultness,
                                                                         abi,
                                                                         vis,
                                                                         span,
                                                                         newline_brace,
                                                                         has_body));

        if self.config.fn_brace_style != BraceStyle::AlwaysNextLine && !result.contains('\n') {
            newline_brace = false;
        } else if force_newline_brace {
            newline_brace = true;
        }

        // Prepare for the function body by possibly adding a newline and
        // indent.
        // FIXME we'll miss anything between the end of the signature and the
        // start of the body, but we need more spans from the compiler to solve
        // this.
        if newline_brace {
            result.push('\n');
            result.push_str(&indent.to_string(self.config));
        } else {
            result.push(' ');
        }

        self.single_line_fn(&result, block).or_else(|| Some(result))
    }

    pub fn rewrite_required_fn(&mut self,
                               indent: Indent,
                               ident: ast::Ident,
                               sig: &ast::MethodSig,
                               span: Span)
                               -> Option<String> {
        // Drop semicolon or it will be interpreted as comment.
        let span = mk_sp(span.lo, span.hi - BytePos(1));
        let context = self.get_context();

        let (mut result, _) = try_opt!(rewrite_fn_base(&context,
                                                       indent,
                                                       ident,
                                                       &sig.decl,
                                                       &sig.generics,
                                                       sig.unsafety,
                                                       sig.constness.node,
                                                       ast::Defaultness::Final,
                                                       sig.abi,
                                                       &ast::Visibility::Inherited,
                                                       span,
                                                       false,
                                                       false));

        // Re-attach semicolon
        result.push(';');

        Some(result)
    }

    fn single_line_fn(&self, fn_str: &str, block: &ast::Block) -> Option<String> {
        if fn_str.contains('\n') {
            return None;
        }

        let codemap = self.get_context().codemap;

        if self.config.fn_empty_single_line && is_empty_block(block, codemap) &&
           self.block_indent.width() + fn_str.len() + 2 <= self.config.max_width {
            return Some(format!("{}{{}}", fn_str));
        }

        if self.config.fn_single_line && is_simple_block_stmt(block, codemap) {
            let rewrite = {
                if let Some(ref stmt) = block.stmts.first() {
                    match stmt_expr(stmt) {
                        Some(e) => {
                            let suffix = if semicolon_for_expr(e) { ";" } else { "" };

                            e.rewrite(&self.get_context(),
                                         self.config.max_width - self.block_indent.width(),
                                         self.block_indent)
                                .map(|s| s + suffix)
                                .or_else(|| Some(self.snippet(e.span)))
                        }
                        None => {
                            stmt.rewrite(&self.get_context(),
                                         self.config.max_width - self.block_indent.width(),
                                         self.block_indent)
                        }
                    }
                } else {
                    None
                }
            };

            if let Some(res) = rewrite {
                let width = self.block_indent.width() + fn_str.len() + res.len() + 4;
                if !res.contains('\n') && width <= self.config.max_width {
                    return Some(format!("{}{{ {} }}", fn_str, res));
                }
            }
        }

        None
    }

    pub fn visit_enum(&mut self,
                      ident: ast::Ident,
                      vis: &ast::Visibility,
                      enum_def: &ast::EnumDef,
                      generics: &ast::Generics,
                      span: Span) {
        self.buffer.push_str(&format_header("enum ", ident, vis));

        let enum_snippet = self.snippet(span);
        let brace_pos = enum_snippet.find_uncommented("{").unwrap();
        let body_start = span.lo + BytePos(brace_pos as u32 + 1);
        let generics_str = format_generics(&self.get_context(),
                                           generics,
                                           "{",
                                           "{",
                                           self.config.item_brace_style,
                                           enum_def.variants.is_empty(),
                                           self.block_indent,
                                           self.block_indent.block_indent(self.config),
                                           mk_sp(span.lo, body_start))
            .unwrap();
        self.buffer.push_str(&generics_str);

        self.last_pos = body_start;

        self.block_indent = self.block_indent.block_indent(self.config);
        let variant_list = self.format_variant_list(enum_def, body_start, span.hi - BytePos(1));
        match variant_list {
            Some(ref body_str) => self.buffer.push_str(body_str),
            None => {
                if contains_comment(&enum_snippet[brace_pos..]) {
                    self.format_missing_no_indent(span.hi - BytePos(1))
                } else {
                    self.format_missing(span.hi - BytePos(1))
                }
            }
        }
        self.block_indent = self.block_indent.block_unindent(self.config);

        if variant_list.is_some() || contains_comment(&enum_snippet[brace_pos..]) {
            self.buffer.push_str(&self.block_indent.to_string(self.config));
        }
        self.buffer.push_str("}");
        self.last_pos = span.hi;
    }

    // Format the body of an enum definition
    fn format_variant_list(&self,
                           enum_def: &ast::EnumDef,
                           body_lo: BytePos,
                           body_hi: BytePos)
                           -> Option<String> {
        if enum_def.variants.is_empty() {
            return None;
        }
        let mut result = String::with_capacity(1024);
        result.push('\n');
        let indentation = self.block_indent.to_string(self.config);
        result.push_str(&indentation);

        let items = itemize_list(self.codemap,
                                 enum_def.variants.iter(),
                                 "}",
                                 |f| if !f.node.attrs.is_empty() {
                                     f.node.attrs[0].span.lo
                                 } else {
                                     f.span.lo
                                 },
                                 |f| f.span.hi,
                                 |f| self.format_variant(f),
                                 body_lo,
                                 body_hi);

        let budget = self.config.max_width - self.block_indent.width() - 2;
        let fmt = ListFormatting {
            tactic: DefinitiveListTactic::Vertical,
            separator: ",",
            trailing_separator: SeparatorTactic::from_bool(self.config.enum_trailing_comma),
            indent: self.block_indent,
            width: budget,
            ends_with_newline: true,
            config: self.config,
        };

        let list = try_opt!(write_list(items, &fmt));
        result.push_str(&list);
        result.push('\n');
        Some(result)
    }

    // Variant of an enum.
    fn format_variant(&self, field: &ast::Variant) -> Option<String> {
        if contains_skip(&field.node.attrs) {
            let lo = field.node.attrs[0].span.lo;
            let span = mk_sp(lo, field.span.hi);
            return Some(self.snippet(span));
        }

        let indent = self.block_indent;
        let mut result = try_opt!(field.node
            .attrs
            .rewrite(&self.get_context(),
                     self.config.max_width - indent.width(),
                     indent));
        if !result.is_empty() {
            result.push('\n');
            result.push_str(&indent.to_string(self.config));
        }

        let context = self.get_context();
        let variant_body = match field.node.data {
            ast::VariantData::Tuple(..) |
            ast::VariantData::Struct(..) => {
                // FIXME: Should limit the width, as we have a trailing comma
                format_struct(&context,
                              "",
                              field.node.name,
                              &ast::Visibility::Inherited,
                              &field.node.data,
                              None,
                              field.span,
                              indent,
                              Some(self.config.struct_variant_width))
            }
            ast::VariantData::Unit(..) => {
                let tag = if let Some(ref expr) = field.node.disr_expr {
                    format!("{} = {}", field.node.name, self.snippet(expr.span))
                } else {
                    field.node.name.to_string()
                };

                wrap_str(tag,
                         self.config.max_width,
                         self.config.max_width - indent.width(),
                         indent)
            }
        };

        if let Some(variant_str) = variant_body {
            result.push_str(&variant_str);
            Some(result)
        } else {
            None
        }
    }
}

pub fn format_impl(context: &RewriteContext, item: &ast::Item, offset: Indent) -> Option<String> {
    if let ast::ItemKind::Impl(_, _, ref generics, ref trait_ref, _, ref items) = item.node {
        let mut result = String::new();

        // First try to format the ref and type without a split at the 'for'.
        let mut ref_and_type = try_opt!(format_impl_ref_and_type(context, item, offset, false));

        // If there is a line break present in the first result format it again
        // with a split at the 'for'. Skip this if there is no trait ref and
        // therefore no 'for'.
        if let Some(_) = *trait_ref {
            if ref_and_type.contains('\n') {
                ref_and_type = try_opt!(format_impl_ref_and_type(context, item, offset, true));
            }
        }
        result.push_str(&ref_and_type);

        let where_budget = try_opt!(context.config.max_width.checked_sub(last_line_width(&result)));
        let where_clause_str = try_opt!(rewrite_where_clause(context,
                                                             &generics.where_clause,
                                                             context.config,
                                                             context.config.item_brace_style,
                                                             context.block_indent,
                                                             where_budget,
                                                             context.config.where_density,
                                                             "{",
                                                             true,
                                                             None));

        if try_opt!(is_impl_single_line(context, &items, &result, &where_clause_str, &item)) {
            result.push_str(&where_clause_str);
            if where_clause_str.contains('\n') {
                let white_space = offset.to_string(context.config);
                result.push_str(&format!("\n{}{{\n{}}}", &white_space, &white_space));
            } else {
                result.push_str(" {}");
            }
            return Some(result);
        }

        if !where_clause_str.is_empty() && !where_clause_str.contains('\n') {
            result.push('\n');
            let width = context.block_indent.width() + context.config.tab_spaces - 1;
            let where_indent = Indent::new(0, width);
            result.push_str(&where_indent.to_string(context.config));
        }
        result.push_str(&where_clause_str);

        match context.config.item_brace_style {
            BraceStyle::AlwaysNextLine => {
                result.push('\n');
                result.push_str(&offset.to_string(context.config));
            }
            BraceStyle::PreferSameLine => result.push(' '),
            BraceStyle::SameLineWhere => {
                if !where_clause_str.is_empty() {
                    result.push('\n');
                    result.push_str(&offset.to_string(context.config));
                } else {
                    result.push(' ');
                }
            }
        }

        result.push('{');

        let snippet = context.snippet(item.span);
        let open_pos = try_opt!(snippet.find_uncommented("{")) + 1;

        if !items.is_empty() || contains_comment(&snippet[open_pos..]) {
            let mut visitor = FmtVisitor::from_codemap(context.parse_session, context.config);
            visitor.block_indent = context.block_indent.block_indent(context.config);
            visitor.last_pos = item.span.lo + BytePos(open_pos as u32);

            for item in items {
                visitor.visit_impl_item(item);
            }

            visitor.format_missing(item.span.hi - BytePos(1));

            let inner_indent_str = visitor.block_indent.to_string(context.config);
            let outer_indent_str = context.block_indent.to_string(context.config);

            result.push('\n');
            result.push_str(&inner_indent_str);
            result.push_str(trim_newlines(visitor.buffer.to_string().trim()));
            result.push('\n');
            result.push_str(&outer_indent_str);
        }

        if result.chars().last().unwrap() == '{' {
            result.push('\n');
        }
        result.push('}');

        Some(result)
    } else {
        unreachable!();
    }
}

fn is_impl_single_line(context: &RewriteContext,
                       items: &[ImplItem],
                       result: &str,
                       where_clause_str: &str,
                       item: &ast::Item)
                       -> Option<bool> {
    let snippet = context.snippet(item.span);
    let open_pos = try_opt!(snippet.find_uncommented("{")) + 1;

    Some(context.config.impl_empty_single_line && items.is_empty() &&
         result.len() + where_clause_str.len() <= context.config.max_width &&
         !contains_comment(&snippet[open_pos..]))
}

fn format_impl_ref_and_type(context: &RewriteContext,
                            item: &ast::Item,
                            offset: Indent,
                            split_at_for: bool)
                            -> Option<String> {
    if let ast::ItemKind::Impl(unsafety, polarity, ref generics, ref trait_ref, ref self_ty, _) =
        item.node {
        let mut result = String::new();

        result.push_str(&*format_visibility(&item.vis));
        result.push_str(format_unsafety(unsafety));
        result.push_str("impl");

        let lo = context.codemap.span_after(item.span, "impl");
        let hi = match *trait_ref {
            Some(ref tr) => tr.path.span.lo,
            None => self_ty.span.lo,
        };
        let generics_str = try_opt!(rewrite_generics(context,
                                                     generics,
                                                     offset,
                                                     context.config.max_width,
                                                     offset + result.len(),
                                                     mk_sp(lo, hi)));
        result.push_str(&generics_str);

        result.push(' ');
        if polarity == ast::ImplPolarity::Negative {
            result.push('!');
        }
        if let Some(ref trait_ref) = *trait_ref {
            let budget = try_opt!(context.config.max_width.checked_sub(result.len()));
            let indent = offset + result.len();
            result.push_str(&*try_opt!(trait_ref.rewrite(context, budget, indent)));

            if split_at_for {
                result.push('\n');

                // Add indentation of one additional tab.
                let width = context.block_indent.width() + context.config.tab_spaces;
                let for_indent = Indent::new(0, width);
                result.push_str(&for_indent.to_string(context.config));

                result.push_str("for ");
            } else {
                result.push_str(" for ");
            }
        }

        let mut used_space = last_line_width(&result);
        if generics.where_clause.predicates.is_empty() {
            // If there is no where clause adapt budget for type formatting to take space and curly
            // brace into account.
            match context.config.item_brace_style {
                BraceStyle::AlwaysNextLine => {}
                BraceStyle::PreferSameLine => used_space += 2,
                BraceStyle::SameLineWhere => used_space += 2,
            }
        }

        let budget = try_opt!(context.config.max_width.checked_sub(used_space));
        let indent = offset + result.len();
        result.push_str(&*try_opt!(self_ty.rewrite(context, budget, indent)));

        Some(result)
    } else {
        unreachable!();
    }
}

pub fn format_struct(context: &RewriteContext,
                     item_name: &str,
                     ident: ast::Ident,
                     vis: &ast::Visibility,
                     struct_def: &ast::VariantData,
                     generics: Option<&ast::Generics>,
                     span: Span,
                     offset: Indent,
                     one_line_width: Option<usize>)
                     -> Option<String> {
    match *struct_def {
        ast::VariantData::Unit(..) => Some(format_unit_struct(item_name, ident, vis)),
        ast::VariantData::Tuple(ref fields, _) => {
            format_tuple_struct(context,
                                item_name,
                                ident,
                                vis,
                                fields,
                                generics,
                                span,
                                offset)
        }
        ast::VariantData::Struct(ref fields, _) => {
            format_struct_struct(context,
                                 item_name,
                                 ident,
                                 vis,
                                 fields,
                                 generics,
                                 span,
                                 offset,
                                 one_line_width)
        }
    }
}

pub fn format_trait(context: &RewriteContext, item: &ast::Item, offset: Indent) -> Option<String> {
    if let ast::ItemKind::Trait(unsafety, ref generics, ref type_param_bounds, ref trait_items) =
        item.node {
        let mut result = String::new();
        let header = format!("{}{}trait {}",
                             format_visibility(&item.vis),
                             format_unsafety(unsafety),
                             item.ident);

        result.push_str(&header);

        let body_lo = context.codemap.span_after(item.span, "{");

        let generics_str = try_opt!(rewrite_generics(context,
                                                     generics,
                                                     offset,
                                                     context.config.max_width,
                                                     offset + result.len(),
                                                     mk_sp(item.span.lo, body_lo)));
        result.push_str(&generics_str);

        let trait_bound_str = try_opt!(rewrite_trait_bounds(context,
                                                            type_param_bounds,
                                                            offset,
                                                            context.config.max_width));
        // If the trait, generics, and trait bound cannot fit on the same line,
        // put the trait bounds on an indented new line
        if offset.width() + last_line_width(&result) + trait_bound_str.len() >
           context.config.ideal_width {
            result.push('\n');
            let width = context.block_indent.width() + context.config.tab_spaces;
            let trait_indent = Indent::new(0, width);
            result.push_str(&trait_indent.to_string(context.config));
        }
        result.push_str(&trait_bound_str);

        let has_body = !trait_items.is_empty();

        let where_density =
            if (context.config.where_density == Density::Compressed &&
                (!result.contains('\n') ||
                 context.config.fn_args_layout == FnArgLayoutStyle::Block)) ||
               (context.config.fn_args_layout == FnArgLayoutStyle::Block && result.is_empty()) ||
               (context.config.where_density == Density::CompressedIfEmpty && !has_body &&
                !result.contains('\n')) {
                Density::Compressed
            } else {
                Density::Tall
            };

        let where_budget = try_opt!(context.config
            .max_width
            .checked_sub(last_line_width(&result)));
        let where_clause_str = try_opt!(rewrite_where_clause(context,
                                                             &generics.where_clause,
                                                             context.config,
                                                             context.config.item_brace_style,
                                                             context.block_indent,
                                                             where_budget,
                                                             where_density,
                                                             "{",
                                                             has_body,
                                                             None));
        // If the where clause cannot fit on the same line,
        // put the where clause on a new line
        if !where_clause_str.contains('\n') &&
           last_line_width(&result) + where_clause_str.len() + offset.width() >
           context.config.ideal_width {
            result.push('\n');
            let width = context.block_indent.width() + context.config.tab_spaces - 1;
            let where_indent = Indent::new(0, width);
            result.push_str(&where_indent.to_string(context.config));
        }
        result.push_str(&where_clause_str);

        match context.config.item_brace_style {
            BraceStyle::AlwaysNextLine => {
                result.push('\n');
                result.push_str(&offset.to_string(context.config));
            }
            BraceStyle::PreferSameLine => result.push(' '),
            BraceStyle::SameLineWhere => {
                if !where_clause_str.is_empty() &&
                   (!trait_items.is_empty() || result.contains('\n')) {
                    result.push('\n');
                    result.push_str(&offset.to_string(context.config));
                } else {
                    result.push(' ');
                }
            }
        }
        result.push('{');

        let snippet = context.snippet(item.span);
        let open_pos = try_opt!(snippet.find_uncommented("{")) + 1;

        if !trait_items.is_empty() || contains_comment(&snippet[open_pos..]) {
            let mut visitor = FmtVisitor::from_codemap(context.parse_session, context.config);
            visitor.block_indent = context.block_indent.block_indent(context.config);
            visitor.last_pos = item.span.lo + BytePos(open_pos as u32);

            for item in trait_items {
                visitor.visit_trait_item(item);
            }

            visitor.format_missing(item.span.hi - BytePos(1));

            let inner_indent_str = visitor.block_indent.to_string(context.config);
            let outer_indent_str = context.block_indent.to_string(context.config);

            result.push('\n');
            result.push_str(&inner_indent_str);
            result.push_str(trim_newlines(visitor.buffer.to_string().trim()));
            result.push('\n');
            result.push_str(&outer_indent_str);
        } else if result.contains('\n') {
            result.push('\n');
        }

        result.push('}');
        Some(result)
    } else {
        unreachable!();
    }
}

fn format_unit_struct(item_name: &str, ident: ast::Ident, vis: &ast::Visibility) -> String {
    format!("{};", format_header(item_name, ident, vis))
}

fn format_struct_struct(context: &RewriteContext,
                        item_name: &str,
                        ident: ast::Ident,
                        vis: &ast::Visibility,
                        fields: &[ast::StructField],
                        generics: Option<&ast::Generics>,
                        span: Span,
                        offset: Indent,
                        one_line_width: Option<usize>)
                        -> Option<String> {
    let mut result = String::with_capacity(1024);

    let header_str = format_header(item_name, ident, vis);
    result.push_str(&header_str);

    let body_lo = context.codemap.span_after(span, "{");

    let generics_str = match generics {
        Some(g) => {
            try_opt!(format_generics(context,
                                     g,
                                     "{",
                                     "{",
                                     context.config.item_brace_style,
                                     fields.is_empty(),
                                     offset,
                                     offset + header_str.len(),
                                     mk_sp(span.lo, body_lo)))
        }
        None => {
            if context.config.item_brace_style == BraceStyle::AlwaysNextLine && !fields.is_empty() {
                format!("\n{}{{", context.block_indent.to_string(context.config))
            } else {
                " {".to_owned()
            }
        }
    };
    result.push_str(&generics_str);

    // FIXME(#919): properly format empty structs and their comments.
    if fields.is_empty() {
        result.push_str(&context.snippet(mk_sp(body_lo, span.hi)));
        return Some(result);
    }

    let item_indent = offset.block_indent(context.config);
    // 1 = ","
    let item_budget = try_opt!(context.config.max_width.checked_sub(item_indent.width() + 1));

    let items = itemize_list(context.codemap,
                             fields.iter(),
                             "}",
                             |field| {
        // Include attributes and doc comments, if present
        if !field.attrs.is_empty() {
            field.attrs[0].span.lo
        } else {
            field.span.lo
        }
    },
                             |field| field.ty.span.hi,
                             |field| field.rewrite(context, item_budget, item_indent),
                             context.codemap.span_after(span, "{"),
                             span.hi)
        .collect::<Vec<_>>();
    // 1 = ,
    let budget = context.config.max_width - offset.width() + context.config.tab_spaces - 1;

    let tactic = match one_line_width {
        Some(w) => definitive_tactic(&items, ListTactic::LimitedHorizontalVertical(w), budget),
        None => DefinitiveListTactic::Vertical,
    };

    let fmt = ListFormatting {
        tactic: tactic,
        separator: ",",
        trailing_separator: context.config.struct_trailing_comma,
        indent: item_indent,
        width: budget,
        ends_with_newline: true,
        config: context.config,
    };
    let items_str = try_opt!(write_list(&items, &fmt));
    if one_line_width.is_some() && !items_str.contains('\n') {
        Some(format!("{} {} }}", result, items_str))
    } else {
        Some(format!("{}\n{}{}\n{}}}",
                     result,
                     offset.block_indent(context.config).to_string(context.config),
                     items_str,
                     offset.to_string(context.config)))
    }
}

fn format_tuple_struct(context: &RewriteContext,
                       item_name: &str,
                       ident: ast::Ident,
                       vis: &ast::Visibility,
                       fields: &[ast::StructField],
                       generics: Option<&ast::Generics>,
                       span: Span,
                       offset: Indent)
                       -> Option<String> {
    let mut result = String::with_capacity(1024);

    let header_str = format_header(item_name, ident, vis);
    result.push_str(&header_str);

    // FIXME(#919): don't lose comments on empty tuple structs.
    let body_lo = if fields.is_empty() {
        span.hi
    } else {
        fields[0].span.lo
    };

    let where_clause_str = match generics {
        Some(generics) => {
            let generics_str = try_opt!(rewrite_generics(context,
                                                         generics,
                                                         offset,
                                                         context.config.max_width,
                                                         offset + header_str.len(),
                                                         mk_sp(span.lo, body_lo)));
            result.push_str(&generics_str);

            let where_budget = try_opt!(context.config
                .max_width
                .checked_sub(last_line_width(&result)));
            try_opt!(rewrite_where_clause(context,
                                          &generics.where_clause,
                                          context.config,
                                          context.config.item_brace_style,
                                          context.block_indent,
                                          where_budget,
                                          Density::Compressed,
                                          ";",
                                          false,
                                          None))
        }
        None => "".to_owned(),
    };
    result.push('(');

    let item_indent = context.block_indent + result.len();
    // 2 = ");"
    let item_budget = try_opt!(context.config.max_width.checked_sub(item_indent.width() + 2));

    let items = itemize_list(context.codemap,
                             fields.iter(),
                             ")",
                             |field| {
        // Include attributes and doc comments, if present
        if !field.attrs.is_empty() {
            field.attrs[0].span.lo
        } else {
            field.span.lo
        }
    },
                             |field| field.ty.span.hi,
                             |field| field.rewrite(context, item_budget, item_indent),
                             context.codemap.span_after(span, "("),
                             span.hi);
    let body = try_opt!(format_item_list(items, item_budget, item_indent, context.config));

    if context.config.spaces_within_parens && body.len() > 0 {
        result.push(' ');
    }

    result.push_str(&body);

    if context.config.spaces_within_parens && body.len() > 0 {
        result.push(' ');
    }

    result.push(')');

    if !where_clause_str.is_empty() && !where_clause_str.contains('\n') &&
       (result.contains('\n') ||
        context.block_indent.width() + result.len() + where_clause_str.len() + 1 >
        context.config.max_width) {
        // We need to put the where clause on a new line, but we didn'to_string
        // know that earlier, so the where clause will not be indented properly.
        result.push('\n');
        result.push_str(&(context.block_indent + (context.config.tab_spaces - 1))
            .to_string(context.config));
    }
    result.push_str(&where_clause_str);

    Some(result)
}

pub fn rewrite_type_alias(context: &RewriteContext,
                          indent: Indent,
                          ident: ast::Ident,
                          ty: &ast::Ty,
                          generics: &ast::Generics,
                          vis: &ast::Visibility,
                          span: Span)
                          -> Option<String> {
    let mut result = String::new();

    result.push_str(&format_visibility(vis));
    result.push_str("type ");
    result.push_str(&ident.to_string());

    let generics_indent = indent + result.len();
    let generics_span = mk_sp(context.codemap.span_after(span, "type"), ty.span.lo);
    let generics_width = context.config.max_width - " =".len();
    let generics_str = try_opt!(rewrite_generics(context,
                                                 generics,
                                                 indent,
                                                 generics_width,
                                                 generics_indent,
                                                 generics_span));

    result.push_str(&generics_str);

    let where_budget = try_opt!(context.config
        .max_width
        .checked_sub(last_line_width(&result)));
    let where_clause_str = try_opt!(rewrite_where_clause(context,
                                                         &generics.where_clause,
                                                         context.config,
                                                         context.config.item_brace_style,
                                                         indent,
                                                         where_budget,
                                                         context.config.where_density,
                                                         "=",
                                                         false,
                                                         Some(span.hi)));
    result.push_str(&where_clause_str);
    result.push_str(" = ");

    let line_width = last_line_width(&result);
    // This checked_sub may fail as the extra space after '=' is not taken into account
    // In that case the budget is set to 0 which will make ty.rewrite retry on a new line
    let budget = context.config
        .max_width
        .checked_sub(indent.width() + line_width + ";".len())
        .unwrap_or(0);
    let type_indent = indent + line_width;
    // Try to fit the type on the same line
    let ty_str = try_opt!(ty.rewrite(context, budget, type_indent)
        .or_else(|| {
            // The line was too short, try to put the type on the next line

            // Remove the space after '='
            result.pop();
            let type_indent = indent.block_indent(context.config);
            result.push('\n');
            result.push_str(&type_indent.to_string(context.config));
            let budget = try_opt!(context.config
                .max_width
                .checked_sub(type_indent.width() + ";".len()));
            ty.rewrite(context, budget, type_indent)
        }));
    result.push_str(&ty_str);
    result.push_str(";");
    Some(result)
}

fn type_annotation_spacing(config: &Config) -> (&str, &str) {
    (if config.space_before_type_annotation {
         " "
     } else {
         ""
     },
     if config.space_after_type_annotation_colon {
         " "
     } else {
         ""
     })
}

impl Rewrite for ast::StructField {
    fn rewrite(&self, context: &RewriteContext, width: usize, offset: Indent) -> Option<String> {
        if contains_skip(&self.attrs) {
            let span = context.snippet(mk_sp(self.attrs[0].span.lo, self.span.hi));
            return wrap_str(span, context.config.max_width, width, offset);
        }

        let name = self.ident;
        let vis = format_visibility(&self.vis);
        let mut attr_str = try_opt!(self.attrs
            .rewrite(context, context.config.max_width - offset.width(), offset));
        if !attr_str.is_empty() {
            attr_str.push('\n');
            attr_str.push_str(&offset.to_string(context.config));
        }

        let type_annotation_spacing = type_annotation_spacing(context.config);
        let result = match name {
            Some(name) => {
                format!("{}{}{}{}:{}",
                        attr_str,
                        vis,
                        name,
                        type_annotation_spacing.0,
                        type_annotation_spacing.1)
            }
            None => format!("{}{}", attr_str, vis),
        };

        let last_line_width = last_line_width(&result);
        let budget = try_opt!(width.checked_sub(last_line_width));
        let rewrite = try_opt!(self.ty.rewrite(context, budget, offset + last_line_width));
        Some(result + &rewrite)
    }
}

pub fn rewrite_static(prefix: &str,
                      vis: &ast::Visibility,
                      ident: ast::Ident,
                      ty: &ast::Ty,
                      mutability: ast::Mutability,
                      expr_opt: Option<&ptr::P<ast::Expr>>,
                      context: &RewriteContext)
                      -> Option<String> {
    let type_annotation_spacing = type_annotation_spacing(context.config);
    let prefix = format!("{}{} {}{}{}:{}",
                         format_visibility(vis),
                         prefix,
                         format_mutability(mutability),
                         ident,
                         type_annotation_spacing.0,
                         type_annotation_spacing.1);
    // 2 = " =".len()
    let ty_str = try_opt!(ty.rewrite(context,
                                     context.config.max_width - context.block_indent.width() -
                                     prefix.len() - 2,
                                     context.block_indent));

    if let Some(expr) = expr_opt {
        let lhs = format!("{}{} =", prefix, ty_str);
        // 1 = ;
        let remaining_width = context.config.max_width - context.block_indent.width() - 1;
        rewrite_assign_rhs(context, lhs, expr, remaining_width, context.block_indent)
            .map(|s| s + ";")
    } else {
        let lhs = format!("{}{};", prefix, ty_str);
        Some(lhs)
    }
}

pub fn rewrite_associated_type(ident: ast::Ident,
                               ty_opt: Option<&ptr::P<ast::Ty>>,
                               ty_param_bounds_opt: Option<&ast::TyParamBounds>,
                               context: &RewriteContext,
                               indent: Indent)
                               -> Option<String> {
    let prefix = format!("type {}", ident);

    let type_bounds_str = if let Some(ty_param_bounds) = ty_param_bounds_opt {
        let bounds: &[_] = ty_param_bounds;
        let bound_str = try_opt!(bounds.iter()
            .map(|ty_bound| ty_bound.rewrite(context, context.config.max_width, indent))
            .intersperse(Some(" + ".to_string()))
            .collect::<Option<String>>());
        if bounds.len() > 0 {
            format!(": {}", bound_str)
        } else {
            String::new()
        }
    } else {
        String::new()
    };

    if let Some(ty) = ty_opt {
        let ty_str = try_opt!(ty.rewrite(context,
                                         context.config.max_width - context.block_indent.width() -
                                         prefix.len() -
                                         2,
                                         context.block_indent));
        Some(format!("{} = {};", prefix, ty_str))
    } else {
        Some(format!("{}{};", prefix, type_bounds_str))
    }
}

impl Rewrite for ast::FunctionRetTy {
    fn rewrite(&self, context: &RewriteContext, width: usize, offset: Indent) -> Option<String> {
        match *self {
            ast::FunctionRetTy::Default(_) => Some(String::new()),
            ast::FunctionRetTy::Ty(ref ty) => {
                let inner_width = try_opt!(width.checked_sub(3));
                ty.rewrite(context, inner_width, offset + 3).map(|r| format!("-> {}", r))
            }
        }
    }
}

impl Rewrite for ast::Arg {
    fn rewrite(&self, context: &RewriteContext, width: usize, offset: Indent) -> Option<String> {
        if is_named_arg(self) {
            let mut result = try_opt!(self.pat.rewrite(context, width, offset));

            if self.ty.node != ast::TyKind::Infer {
                if context.config.space_before_type_annotation {
                    result.push_str(" ");
                }
                result.push_str(":");
                if context.config.space_after_type_annotation_colon {
                    result.push_str(" ");
                }
                let max_width = try_opt!(width.checked_sub(result.len()));
                let ty_str = try_opt!(self.ty.rewrite(context, max_width, offset + result.len()));
                result.push_str(&ty_str);
            }

            Some(result)
        } else {
            self.ty.rewrite(context, width, offset)
        }
    }
}

fn rewrite_explicit_self(explicit_self: &ast::ExplicitSelf,
                         args: &[ast::Arg],
                         context: &RewriteContext)
                         -> Option<String> {
    match explicit_self.node {
        ast::SelfKind::Region(lt, m) => {
            let mut_str = format_mutability(m);
            match lt {
                Some(ref l) => {
                    let lifetime_str =
                        try_opt!(l.rewrite(context, usize::max_value(), Indent::empty()));
                    Some(format!("&{} {}self", lifetime_str, mut_str))
                }
                None => Some(format!("&{}self", mut_str)),
            }
        }
        ast::SelfKind::Explicit(ref ty, _) => {
            assert!(!args.is_empty(), "&[ast::Arg] shouldn't be empty.");

            let mutability = explicit_self_mutability(&args[0]);
            let type_str = try_opt!(ty.rewrite(context, usize::max_value(), Indent::empty()));

            Some(format!("{}self: {}", format_mutability(mutability), type_str))
        }
        ast::SelfKind::Value(_) => {
            assert!(!args.is_empty(), "&[ast::Arg] shouldn't be empty.");

            let mutability = explicit_self_mutability(&args[0]);

            Some(format!("{}self", format_mutability(mutability)))
        }
    }
}

// Hacky solution caused by absence of `Mutability` in `SelfValue` and
// `SelfExplicit` variants of `ast::ExplicitSelf_`.
fn explicit_self_mutability(arg: &ast::Arg) -> ast::Mutability {
    if let ast::PatKind::Ident(ast::BindingMode::ByValue(mutability), _, _) = arg.pat.node {
        mutability
    } else {
        unreachable!()
    }
}

pub fn span_lo_for_arg(arg: &ast::Arg) -> BytePos {
    if is_named_arg(arg) {
        arg.pat.span.lo
    } else {
        arg.ty.span.lo
    }
}

pub fn span_hi_for_arg(arg: &ast::Arg) -> BytePos {
    match arg.ty.node {
        ast::TyKind::Infer if is_named_arg(arg) => arg.pat.span.hi,
        _ => arg.ty.span.hi,
    }
}

pub fn is_named_arg(arg: &ast::Arg) -> bool {
    if let ast::PatKind::Ident(_, ident, _) = arg.pat.node {
        ident.node != token::keywords::Invalid.ident()
    } else {
        true
    }
}

fn span_for_return(ret: &ast::FunctionRetTy) -> Span {
    match *ret {
        ast::FunctionRetTy::Default(ref span) => span.clone(),
        ast::FunctionRetTy::Ty(ref ty) => ty.span,
    }
}

fn span_for_ty_param(ty: &ast::TyParam) -> Span {
    // Note that ty.span is the span for ty.ident, not the whole item.
    let lo = ty.span.lo;
    if let Some(ref def) = ty.default {
        return mk_sp(lo, def.span.hi);
    }
    if ty.bounds.is_empty() {
        return ty.span;
    }
    let hi = match ty.bounds[ty.bounds.len() - 1] {
        ast::TyParamBound::TraitTyParamBound(ref ptr, _) => ptr.span.hi,
        ast::TyParamBound::RegionTyParamBound(ref l) => l.span.hi,
    };
    mk_sp(lo, hi)
}

fn span_for_where_pred(pred: &ast::WherePredicate) -> Span {
    match *pred {
        ast::WherePredicate::BoundPredicate(ref p) => p.span,
        ast::WherePredicate::RegionPredicate(ref p) => p.span,
        ast::WherePredicate::EqPredicate(ref p) => p.span,
    }
}

// Return type is (result, force_new_line_for_brace)
fn rewrite_fn_base(context: &RewriteContext,
                   indent: Indent,
                   ident: ast::Ident,
                   fd: &ast::FnDecl,
                   generics: &ast::Generics,
                   unsafety: ast::Unsafety,
                   constness: ast::Constness,
                   defaultness: ast::Defaultness,
                   abi: abi::Abi,
                   vis: &ast::Visibility,
                   span: Span,
                   newline_brace: bool,
                   has_body: bool)
                   -> Option<(String, bool)> {
    let mut force_new_line_for_brace = false;
    // FIXME we'll lose any comments in between parts of the function decl, but
    // anyone who comments there probably deserves what they get.

    let where_clause = &generics.where_clause;

    let mut result = String::with_capacity(1024);
    // Vis unsafety abi.
    result.push_str(&*format_visibility(vis));

    if let ast::Defaultness::Default = defaultness {
        result.push_str("default ");
    }

    if let ast::Constness::Const = constness {
        result.push_str("const ");
    }

    result.push_str(::utils::format_unsafety(unsafety));

    if abi != abi::Abi::Rust {
        result.push_str(&::utils::format_abi(abi, context.config.force_explicit_abi));
    }

    // fn foo
    result.push_str("fn ");
    result.push_str(&ident.to_string());

    // Generics.
    let generics_indent = indent + result.len();
    let generics_span = mk_sp(span.lo, span_for_return(&fd.output).lo);
    let generics_str = try_opt!(rewrite_generics(context,
                                                 generics,
                                                 indent,
                                                 context.config.max_width,
                                                 generics_indent,
                                                 generics_span));
    result.push_str(&generics_str);

    // Note that if the width and indent really matter, we'll re-layout the
    // return type later anyway.
    let ret_str = try_opt!(fd.output
        .rewrite(&context, context.config.max_width - indent.width(), indent));

    let multi_line_ret_str = ret_str.contains('\n');
    let ret_str_len = if multi_line_ret_str { 0 } else { ret_str.len() };

    // Args.
    let (mut one_line_budget, mut multi_line_budget, mut arg_indent) =
        try_opt!(compute_budgets_for_args(context, &result, indent, ret_str_len, newline_brace));

    if context.config.fn_args_layout == FnArgLayoutStyle::Block ||
       context.config.fn_args_layout == FnArgLayoutStyle::BlockAlways {
        arg_indent = indent.block_indent(context.config);
        multi_line_budget = context.config.max_width - arg_indent.width();
    }

    debug!("rewrite_fn: one_line_budget: {}, multi_line_budget: {}, arg_indent: {:?}",
           one_line_budget,
           multi_line_budget,
           arg_indent);

    // Check if vertical layout was forced by compute_budget_for_args.
    if one_line_budget == 0 {
        if context.config.fn_args_paren_newline {
            result.push('\n');
            result.push_str(&arg_indent.to_string(context.config));
            arg_indent = arg_indent + 1; // extra space for `(`
            result.push('(');
            if context.config.spaces_within_parens && fd.inputs.len() > 0 {
                result.push(' ')
            }
        } else {
            result.push_str("(\n");
            result.push_str(&arg_indent.to_string(context.config));
        }
    } else {
        result.push('(');
        if context.config.spaces_within_parens && fd.inputs.len() > 0 {
            result.push(' ')
        }
    }

    if multi_line_ret_str {
        one_line_budget = 0;
    }

    // A conservative estimation, to goal is to be over all parens in generics
    let args_start = generics.ty_params
        .last()
        .map_or(span.lo, |tp| end_typaram(tp));
    let args_span = mk_sp(context.codemap.span_after(mk_sp(args_start, span.hi), "("),
                          span_for_return(&fd.output).lo);
    let arg_str = try_opt!(rewrite_args(context,
                                        &fd.inputs,
                                        fd.get_self().as_ref(),
                                        one_line_budget,
                                        multi_line_budget,
                                        indent,
                                        arg_indent,
                                        args_span,
                                        fd.variadic));

    let multi_line_arg_str = arg_str.contains('\n');

    let put_args_in_block = match context.config.fn_args_layout {
        FnArgLayoutStyle::Block => multi_line_arg_str,
        FnArgLayoutStyle::BlockAlways => true,
        _ => false,
    } && !fd.inputs.is_empty();

    if put_args_in_block {
        arg_indent = indent.block_indent(context.config);
        result.push('\n');
        result.push_str(&arg_indent.to_string(context.config));
        result.push_str(&arg_str);
        result.push('\n');
        result.push_str(&indent.to_string(context.config));
        result.push(')');
    } else {
        result.push_str(&arg_str);
        if context.config.spaces_within_parens && fd.inputs.len() > 0 {
            result.push(' ')
        }
        result.push(')');
    }

    // Return type.
    if !ret_str.is_empty() {
        let ret_should_indent = match context.config.fn_args_layout {
            // If our args are block layout then we surely must have space.
            FnArgLayoutStyle::Block if put_args_in_block => false,
            FnArgLayoutStyle::BlockAlways => false,
            _ => {
                // If we've already gone multi-line, or the return type would push over the max
                // width, then put the return type on a new line. With the +1 for the signature
                // length an additional space between the closing parenthesis of the argument and
                // the arrow '->' is considered.
                let mut sig_length = result.len() + indent.width() + ret_str_len + 1;

                // If there is no where clause, take into account the space after the return type
                // and the brace.
                if where_clause.predicates.is_empty() {
                    sig_length += 2;
                }

                let overlong_sig = sig_length > context.config.max_width;

                result.contains('\n') || multi_line_ret_str || overlong_sig
            }
        };
        let ret_indent = if ret_should_indent {
            let indent = match context.config.fn_return_indent {
                ReturnIndent::WithWhereClause => indent + 4,
                // Aligning with non-existent args looks silly.
                _ if arg_str.is_empty() => {
                    force_new_line_for_brace = true;
                    indent + 4
                }
                // FIXME: we might want to check that using the arg indent
                // doesn't blow our budget, and if it does, then fallback to
                // the where clause indent.
                _ => arg_indent,
            };

            result.push('\n');
            result.push_str(&indent.to_string(context.config));
            indent
        } else {
            result.push(' ');
            Indent::new(indent.width(), result.len())
        };

        if multi_line_ret_str {
            // Now that we know the proper indent and width, we need to
            // re-layout the return type.
            let budget = try_opt!(context.config.max_width.checked_sub(ret_indent.width()));
            let ret_str = try_opt!(fd.output.rewrite(context, budget, ret_indent));
            result.push_str(&ret_str);
        } else {
            result.push_str(&ret_str);
        }

        // Comment between return type and the end of the decl.
        let snippet_lo = fd.output.span().hi;
        if where_clause.predicates.is_empty() {
            let snippet_hi = span.hi;
            let snippet = context.snippet(mk_sp(snippet_lo, snippet_hi));
            let snippet = snippet.trim();
            if !snippet.is_empty() {
                result.push(' ');
                result.push_str(snippet);
            }
        } else {
            // FIXME it would be nice to catch comments between the return type
            // and the where clause, but we don't have a span for the where
            // clause.
        }
    }

    let should_compress_where = match context.config.where_density {
        Density::Compressed => !result.contains('\n') || put_args_in_block,
        Density::CompressedIfEmpty => !has_body && !result.contains('\n'),
        _ => false,
    } || (put_args_in_block && ret_str.is_empty());

    let where_density = if should_compress_where {
        Density::Compressed
    } else {
        Density::Tall
    };

    // Where clause.
    let where_budget = try_opt!(context.config.max_width.checked_sub(last_line_width(&result)));
    let where_clause_str = try_opt!(rewrite_where_clause(context,
                                                         where_clause,
                                                         context.config,
                                                         context.config.fn_brace_style,
                                                         indent,
                                                         where_budget,
                                                         where_density,
                                                         "{",
                                                         has_body,
                                                         Some(span.hi)));

    if last_line_width(&result) + where_clause_str.len() > context.config.max_width &&
       !where_clause_str.contains('\n') {
        result.push('\n');
    }

    result.push_str(&where_clause_str);

    Some((result, force_new_line_for_brace))
}

fn rewrite_args(context: &RewriteContext,
                args: &[ast::Arg],
                explicit_self: Option<&ast::ExplicitSelf>,
                one_line_budget: usize,
                multi_line_budget: usize,
                indent: Indent,
                arg_indent: Indent,
                span: Span,
                variadic: bool)
                -> Option<String> {
    let mut arg_item_strs = try_opt!(args.iter()
        .map(|arg| arg.rewrite(&context, multi_line_budget, arg_indent))
        .collect::<Option<Vec<_>>>());

    // Account for sugary self.
    // FIXME: the comment for the self argument is dropped. This is blocked
    // on rust issue #27522.
    let min_args =
        explicit_self.and_then(|explicit_self| rewrite_explicit_self(explicit_self, args, context))
            .map_or(1, |self_str| {
                arg_item_strs[0] = self_str;
                2
            });

    // Comments between args.
    let mut arg_items = Vec::new();
    if min_args == 2 {
        arg_items.push(ListItem::from_str(""));
    }

    // FIXME(#21): if there are no args, there might still be a comment, but
    // without spans for the comment or parens, there is no chance of
    // getting it right. You also don't get to put a comment on self, unless
    // it is explicit.
    if args.len() >= min_args || variadic {
        let comment_span_start = if min_args == 2 {
            let second_arg_start = if arg_has_pattern(&args[1]) {
                args[1].pat.span.lo
            } else {
                args[1].ty.span.lo
            };
            let reduced_span = mk_sp(span.lo, second_arg_start);

            context.codemap.span_after_last(reduced_span, ",")
        } else {
            span.lo
        };

        enum ArgumentKind<'a> {
            Regular(&'a ast::Arg),
            Variadic(BytePos),
        }

        let variadic_arg = if variadic {
            let variadic_span = mk_sp(args.last().unwrap().ty.span.hi, span.hi);
            let variadic_start = context.codemap.span_after(variadic_span, "...") - BytePos(3);
            Some(ArgumentKind::Variadic(variadic_start))
        } else {
            None
        };

        let more_items = itemize_list(context.codemap,
                                      args[min_args - 1..]
                                          .iter()
                                          .map(ArgumentKind::Regular)
                                          .chain(variadic_arg),
                                      ")",
                                      |arg| match *arg {
                                          ArgumentKind::Regular(arg) => span_lo_for_arg(arg),
                                          ArgumentKind::Variadic(start) => start,
                                      },
                                      |arg| match *arg {
                                          ArgumentKind::Regular(arg) => arg.ty.span.hi,
                                          ArgumentKind::Variadic(start) => start + BytePos(3),
                                      },
                                      |arg| match *arg {
                                          ArgumentKind::Regular(..) => None,
                                          ArgumentKind::Variadic(..) => Some("...".to_owned()),
                                      },
                                      comment_span_start,
                                      span.hi);

        arg_items.extend(more_items);
    }

    for (item, arg) in arg_items.iter_mut().zip(arg_item_strs) {
        item.item = Some(arg);
    }

    let indent = match context.config.fn_arg_indent {
        BlockIndentStyle::Inherit => indent,
        BlockIndentStyle::Tabbed => indent.block_indent(context.config),
        BlockIndentStyle::Visual => arg_indent,
    };

    let tactic = definitive_tactic(&arg_items,
                                   context.config.fn_args_density.to_list_tactic(),
                                   one_line_budget);
    let budget = match tactic {
        DefinitiveListTactic::Horizontal => one_line_budget,
        _ => multi_line_budget,
    };

    debug!("rewrite_args: budget: {}, tactic: {:?}", budget, tactic);

    let end_with_newline = match context.config.fn_args_layout {
        FnArgLayoutStyle::Block |
        FnArgLayoutStyle::BlockAlways => true,
        _ => false,
    };

    let fmt = ListFormatting {
        tactic: tactic,
        separator: ",",
        trailing_separator: SeparatorTactic::Never,
        indent: indent,
        width: budget,
        ends_with_newline: end_with_newline,
        config: context.config,
    };

    write_list(&arg_items, &fmt)
}

fn arg_has_pattern(arg: &ast::Arg) -> bool {
    if let ast::PatKind::Ident(_,
                               codemap::Spanned {
                                   node: ast::Ident { name: ast::Name(0u32), .. },
                                   ..
                               },
                               _) = arg.pat.node {
        false
    } else {
        true
    }
}

fn compute_budgets_for_args(context: &RewriteContext,
                            result: &str,
                            indent: Indent,
                            ret_str_len: usize,
                            newline_brace: bool)
                            -> Option<((usize, usize, Indent))> {
    // Try keeping everything on the same line.
    if !result.contains('\n') {
        // 3 = `() `, space is before ret_string.
        let mut used_space = indent.width() + result.len() + ret_str_len + 3;
        if !newline_brace {
            used_space += 2;
        }
        let one_line_budget = context.config.max_width.checked_sub(used_space).unwrap_or(0);

        if one_line_budget > 0 {
            // 4 = "() {".len()
            let multi_line_budget =
                try_opt!(context.config.max_width.checked_sub(indent.width() + result.len() + 4));

            return Some((one_line_budget, multi_line_budget, indent + result.len() + 1));
        }
    }

    // Didn't work. we must force vertical layout and put args on a newline.
    let new_indent = indent.block_indent(context.config);
    let used_space = new_indent.width() + 4; // Account for `(` and `)` and possibly ` {`.
    let max_space = context.config.max_width;
    if used_space <= max_space {
        Some((0, max_space - used_space, new_indent))
    } else {
        // Whoops! bankrupt.
        None
    }
}

fn newline_for_brace(config: &Config, where_clause: &ast::WhereClause) -> bool {
    match config.fn_brace_style {
        BraceStyle::AlwaysNextLine => true,
        BraceStyle::SameLineWhere if !where_clause.predicates.is_empty() => true,
        _ => false,
    }
}

fn rewrite_generics(context: &RewriteContext,
                    generics: &ast::Generics,
                    offset: Indent,
                    width: usize,
                    generics_offset: Indent,
                    span: Span)
                    -> Option<String> {
    // FIXME: convert bounds to where clauses where they get too big or if
    // there is a where clause at all.
    let lifetimes: &[_] = &generics.lifetimes;
    let tys: &[_] = &generics.ty_params;
    if lifetimes.is_empty() && tys.is_empty() {
        return Some(String::new());
    }

    let offset = match context.config.generics_indent {
        BlockIndentStyle::Inherit => offset,
        BlockIndentStyle::Tabbed => offset.block_indent(context.config),
        // 1 = <
        BlockIndentStyle::Visual => generics_offset + 1,
    };

    let h_budget = try_opt!(width.checked_sub(generics_offset.width() + 2));
    // FIXME: might need to insert a newline if the generics are really long.

    // Strings for the generics.
    let lt_strs = lifetimes.iter().map(|lt| lt.rewrite(context, h_budget, offset));
    let ty_strs = tys.iter().map(|ty_param| ty_param.rewrite(context, h_budget, offset));

    // Extract comments between generics.
    let lt_spans = lifetimes.iter().map(|l| {
        let hi = if l.bounds.is_empty() {
            l.lifetime.span.hi
        } else {
            l.bounds[l.bounds.len() - 1].span.hi
        };
        mk_sp(l.lifetime.span.lo, hi)
    });
    let ty_spans = tys.iter().map(span_for_ty_param);

    let items = itemize_list(context.codemap,
                             lt_spans.chain(ty_spans).zip(lt_strs.chain(ty_strs)),
                             ">",
                             |&(sp, _)| sp.lo,
                             |&(sp, _)| sp.hi,
                             // FIXME: don't clone
                             |&(_, ref str)| str.clone(),
                             context.codemap.span_after(span, "<"),
                             span.hi);
    let list_str = try_opt!(format_item_list(items, h_budget, offset, context.config));

    Some(if context.config.spaces_within_angle_brackets {
        format!("< {} >", list_str)
    } else {
        format!("<{}>", list_str)
    })
}

fn rewrite_trait_bounds(context: &RewriteContext,
                        type_param_bounds: &ast::TyParamBounds,
                        indent: Indent,
                        width: usize)
                        -> Option<String> {
    let bounds: &[_] = type_param_bounds;

    if bounds.is_empty() {
        return Some(String::new());
    }

    let bound_str = try_opt!(bounds.iter()
        .map(|ty_bound| ty_bound.rewrite(&context, width, indent))
        .intersperse(Some(" + ".to_string()))
        .collect::<Option<String>>());

    let mut result = String::new();
    result.push_str(": ");
    result.push_str(&bound_str);
    Some(result)
}

fn rewrite_where_clause(context: &RewriteContext,
                        where_clause: &ast::WhereClause,
                        config: &Config,
                        brace_style: BraceStyle,
                        indent: Indent,
                        width: usize,
                        density: Density,
                        terminator: &str,
                        allow_trailing_comma: bool,
                        span_end: Option<BytePos>)
                        -> Option<String> {
    if where_clause.predicates.is_empty() {
        return Some(String::new());
    }

    let extra_indent = match context.config.where_indent {
        BlockIndentStyle::Inherit => Indent::empty(),
        BlockIndentStyle::Tabbed | BlockIndentStyle::Visual => Indent::new(config.tab_spaces, 0),
    };

    let offset = match context.config.where_pred_indent {
        BlockIndentStyle::Inherit => indent + extra_indent,
        BlockIndentStyle::Tabbed => indent + extra_indent.block_indent(config),
        // 6 = "where ".len()
        BlockIndentStyle::Visual => indent + extra_indent + 6,
    };
    // FIXME: if where_pred_indent != Visual, then the budgets below might
    // be out by a char or two.

    let budget = context.config.max_width - offset.width();
    let span_start = span_for_where_pred(&where_clause.predicates[0]).lo;
    // If we don't have the start of the next span, then use the end of the
    // predicates, but that means we miss comments.
    let len = where_clause.predicates.len();
    let end_of_preds = span_for_where_pred(&where_clause.predicates[len - 1]).hi;
    let span_end = span_end.unwrap_or(end_of_preds);
    let items = itemize_list(context.codemap,
                             where_clause.predicates.iter(),
                             terminator,
                             |pred| span_for_where_pred(pred).lo,
                             |pred| span_for_where_pred(pred).hi,
                             |pred| pred.rewrite(context, budget, offset),
                             span_start,
                             span_end);
    let item_vec = items.collect::<Vec<_>>();
    // FIXME: we don't need to collect here if the where_layout isn't
    // HorizontalVertical.
    let tactic = definitive_tactic(&item_vec, context.config.where_layout, budget);
    let use_trailing_comma = allow_trailing_comma && context.config.where_trailing_comma;

    let fmt = ListFormatting {
        tactic: tactic,
        separator: ",",
        trailing_separator: SeparatorTactic::from_bool(use_trailing_comma),
        indent: offset,
        width: budget,
        ends_with_newline: true,
        config: context.config,
    };
    let preds_str = try_opt!(write_list(&item_vec, &fmt));

    let end_length = if terminator == "{" {
        // If the brace is on the next line we don't need to count it otherwise it needs two
        // characters " {"
        match brace_style {
            BraceStyle::AlwaysNextLine |
            BraceStyle::SameLineWhere => 0,
            BraceStyle::PreferSameLine => 2,
        }
    } else if terminator == "=" {
        2
    } else {
        terminator.len()
    };
    if density == Density::Tall || preds_str.contains('\n') ||
       indent.width() + " where ".len() + preds_str.len() + end_length > width {
        Some(format!("\n{}where {}",
                     (indent + extra_indent).to_string(context.config),
                     preds_str))
    } else {
        Some(format!(" where {}", preds_str))
    }
}

fn format_header(item_name: &str, ident: ast::Ident, vis: &ast::Visibility) -> String {
    format!("{}{}{}", format_visibility(vis), item_name, ident)
}

fn format_generics(context: &RewriteContext,
                   generics: &ast::Generics,
                   opener: &str,
                   terminator: &str,
                   brace_style: BraceStyle,
                   force_same_line_brace: bool,
                   offset: Indent,
                   generics_offset: Indent,
                   span: Span)
                   -> Option<String> {
    let mut result = try_opt!(rewrite_generics(context,
                                               generics,
                                               offset,
                                               context.config.max_width,
                                               generics_offset,
                                               span));

    if !generics.where_clause.predicates.is_empty() || result.contains('\n') {
        let budget = try_opt!(context.config.max_width.checked_sub(last_line_width(&result)));
        let where_clause_str = try_opt!(rewrite_where_clause(context,
                                                             &generics.where_clause,
                                                             context.config,
                                                             brace_style,
                                                             context.block_indent,
                                                             budget,
                                                             Density::Tall,
                                                             terminator,
                                                             true,
                                                             Some(span.hi)));
        result.push_str(&where_clause_str);
        if !force_same_line_brace &&
           (brace_style == BraceStyle::SameLineWhere || brace_style == BraceStyle::AlwaysNextLine) {
            result.push('\n');
            result.push_str(&context.block_indent.to_string(context.config));
        } else {
            result.push(' ');
        }
        result.push_str(opener);
    } else {
        if !force_same_line_brace && brace_style == BraceStyle::AlwaysNextLine {
            result.push('\n');
            result.push_str(&context.block_indent.to_string(context.config));
        } else {
            result.push(' ');
        }
        result.push_str(opener);
    }

    Some(result)
}
