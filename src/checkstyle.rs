// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.

use std::io::{self, Write};
use std::path::Path;

use rustfmt_diff::{DiffLine, Mismatch};

pub fn output_header<T>(out: &mut T) -> Result<(), io::Error>
where
    T: Write,
{
    let mut xml_heading = String::new();
    xml_heading.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>");
    xml_heading.push_str("\n");
    xml_heading.push_str("<checkstyle version=\"4.3\">");
    write!(out, "{}", xml_heading)
}

pub fn output_footer<T>(out: &mut T) -> Result<(), io::Error>
where
    T: Write,
{
    let mut xml_tail = String::new();
    xml_tail.push_str("</checkstyle>\n");
    write!(out, "{}", xml_tail)
}

pub fn output_checkstyle_file<T>(
    mut writer: T,
    filename: &Path,
    diff: Vec<Mismatch>,
) -> Result<(), io::Error>
where
    T: Write,
{
    write!(writer, "<file name=\"{}\">", filename.display())?;
    for mismatch in diff {
        for line in mismatch.lines {
            // Do nothing with `DiffLine::Context` and `DiffLine::Resulting`.
            if let DiffLine::Expected(ref str) = line {
                let message = xml_escape_str(str);
                write!(
                    writer,
                    "<error line=\"{}\" severity=\"warning\" message=\"Should be `{}`\" \
                     />",
                    mismatch.line_number, message
                )?;
            }
        }
    }
    write!(writer, "</file>")?;
    Ok(())
}

// Convert special characters into XML entities.
// This is needed for checkstyle output.
fn xml_escape_str(string: &str) -> String {
    let mut out = String::new();
    for c in string.chars() {
        match c {
            '<' => out.push_str("&lt;"),
            '>' => out.push_str("&gt;"),
            '"' => out.push_str("&quot;"),
            '\'' => out.push_str("&apos;"),
            '&' => out.push_str("&amp;"),
            _ => out.push(c),
        }
    }
    out
}
