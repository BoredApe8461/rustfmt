use std::borrow::Cow;
use std::panic::{catch_unwind, AssertUnwindSafe};
use std::path::{Path, PathBuf};

use syntax::ast;
use syntax::errors::Diagnostic;
use syntax::parse::parser::Parser as RawParser;
use syntax::parse::token::{DelimToken, TokenKind};
use syntax::parse::{new_sub_parser_from_file, PResult};
use syntax::source_map::{Span, DUMMY_SP};
use syntax::symbol::kw;

use crate::syntux::session::ParseSess;
use crate::{Config, Input};

pub(crate) type DirectoryOwnership = syntax::parse::DirectoryOwnership;
pub(crate) type ModulePathSuccess = syntax::parse::parser::ModulePathSuccess;

#[derive(Clone)]
pub(crate) struct Directory {
    pub(crate) path: PathBuf,
    pub(crate) ownership: DirectoryOwnership,
}

impl<'a> Directory {
    fn to_syntax_directory(&'a self) -> syntax::parse::Directory<'a> {
        syntax::parse::Directory {
            path: Cow::Borrowed(&self.path),
            ownership: self.ownership.clone(),
        }
    }
}

/// A parser for Rust source code.
pub(crate) struct Parser<'a> {
    parser: RawParser<'a>,
    sess: &'a ParseSess,
}

/// A builder for the `Parser`.
#[derive(Default)]
pub(crate) struct ParserBuilder<'a> {
    config: Option<&'a Config>,
    sess: Option<&'a ParseSess>,
    input: Option<Input>,
    directory_ownership: Option<DirectoryOwnership>,
}

impl<'a> ParserBuilder<'a> {
    pub(crate) fn input(mut self, input: Input) -> ParserBuilder<'a> {
        self.input = Some(input);
        self
    }

    pub(crate) fn sess(mut self, sess: &'a ParseSess) -> ParserBuilder<'a> {
        self.sess = Some(sess);
        self
    }

    pub(crate) fn config(mut self, config: &'a Config) -> ParserBuilder<'a> {
        self.config = Some(config);
        self
    }

    pub(crate) fn directory_ownership(
        mut self,
        directory_ownership: Option<DirectoryOwnership>,
    ) -> ParserBuilder<'a> {
        self.directory_ownership = directory_ownership;
        self
    }

    pub(crate) fn build(self) -> Result<Parser<'a>, ParserError> {
        let config = self.config.ok_or(ParserError::NoConfig)?;
        let sess = self.sess.ok_or(ParserError::NoParseSess)?;
        let input = self.input.ok_or(ParserError::NoInput)?;

        let mut parser = match Self::parser(sess.inner(), input, self.directory_ownership) {
            Ok(p) => p,
            Err(db) => {
                sess.emit_diagnostics(db);
                return Err(ParserError::ParserCreationError);
            }
        };

        parser.cfg_mods = false;

        if config.skip_children() {
            parser.recurse_into_file_modules = false;
        }

        Ok(Parser { parser, sess })
    }

    fn parser(
        sess: &'a syntax::parse::ParseSess,
        input: Input,
        directory_ownership: Option<DirectoryOwnership>,
    ) -> Result<syntax::parse::parser::Parser<'a>, Vec<Diagnostic>> {
        match input {
            Input::File(ref file) => Ok(if let Some(directory_ownership) = directory_ownership {
                syntax::parse::new_sub_parser_from_file(
                    sess,
                    file,
                    directory_ownership,
                    None,
                    DUMMY_SP,
                )
            } else {
                syntax::parse::new_parser_from_file(sess, file)
            }),
            Input::Text(text) => syntax::parse::maybe_new_parser_from_source_str(
                sess,
                syntax::source_map::FileName::Custom("stdin".to_owned()),
                text,
            )
            .map(|mut parser| {
                parser.recurse_into_file_modules = false;
                parser
            }),
        }
    }
}

#[derive(Debug, PartialEq)]
pub(crate) enum ParserError {
    NoConfig,
    NoParseSess,
    NoInput,
    ParserCreationError,
    ParseError,
    ParsePanicError,
}

impl<'a> Parser<'a> {
    pub(crate) fn submod_path_from_attr(attrs: &[ast::Attribute], path: &Path) -> Option<PathBuf> {
        syntax::parse::parser::Parser::submod_path_from_attr(attrs, path)
    }

