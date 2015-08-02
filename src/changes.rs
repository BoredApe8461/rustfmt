// Copyright 2015 The Rust Project Developers. See the COPYRIGHT
// file at the top-level directory of this distribution and at
// http://rust-lang.org/COPYRIGHT.
//
// Licensed under the Apache License, Version 2.0 <LICENSE-APACHE or
// http://www.apache.org/licenses/LICENSE-2.0> or the MIT license
// <LICENSE-MIT or http://opensource.org/licenses/MIT>, at your
// option. This file may not be copied, modified, or distributed
// except according to those terms.


// TODO
// print to files
// tests

use strings::string_buffer::StringBuffer;
use std::collections::HashMap;
use std::fmt;
use std::fs::File;
use std::io::{Write, stdout};
use WriteMode;
use NewlineStyle;
use config::Config;

// This is basically a wrapper around a bunch of Ropes which makes it convenient
// to work with libsyntax. It is badly named.
pub struct ChangeSet {
    pub file_map: HashMap<String, StringBuffer>,
}

impl ChangeSet {
    // Create a new ChangeSet for a given libsyntax CodeMap.
    pub fn new() -> ChangeSet {
        ChangeSet { file_map: HashMap::new() }
    }

    // Fetch a mutable reference to the output buffer for the given file name.
    // Panics on unknown files.
    pub fn get_mut(&mut self, file_name: &str) -> &mut StringBuffer {
        self.file_map.get_mut(file_name).unwrap()
    }

    // Return an iterator over the entire changed text.
    pub fn text<'c>(&'c self) -> FileIterator<'c> {
        FileIterator { change_set: self, keys: self.file_map.keys().collect(), cur_key: 0 }
    }

    // Append a newline to the end of each file.
    pub fn append_newlines(&mut self) {
        for (_, s) in self.file_map.iter_mut() {
            s.push_str("\n");
        }
    }

    pub fn write_all_files(&self,
                           mode: WriteMode,
                           config: &Config)
                           -> Result<(HashMap<String, String>), ::std::io::Error> {
        let mut result = HashMap::new();
        for filename in self.file_map.keys() {
            let one_result = try!(self.write_file(filename, mode, config));
            if let Some(r) = one_result {
                result.insert(filename.clone(), r);
            }
        }

        Ok(result)
    }

    pub fn write_file(&self,
                      filename: &str,
                      mode: WriteMode,
                      config: &Config)
                      -> Result<Option<String>, ::std::io::Error> {
        let text = &self.file_map[filename];

        // prints all newlines either as `\n` or as `\r\n`
        fn write_system_newlines<T>(mut writer: T,
                                    text: &StringBuffer,
                                    config: &Config)
                                    -> Result<(), ::std::io::Error>
            where T: Write
        {
            match config.newline_style {
                NewlineStyle::Unix => write!(writer, "{}", text),
                NewlineStyle::Windows => {
                    for (c, _) in text.chars() {
                        match c {
                            '\n' => try!(write!(writer, "\r\n")),
                            '\r' => continue,
                            c => try!(write!(writer, "{}", c)),
                        }
                    }
                    Ok(())
                },
            }
        }

        match mode {
            WriteMode::Overwrite => {
                // Do a little dance to make writing safer - write to a temp file
                // rename the original to a .bk, then rename the temp file to the
                // original.
                let tmp_name = filename.to_owned() + ".tmp";
                let bk_name = filename.to_owned() + ".bk";
                {
                    // Write text to temp file
                    let tmp_file = try!(File::create(&tmp_name));
                    try!(write_system_newlines(tmp_file, text, config));
                }

                try!(::std::fs::rename(filename, bk_name));
                try!(::std::fs::rename(tmp_name, filename));
            }
            WriteMode::NewFile(extn) => {
                let filename = filename.to_owned() + "." + extn;
                let file = try!(File::create(&filename));
                try!(write_system_newlines(file, text, config));
            }
            WriteMode::Display => {
                println!("{}:\n", filename);
                let stdout = stdout();
                let stdout_lock = stdout.lock();
                try!(write_system_newlines(stdout_lock, text, config));
            }
            WriteMode::Return(_) => {
                // io::Write is not implemented for String, working around with Vec<u8>
                let mut v = Vec::new();
                try!(write_system_newlines(&mut v, text, config));
                // won't panic, we are writing correct utf8
                return Ok(Some(String::from_utf8(v).unwrap()));
            }
        }

        Ok(None)
    }
}

// Iterates over each file in the ChangSet. Yields the filename and the changed
// text for that file.
pub struct FileIterator<'c> {
    change_set: &'c ChangeSet,
    keys: Vec<&'c String>,
    cur_key: usize,
}

impl<'c> Iterator for FileIterator<'c> {
    type Item = (&'c str, &'c StringBuffer);

    fn next(&mut self) -> Option<(&'c str, &'c StringBuffer)> {
        if self.cur_key >= self.keys.len() {
            return None;
        }

        let key = self.keys[self.cur_key];
        self.cur_key += 1;
        return Some((&key, &self.change_set.file_map[&*key]))
    }
}

impl fmt::Display for ChangeSet {
    // Prints the entire changed text.
    fn fmt(&self, fmt: &mut fmt::Formatter) -> Result<(), fmt::Error> {
        for (f, r) in self.text() {
            try!(write!(fmt, "{}:\n", f));
            try!(write!(fmt, "{}\n\n", r));
        }
        Ok(())
    }
}
