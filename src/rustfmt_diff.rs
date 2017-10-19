// Copyright 2017 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use diff;
use std::collections::VecDeque;
use std::io;
use term;
use utils::isatty;

#[derive(Debug, PartialEq)]
pub enum DiffLine {
    Context(String),
    Expected(String),
    Resulting(String),
}

#[derive(Debug, PartialEq)]
pub struct Mismatch {
    pub line_number: u32,
    pub lines: Vec<DiffLine>,
}

impl Mismatch {
    fn new(line_number: u32) -> Mismatch {
        Mismatch {
            line_number: line_number,
            lines: Vec::new(),
        }
    }
}

// Produces a diff between the expected output and actual output of rustfmt.
pub fn make_diff(expected: &str, actual: &str, context_size: usize) -> Vec<Mismatch> {
    let mut line_number = 1;
    let mut context_queue: VecDeque<&str> = VecDeque::with_capacity(context_size);
    let mut lines_since_mismatch = context_size + 1;
    let mut results = Vec::new();
    let mut mismatch = Mismatch::new(0);

    for result in diff::lines(expected, actual) {
        match result {
            diff::Result::Left(str) => {
                if lines_since_mismatch >= context_size && lines_since_mismatch > 0 {
                    results.push(mismatch);
                    mismatch = Mismatch::new(line_number - context_queue.len() as u32);
                }

                while let Some(line) = context_queue.pop_front() {
                    mismatch.lines.push(DiffLine::Context(line.to_owned()));
                }

                mismatch.lines.push(DiffLine::Resulting(str.to_owned()));
                lines_since_mismatch = 0;
            }
            diff::Result::Right(str) => {
                if lines_since_mismatch >= context_size && lines_since_mismatch > 0 {
                    results.push(mismatch);
                    mismatch = Mismatch::new(line_number - context_queue.len() as u32);
                }

                while let Some(line) = context_queue.pop_front() {
                    mismatch.lines.push(DiffLine::Context(line.to_owned()));
                }

                mismatch.lines.push(DiffLine::Expected(str.to_owned()));
                line_number += 1;
                lines_since_mismatch = 0;
            }
            diff::Result::Both(str, _) => {
                if context_queue.len() >= context_size {
                    let _ = context_queue.pop_front();
                }

                if lines_since_mismatch < context_size {
                    mismatch.lines.push(DiffLine::Context(str.to_owned()));
                } else if context_size > 0 {
                    context_queue.push_back(str);
                }

                line_number += 1;
                lines_since_mismatch += 1;
            }
        }
    }

    results.push(mismatch);
    results.remove(0);

    results
}

pub fn print_diff<F>(diff: Vec<Mismatch>, get_section_title: F)
where
    F: Fn(u32) -> String,
{
    match term::stdout() {
        Some(ref t) if isatty() && t.supports_color() => {
            print_diff_fancy(diff, get_section_title, term::stdout().unwrap())
        }
        _ => print_diff_basic(diff, get_section_title),
    }
}

fn print_diff_fancy<F>(
    diff: Vec<Mismatch>,
    get_section_title: F,
    mut t: Box<term::Terminal<Output = io::Stdout>>,
) where
    F: Fn(u32) -> String,
{
    for mismatch in diff {
        let title = get_section_title(mismatch.line_number);
        writeln!(t, "{}", title).unwrap();

        for line in mismatch.lines {
            match line {
                DiffLine::Context(ref str) => {
                    t.reset().unwrap();
                    writeln!(t, " {}⏎", str).unwrap();
                }
                DiffLine::Expected(ref str) => {
                    t.fg(term::color::GREEN).unwrap();
                    writeln!(t, "+{}⏎", str).unwrap();
                }
                DiffLine::Resulting(ref str) => {
                    t.fg(term::color::RED).unwrap();
                    writeln!(t, "-{}⏎", str).unwrap();
                }
            }
        }
        t.reset().unwrap();
    }
}

pub fn print_diff_basic<F>(diff: Vec<Mismatch>, get_section_title: F)
where
    F: Fn(u32) -> String,
{
    for mismatch in diff {
        let title = get_section_title(mismatch.line_number);
        println!("{}", title);

        for line in mismatch.lines {
            match line {
                DiffLine::Context(ref str) => {
                    println!(" {}⏎", str);
                }
                DiffLine::Expected(ref str) => {
                    println!("+{}⏎", str);
                }
                DiffLine::Resulting(ref str) => {
                    println!("-{}⏎", str);
                }
            }
        }
    }
}

#[cfg(test)]
mod test {
    use super::{make_diff, Mismatch};
    use super::DiffLine::*;

    #[test]
    fn diff_simple() {
        let src = "one\ntwo\nthree\nfour\nfive\n";
        let dest = "one\ntwo\ntrois\nfour\nfive\n";
        let diff = make_diff(src, dest, 1);
        assert_eq!(
            diff,
            vec![
                Mismatch {
                    line_number: 2,
                    lines: vec![
                        Context("two".into()),
                        Resulting("three".into()),
                        Expected("trois".into()),
                        Context("four".into()),
                    ],
                },
            ]
        );
    }

    #[test]
    fn diff_simple2() {
        let src = "one\ntwo\nthree\nfour\nfive\nsix\nseven\n";
        let dest = "one\ntwo\ntrois\nfour\ncinq\nsix\nseven\n";
        let diff = make_diff(src, dest, 1);
        assert_eq!(
            diff,
            vec![
                Mismatch {
                    line_number: 2,
                    lines: vec![
                        Context("two".into()),
                        Resulting("three".into()),
                        Expected("trois".into()),
                        Context("four".into()),
                    ],
                },
                Mismatch {
                    line_number: 5,
                    lines: vec![
                        Resulting("five".into()),
                        Expected("cinq".into()),
                        Context("six".into()),
                    ],
                },
            ]
        );
    }

    #[test]
    fn diff_zerocontext() {
        let src = "one\ntwo\nthree\nfour\nfive\n";
        let dest = "one\ntwo\ntrois\nfour\nfive\n";
        let diff = make_diff(src, dest, 0);
        assert_eq!(
            diff,
            vec![
                Mismatch {
                    line_number: 3,
                    lines: vec![Resulting("three".into()), Expected("trois".into())],
                },
            ]
        );
    }
}