    // FIXME(topecongiro) Use the method from libsyntax[1] once it become public.
    //
    // [1] https://github.com/rust-lang/rust/blob/master/src/libsyntax/parse/attr.rs
    fn parse_inner_attrs(parser: &mut RawParser<'a>) -> PResult<'a, Vec<ast::Attribute>> {
        let mut attrs: Vec<ast::Attribute> = vec![];
        loop {
            match parser.token.kind {
                TokenKind::Pound => {
                    // Don't even try to parse if it's not an inner attribute.
                    if !parser.look_ahead(1, |t| t == &TokenKind::Not) {
                        break;
                    }

                    let attr = parser.parse_attribute(true)?;
                    assert_eq!(attr.style, ast::AttrStyle::Inner);
                    attrs.push(attr);
                }
                TokenKind::DocComment(s) => {
                    // we need to get the position of this token before we bump.
                    let attr = syntax::attr::mk_sugared_doc_attr(s, parser.token.span);
                    if attr.style == ast::AttrStyle::Inner {
                        attrs.push(attr);
                        parser.bump();
                    } else {
                        break;
                    }
                }
                _ => break,
            }
        }
        Ok(attrs)
    }

    fn parse_mod_items(parser: &mut RawParser<'a>, span: Span) -> PResult<'a, ast::Mod> {
        let mut items = vec![];
        while let Some(item) = parser.parse_item()? {
            items.push(item);
        }

        let hi = if parser.token.span.is_dummy() {
            span
        } else {
            parser.prev_span
        };

        Ok(ast::Mod {
            inner: span.to(hi),
            items,
            inline: false,
        })
    }

    pub(crate) fn parse_file_as_module(
        directory_ownership: DirectoryOwnership,
        sess: &'a ParseSess,
        path: &Path,
    ) -> Option<ast::Mod> {
        let result = catch_unwind(AssertUnwindSafe(|| {
            let mut parser =
                new_sub_parser_from_file(sess.inner(), &path, directory_ownership, None, DUMMY_SP);

            parser.cfg_mods = false;
            let lo = parser.token.span;
            // FIXME(topecongiro) Format inner attributes (#3606).
            match Parser::parse_inner_attrs(&mut parser) {
                Ok(_attrs) => (),
                Err(mut e) => {
                    e.cancel();
                    sess.reset_errors();
                    return None;
                }
            }

            match Parser::parse_mod_items(&mut parser, lo) {
                Ok(m) => Some(m.clone()),
                Err(mut db) => {
                    db.cancel();
                    sess.reset_errors();
                    None
                }
            }
        }));
        match result {
            Ok(Some(m)) => Some(m),
            _ => None,
        }
    }

    pub(crate) fn parse_crate(
        config: &'a Config,
        input: Input,
        directory_ownership: Option<DirectoryOwnership>,
        sess: &'a ParseSess,
    ) -> Result<ast::Crate, ParserError> {
        let mut parser = ParserBuilder::default()
            .config(config)
            .input(input)
            .directory_ownership(directory_ownership)
            .sess(sess)
            .build()?;

        parser.parse_crate_inner()
    }

    fn parse_crate_inner(&mut self) -> Result<ast::Crate, ParserError> {
        let mut parser = AssertUnwindSafe(&mut self.parser);

        match catch_unwind(move || parser.parse_crate_mod()) {
            Ok(Ok(krate)) => {
                if !self.sess.has_errors() {
                    return Ok(krate);
                }

                if self.sess.can_reset_errors() {
                    self.sess.reset_errors();
                    return Ok(krate);
                }

                Err(ParserError::ParseError)
            }
            Ok(Err(mut db)) => {
                db.emit();
                Err(ParserError::ParseError)
            }
            Err(_) => Err(ParserError::ParsePanicError),
        }
    }

    pub(crate) fn parse_cfg_if(
        sess: &'a ParseSess,
        mac: &'a ast::Mac,
        base_dir: &Directory,
    ) -> Result<Vec<ast::Item>, &'static str> {
        match catch_unwind(AssertUnwindSafe(|| {
            Parser::parse_cfg_if_inner(sess, mac, base_dir)
        })) {
            Ok(Ok(items)) => Ok(items),
            Ok(err @ Err(_)) => err,
            Err(..) => Err("failed to parse cfg_if!"),
        }
    }

    fn parse_cfg_if_inner(
        sess: &'a ParseSess,
        mac: &'a ast::Mac,
        base_dir: &Directory,
    ) -> Result<Vec<ast::Item>, &'static str> {
        let mut parser = syntax::parse::stream_to_parser_with_base_dir(
            sess.inner(),
            mac.tts.clone(),
            base_dir.to_syntax_directory(),
        );

        parser.cfg_mods = false;
        let mut items = vec![];
        let mut process_if_cfg = true;

        while parser.token.kind != TokenKind::Eof {
            if process_if_cfg {
                if !parser.eat_keyword(kw::If) {
                    return Err("Expected `if`");
                }
                parser
                    .parse_attribute(false)
                    .map_err(|_| "Failed to parse attributes")?;
            }

            if !parser.eat(&TokenKind::OpenDelim(DelimToken::Brace)) {
                return Err("Expected an opening brace");
            }

            while parser.token != TokenKind::CloseDelim(DelimToken::Brace)
                && parser.token.kind != TokenKind::Eof
            {
                let item = match parser.parse_item() {
                    Ok(Some(item_ptr)) => item_ptr.into_inner(),
                    Ok(None) => continue,
                    Err(mut err) => {
                        err.cancel();
                        parser.sess.span_diagnostic.reset_err_count();
                        return Err(
                            "Expected item inside cfg_if block, but failed to parse it as an item",
                        );
                    }
                };
                if let ast::ItemKind::Mod(..) = item.kind {
                    items.push(item);
                }
            }

            if !parser.eat(&TokenKind::CloseDelim(DelimToken::Brace)) {
                return Err("Expected a closing brace");
            }

            if parser.eat(&TokenKind::Eof) {
                break;
            }

            if !parser.eat_keyword(kw::Else) {
                return Err("Expected `else`");
            }

            process_if_cfg = parser.token.is_keyword(kw::If);
        }

        Ok(items)
    }
}
