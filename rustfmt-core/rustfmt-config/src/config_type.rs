use crate::file_lines::FileLines;
use crate::options::{IgnoreList, WidthHeuristics};

/// Trait for types that can be used in `Config`.
pub(crate) trait ConfigType: Sized {
    /// Returns hint text for use in `Config::print_docs()`. For enum types, this is a
    /// pipe-separated list of variants; for other types it returns "<type>".
    fn doc_hint() -> String;
}

impl ConfigType for bool {
    fn doc_hint() -> String {
        String::from("<boolean>")
    }
}

impl ConfigType for usize {
    fn doc_hint() -> String {
        String::from("<unsigned integer>")
    }
}

impl ConfigType for isize {
    fn doc_hint() -> String {
        String::from("<signed integer>")
    }
}

impl ConfigType for String {
    fn doc_hint() -> String {
        String::from("<string>")
    }
}

impl ConfigType for FileLines {
    fn doc_hint() -> String {
        String::from("<json>")
    }
}

impl ConfigType for WidthHeuristics {
    fn doc_hint() -> String {
        String::new()
    }
}

impl ConfigType for IgnoreList {
    fn doc_hint() -> String {
        String::from("[<string>,..]")
    }
}

macro_rules! create_config {
    ($($i:ident: $Ty:ty, $def:expr, $is_stable:literal, $dstring:literal;)+) => (
        use std::collections::HashSet;
        use std::io::Write;

        use serde::{Deserialize, Serialize};

        #[derive(Clone)]
        #[allow(unreachable_pub)]
        pub struct Config {
            // if a license_template_path has been specified, successfully read, parsed and compiled
            // into a regex, it will be stored here
            pub license_template: Option<Regex>,
            // For each config item, we store a bool indicating whether it has
            // been accessed and the value, and a bool whether the option was
            // manually initialized, or taken from the default,
            $($i: (Cell<bool>, bool, $Ty, bool)),+
        }

        // Just like the Config struct but with each property wrapped
        // as Option<T>. This is used to parse a rustfmt.toml that doesn't
        // specify all properties of `Config`.
        // We first parse into `PartialConfig`, then create a default `Config`
        // and overwrite the properties with corresponding values from `PartialConfig`.
        #[derive(Deserialize, Serialize, Clone)]
        #[allow(unreachable_pub)]
        pub struct PartialConfig {
            $(pub $i: Option<$Ty>),+
        }

        // Macro hygiene won't allow us to make `set_$i()` methods on Config
        // for each item, so this struct is used to give the API to set values:
        // `config.set().option(false)`. It's pretty ugly. Consider replacing
        // with `config.set_option(false)` if we ever get a stable/usable
        // `concat_idents!()`.
        #[allow(unreachable_pub)]
        pub struct ConfigSetter<'a>(&'a mut Config);

        impl<'a> ConfigSetter<'a> {
            $(
            #[allow(unreachable_pub)]
            pub fn $i(&mut self, value: $Ty) {
                if value != (self.0).$i.2 {
                    (self.0).$i.1 = true;
                    (self.0).$i.2 = value;
                    match stringify!($i) {
                        "max_width" | "use_small_heuristics" => self.0.set_heuristics(),
                        "license_template_path" => self.0.set_license_template(),
                        &_ => (),
                    }
                }
            }
            )+
        }

        // Query each option, returns true if the user set the option, false if
        // a default was used.
        #[allow(unreachable_pub)]
        pub struct ConfigWasSet<'a>(&'a Config);

        impl<'a> ConfigWasSet<'a> {
            $(
            #[allow(unreachable_pub)]
            pub fn $i(&self) -> bool {
                (self.0).$i.1
            }
            )+
        }

        impl Config {
            $(
            #[allow(unreachable_pub)]
            pub fn $i(&self) -> $Ty {
                self.$i.0.set(true);
                self.$i.2.clone()
            }
            )+

            #[allow(unreachable_pub)]
            pub fn set(&mut self) -> ConfigSetter<'_> {
                ConfigSetter(self)
            }

            #[allow(unreachable_pub)]
            pub fn was_set(&self) -> ConfigWasSet<'_> {
                ConfigWasSet(self)
            }

            fn fill_from_parsed_config(mut self, parsed: PartialConfig, dir: &Path) -> Self {
            $(
                if let Some(val) = parsed.$i {
                    if self.$i.3 {
                        self.$i.1 = true;
                        self.$i.2 = val;
                    } else {
                        if is_nightly_channel!() {
                            self.$i.1 = true;
                            self.$i.2 = val;
                        } else {
                            eprintln!("Warning: can't set `{} = {:?}`, unstable features are only \
                                       available in nightly channel.", stringify!($i), val);
                        }
                    }
                }
            )+
                self.set_heuristics();
                self.set_license_template();
                self.set_ignore(dir);
                self
            }

            /// Returns a hash set initialized with every user-facing config option name.
            pub fn hash_set() -> HashSet<&'static str> {
                [$(
                    stringify!($i),
                )+]
                .iter().copied().collect()
            }

            pub fn is_valid_name(name: &str) -> bool {
                match name {
                    $(
                        stringify!($i) => true,
                    )+
                        _ => false,
                }
            }

            #[allow(unreachable_pub)]
            pub fn is_valid_key_val(key: &str, val: &str) -> bool {
                match key {
                    $(
                        stringify!($i) => val.parse::<$Ty>().is_ok(),
                    )+
                        _ => false,
                }
            }

            #[allow(unreachable_pub)]
            pub fn used_options(&self) -> PartialConfig {
                PartialConfig {
                    $(
                        $i: if self.$i.1 && self.$i.2 != $def {
                                Some(self.$i.2.clone())
                            } else {
                                None
                            },
                    )+
                }
            }

            #[allow(unreachable_pub)]
            pub fn all_options(&self) -> PartialConfig {
                PartialConfig {
                    $(
                        $i: Some(self.$i.2.clone()),
                    )+
                }
            }

            #[allow(unreachable_pub)]
            pub fn override_value(&mut self, key: &str, val: &str)
            {
                match key {
                    $(
                        stringify!($i) => {
                            self.$i.1 = true;
                            self.$i.2 = val.parse::<$Ty>()
                                .expect(&format!("Failed to parse override for {} (\"{}\") as a {}",
                                                 stringify!($i),
                                                 val,
                                                 stringify!($Ty)));
                        }
                    )+
                    _ => panic!("Unknown config key in override: {}", key)
                }

                match key {
                    "max_width" | "use_small_heuristics" => self.set_heuristics(),
                    "license_template_path" => self.set_license_template(),
                    &_ => (),
                }
            }

            #[allow(unreachable_pub)]
            pub fn is_hidden_option(name: &str) -> bool {
                const HIDE_OPTIONS: [&str; 6] = [
                    "verbose", "verbose_diff", "file_lines", "width_heuristics",
                    "recursive", "print_misformatted_file_names",
                ];
                HIDE_OPTIONS.contains(&name)
            }

            #[allow(unreachable_pub)]
            pub fn print_docs(out: &mut dyn Write, include_unstable: bool) {
                use std::cmp;
                let mut max = 0;
                $( max = cmp::max(max, stringify!($i).len()); )+
                max += 1;
                let space_str = " ".repeat(max);
                writeln!(out, "Configuration Options:").unwrap();
                $(
                    if $is_stable || include_unstable {
                        let name_raw = stringify!($i);

                        if !Config::is_hidden_option(name_raw) {
                            let mut name_out = String::with_capacity(max);
                            for _ in name_raw.len()..max-1 {
                                name_out.push(' ')
                            }
                            name_out.push_str(name_raw);
                            name_out.push(' ');
                            let mut default_str = $def.to_string();
                            if default_str.is_empty() {
                                default_str = String::from("\"\"");
                            }
                            writeln!(
                                out,
                                "{}{} Default: {}{}",
                                name_out,
                                <$Ty>::doc_hint(),
                                default_str,
                                if !$is_stable { " (unstable)" } else { "" },
                            ).unwrap();
                            writeln!(out, "{}{}\n", space_str, $dstring).unwrap();
                        }
                    }
                )+
            }

            fn set_heuristics(&mut self) {
                if self.use_small_heuristics.2 == Heuristics::Default {
                    let max_width = self.max_width.2;
                    self.set().width_heuristics(WidthHeuristics::scaled(max_width));
                } else if self.use_small_heuristics.2 == Heuristics::Max {
                    let max_width = self.max_width.2;
                    self.set().width_heuristics(WidthHeuristics::set(max_width));
                } else {
                    self.set().width_heuristics(WidthHeuristics::null());
                }
            }

            fn set_license_template(&mut self) {
                if self.was_set().license_template_path() {
                    let lt_path = self.license_template_path();
                    if lt_path.len() > 0 {
                        match license::load_and_compile_template(&lt_path) {
                            Ok(re) => self.license_template = Some(re),
                            Err(msg) => eprintln!("Warning for license template file {:?}: {}",
                                                lt_path, msg),
                        }
                    }
                }
            }

            fn set_ignore(&mut self, dir: &Path) {
                self.ignore.2.add_prefix(dir);
            }

            #[allow(unreachable_pub)]
            /// Returns `true` if the config key was explicitly set and is the default value.
            pub fn is_default(&self, key: &str) -> bool {
                match key {
                    $(
                        stringify!($i) => self.$i.1 && { let default = $def; self.$i.2 == default},
                    )+
                    _ => false,
                }
            }
        }

        // Template for the default configuration
        impl Default for Config {
            fn default() -> Self {
                Self {
                    license_template: None,
                    $(
                        $i: (Cell::new(false), false, $def, $is_stable),
                    )+
                }
            }
        }
    )
}

macro_rules! is_nightly_channel {
    () => {
        option_env!("CFG_RELEASE_CHANNEL").map_or(true, |c| c == "nightly" || c == "dev")
    };
}
