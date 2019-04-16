use ignore::{self, gitignore};
use std::path::PathBuf;

use crate::config::{FileName, IgnoreList};

pub struct IgnorePathSet {
    ignore_set: gitignore::Gitignore,
}

impl IgnorePathSet {
    pub fn from_ignore_list(ignore_list: &IgnoreList) -> Result<Self, ignore::Error> {
        let mut ignore_builder = gitignore::GitignoreBuilder::new(PathBuf::from(""));

        for ignore_path in ignore_list {
            ignore_builder.add_line(None, ignore_path.to_str().unwrap())?;
        }

        Ok(IgnorePathSet {
            ignore_set: ignore_builder.build()?,
        })
    }

    pub fn is_match(&self, file_name: &FileName) -> bool {
        match file_name {
            FileName::Stdin => false,
            FileName::Real(p) => self
                .ignore_set
                .matched_path_or_any_parents(p, false)
                .is_ignore(),
        }
    }
}

#[cfg(test)]
mod test {
    use crate::config::{Config, FileName};
    use crate::ignore_path::IgnorePathSet;
    use std::path::PathBuf;

    #[test]
    fn test_ignore_path_set() {
        match option_env!("CFG_RELEASE_CHANNEL") {
            // this test requires nightly
            None | Some("nightly") => {
                let config = Config::from_toml(r#"ignore = ["foo.rs", "bar_dir/*"]"#).unwrap();
                let ignore_path_set = IgnorePathSet::from_ignore_list(&config.ignore()).unwrap();

                assert!(ignore_path_set.is_match(&FileName::Real(PathBuf::from("src/foo.rs"))));
                assert!(ignore_path_set.is_match(&FileName::Real(PathBuf::from("bar_dir/baz.rs"))));
                assert!(!ignore_path_set.is_match(&FileName::Real(PathBuf::from("src/bar.rs"))));
            }
            _ => (),
        };
    }
}
