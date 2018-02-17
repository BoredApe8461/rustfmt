// Copyright 2018 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

//! Reorder items.
//!
//! `mod`, `extern crate` and `use` declarations are reorderd in alphabetical
//! order. Trait items are reordered in pre-determined order (associated types
//! and constatns comes before methods).

use config::lists::*;
use syntax::{ast, codemap::Span};

use comment::combine_strs_with_missing_comments;
use imports::{path_to_imported_ident, rewrite_import, rewrite_mod};
use lists::{itemize_list, write_list, ListFormatting};
use rewrite::{Rewrite, RewriteContext};
use shape::Shape;
use spanned::Spanned;
use utils::mk_sp;
use visitor::{filter_inline_attrs, rewrite_extern_crate};

use std::cmp::Ordering;

fn compare_path_segments(a: &ast::PathSegment, b: &ast::PathSegment) -> Ordering {
    a.identifier.name.as_str().cmp(&b.identifier.name.as_str())
}

fn compare_paths(a: &ast::Path, b: &ast::Path) -> Ordering {
    for segment in a.segments.iter().zip(b.segments.iter()) {
        let ord = compare_path_segments(segment.0, segment.1);
        if ord != Ordering::Equal {
            return ord;
        }
    }
    a.segments.len().cmp(&b.segments.len())
}

fn compare_use_trees(a: &ast::UseTree, b: &ast::UseTree, nested: bool) -> Ordering {
    use ast::UseTreeKind::*;

    // `use_nested_groups` is not yet supported, remove the `if !nested` when support will be
    // fully added
    if !nested {
        let paths_cmp = compare_paths(&a.prefix, &b.prefix);
        if paths_cmp != Ordering::Equal {
            return paths_cmp;
        }
    }

    match (&a.kind, &b.kind) {
        (&Simple(ident_a), &Simple(ident_b)) => {
            let name_a = &*path_to_imported_ident(&a.prefix).name.as_str();
            let name_b = &*path_to_imported_ident(&b.prefix).name.as_str();
            let name_ordering = if name_a == "self" {
                if name_b == "self" {
                    Ordering::Equal
                } else {
                    Ordering::Less
                }
            } else if name_b == "self" {
                Ordering::Greater
            } else {
                name_a.cmp(name_b)
            };
            if name_ordering == Ordering::Equal {
                if ident_a.name.as_str() != name_a {
                    if ident_b.name.as_str() != name_b {
                        ident_a.name.as_str().cmp(&ident_b.name.as_str())
                    } else {
                        Ordering::Greater
                    }
                } else {
                    Ordering::Less
                }
            } else {
                name_ordering
            }
        }
        (&Glob, &Glob) => Ordering::Equal,
        (&Simple(_), _) | (&Glob, &Nested(_)) => Ordering::Less,
        (&Nested(ref a_items), &Nested(ref b_items)) => {
            let mut a = a_items
                .iter()
                .map(|&(ref tree, _)| tree.clone())
                .collect::<Vec<_>>();
            let mut b = b_items
                .iter()
                .map(|&(ref tree, _)| tree.clone())
                .collect::<Vec<_>>();
            a.sort_by(|a, b| compare_use_trees(a, b, true));
            b.sort_by(|a, b| compare_use_trees(a, b, true));
            for comparison_pair in a.iter().zip(b.iter()) {
                let ord = compare_use_trees(comparison_pair.0, comparison_pair.1, true);
                if ord != Ordering::Equal {
                    return ord;
                }
            }
            a.len().cmp(&b.len())
        }
        (&Glob, &Simple(_)) | (&Nested(_), _) => Ordering::Greater,
    }
}

/// Choose the ordering between the given two items.
fn compare_items(a: &ast::Item, b: &ast::Item) -> Ordering {
    match (&a.node, &b.node) {
        (&ast::ItemKind::Mod(..), &ast::ItemKind::Mod(..)) => {
            a.ident.name.as_str().cmp(&b.ident.name.as_str())
        }
        (&ast::ItemKind::Use(ref a_tree), &ast::ItemKind::Use(ref b_tree)) => {
            compare_use_trees(a_tree, b_tree, false)
        }
        (&ast::ItemKind::ExternCrate(ref a_name), &ast::ItemKind::ExternCrate(ref b_name)) => {
            // `extern crate foo as bar;`
            //               ^^^ Comparing this.
            let a_orig_name =
                a_name.map_or_else(|| a.ident.name.as_str(), |symbol| symbol.as_str());
            let b_orig_name =
                b_name.map_or_else(|| b.ident.name.as_str(), |symbol| symbol.as_str());
            let result = a_orig_name.cmp(&b_orig_name);
            if result != Ordering::Equal {
                return result;
            }

            // `extern crate foo as bar;`
            //                      ^^^ Comparing this.
            match (a_name, b_name) {
                (Some(..), None) => Ordering::Greater,
                (None, Some(..)) => Ordering::Less,
                (None, None) => Ordering::Equal,
                (Some(..), Some(..)) => a.ident.name.as_str().cmp(&b.ident.name.as_str()),
            }
        }
        _ => unreachable!(),
    }
}

/// Rewrite a list of items with reordering. Every item in `items` must have
/// the same `ast::ItemKind`.
// TODO (some day) remove unused imports, expand globs, compress many single
// imports into a list import.
pub fn rewrite_reorderable_items(
    context: &RewriteContext,
    reorderable_items: &[&ast::Item],
    shape: Shape,
    span: Span,
) -> Option<String> {
    let items = itemize_list(
        context.codemap,
        reorderable_items.iter(),
        "",
        ";",
        |item| item.span().lo(),
        |item| item.span().hi(),
        |item| {
            let attrs = filter_inline_attrs(&item.attrs, item.span());
            let attrs_str = attrs.rewrite(context, shape)?;

            let missed_span = if attrs.is_empty() {
                mk_sp(item.span.lo(), item.span.lo())
            } else {
                mk_sp(attrs.last().unwrap().span.hi(), item.span.lo())
            };

            let item_str = match item.node {
                ast::ItemKind::Use(ref tree) => {
                    rewrite_import(context, &item.vis, tree, &item.attrs, shape)?
                }
                ast::ItemKind::ExternCrate(..) => rewrite_extern_crate(context, item)?,
                ast::ItemKind::Mod(..) => rewrite_mod(item),
                _ => return None,
            };

            combine_strs_with_missing_comments(
                context,
                &attrs_str,
                &item_str,
                missed_span,
                shape,
                false,
            )
        },
        span.lo(),
        span.hi(),
        false,
    );
    let mut item_pair_vec: Vec<_> = items.zip(reorderable_items.iter()).collect();
    item_pair_vec.sort_by(|a, b| compare_items(a.1, b.1));
    let item_vec: Vec<_> = item_pair_vec.into_iter().map(|pair| pair.0).collect();

    let fmt = ListFormatting {
        tactic: DefinitiveListTactic::Vertical,
        separator: "",
        trailing_separator: SeparatorTactic::Never,
        separator_place: SeparatorPlace::Back,
        shape,
        ends_with_newline: true,
        preserve_newline: false,
        config: context.config,
    };

    write_list(&item_vec, &fmt)
}
