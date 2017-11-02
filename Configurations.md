# Configuring Rustfmt

Rustfmt is designed to be very configurable. You can create a TOML file called `rustfmt.toml` or `.rustfmt.toml`, place it in the project or any other parent directory and it will apply the options in that file.

A possible content of `rustfmt.toml` or `.rustfmt.toml` might look like this:

```toml
array_layout = "Block"
array_width = 80
reorder_imported_names = true
```

# Configuration Options

Below you find a detailed visual guide on all the supported configuration options of rustfmt:

## `array_horizontal_layout_threshold`

How many elements array must have before rustfmt uses horizontal layout.  
Use this option to prevent a huge array from being vertically formatted.

- **Default value**: `0`
- **Possible values**: any positive integer

**Note:** A value of `0` results in [`array_layout`](#array_layout) being applied regardless of a line's width.

#### `0` (default):

```rust
// Each element will be placed on its own line.
let a = vec![
    0,
    1,
    2,
    3,
    4,
    ...
    999,
    1000,
];
```

#### `1000`:

```rust
// Each element will be placed on the same line as much as possible.
let a = vec![
    0, 1, 2, 3, 4, ...
    ..., 999, 1000,
];
```

## `array_layout`

Indent on arrays

- **Default value**: `"Block"`
- **Possible values**: `"Block"`, `"Visual"`

#### `"Block"` (default):

```rust
let lorem = vec![
    "ipsum",
    "dolor",
    "sit",
    "amet",
    "consectetur",
    "adipiscing",
    "elit",
];
```

#### `"Visual"`:

```rust
let lorem = vec!["ipsum",
                 "dolor",
                 "sit",
                 "amet",
                 "consectetur",
                 "adipiscing",
                 "elit"];
```

## `array_width`

Maximum width of an array literal before falling back to vertical formatting

- **Default value**: `60`
- **Possible values**: any positive integer

**Note:** A value of `0` results in [`array_layout`](#array_layout) being applied regardless of a line's width.

#### Lines shorter than `array_width`:
```rust
let lorem = vec!["ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing", "elit"];
```

#### Lines longer than `array_width`:
See [`array_layout`](#array_layout).

## `attributes_on_same_line_as_field`

Try to put attributes on the same line as fields

- **Default value**: `true`
- **Possible values**: `true`, `false`

#### `true` (default):

```rust
struct Lorem {
    #[serde(rename = "Ipsum")] ipsum: usize,
    #[serde(rename = "Dolor")] dolor: usize,
    #[serde(rename = "Amet")] amet: usize,
}
```

#### `false`:

```rust
struct Lorem {
    #[serde(rename = "Ipsum")]
    ipsum: usize,
    #[serde(rename = "Dolor")]
    dolor: usize,
    #[serde(rename = "Amet")]
    amet: usize,
}
```

## `attributes_on_same_line_as_variant`

Try to put attributes on the same line as variants

- **Default value**: `true`
- **Possible values**: `true`, `false`

#### `true` (default):

```rust
enum Lorem {
    #[serde(skip_serializing)] Ipsum,
    #[serde(skip_serializing)] Dolor,
    #[serde(skip_serializing)] Amet,
}
```

#### `false`:

```rust
enum Lorem {
    #[serde(skip_serializing)]
    Ipsum,
    #[serde(skip_serializing)]
    Dolor,
    #[serde(skip_serializing)]
    Amet,
}
```

## `binop_separator`

Where to put a binary operator when a binary expression goes multiline.

- **Default value**: `"Front"`
- **Possible values**: `"Front"`, `"Back"`

#### `"Front"` (default):

```rust
let or = foo
    || bar
    || foobar;

let sum = 1234
    + 5678
    + 910;

let range = aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
    ..bbbbbbbbbbbbbbbbbbbbbbbbbbbbb;
```

#### `"Back"`:

```rust
let or = foo ||
    bar ||
    foobar;

let sum = 1234 +
    5678 +
    910;

let range = aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa..
    bbbbbbbbbbbbbbbbbbbbbbbbbbbbb;
```

## `chain_indent`

Indentation of chain

- **Default value**: `"Block"`
- **Possible values**: `"Block"`, `"Visual"`

#### `"Block"` (default):

```rust
let lorem = ipsum
    .dolor()
    .sit()
    .amet()
    .consectetur()
    .adipiscing()
    .elit();
```

#### `"Visual"`:

```rust
let lorem = ipsum.dolor()
                 .sit()
                 .amet()
                 .consectetur()
                 .adipiscing()
                 .elit();
```

See also [`chain_one_line_max`](#chain_one_line_max).

## `chain_one_line_max`

Maximum length of a chain to fit on a single line

- **Default value**: `60`
- **Possible values**: any positive integer

#### Lines shorter than `chain_one_line_max`:
```rust
let lorem = ipsum.dolor().sit().amet().consectetur().adipiscing().elit();
```

#### Lines longer than `chain_one_line_max`:
See [`chain_indent`](#chain_indent).

## `chain_split_single_child`

Split a chain with a single child if its length exceeds [`chain_one_line_max`](#chain_one_line_max).

- **Default value**: `false`
- **Possible values**: `false`, `true`

#### `false` (default):

```rust
let files = fs::read_dir("tests/coverage/source").expect("Couldn't read source dir");
```

#### `true`:

```rust
let files = fs::read_dir("tests/coverage/source")
    .expect("Couldn't read source dir");
```

See also [`chain_one_line_max`](#chain_one_line_max).

## `closure_block_indent_threshold`

How many lines a closure must have before it is block indented. -1 means never use block indent.

- **Default value**: `7`
- **Possible values**: `-1`, or any positive integer

#### Closures shorter than `closure_block_indent_threshold`:
```rust
lorem_ipsum(|| {
                println!("lorem");
                println!("ipsum");
                println!("dolor");
                println!("sit");
                println!("amet");
            });
```

#### Closures longer than `closure_block_indent_threshold`:
```rust
lorem_ipsum(|| {
    println!("lorem");
    println!("ipsum");
    println!("dolor");
    println!("sit");
    println!("amet");
    println!("consectetur");
    println!("adipiscing");
    println!("elit");
});
```

**Note**: This option only takes effect when `fn_call_style` is set to `"Visual"`.

## `combine_control_expr`

Combine control expressions with function calls.

- **Default value**: `true`
- **Possible values**: `true`, `false`

#### `true` (default):

```rust
fn example() {
    // If
    foo!(if x {
        foo();
    } else {
        bar();
    });

    // IfLet
    foo!(if let Some(..) = x {
        foo();
    } else {
        bar();
    });

    // While
    foo!(while x {
        foo();
        bar();
    });

    // WhileLet
    foo!(while let Some(..) = x {
        foo();
        bar();
    });

    // ForLoop
    foo!(for x in y {
        foo();
        bar();
    });

    // Loop
    foo!(loop {
        foo();
        bar();
    });
}
```

#### `false`:

```rust
```

## `comment_width`

Maximum length of comments. No effect unless`wrap_comments = true`.

- **Default value**: `80`
- **Possible values**: any positive integer

**Note:** A value of `0` results in [`wrap_comments`](#wrap_comments) being applied regardless of a line's width.

#### Comments shorter than `comment_width`:
```rust
// Lorem ipsum dolor sit amet, consectetur adipiscing elit.
```

#### Comments longer than `comment_width`:
```rust
// Lorem ipsum dolor sit amet,
// consectetur adipiscing elit.
```

See also [`wrap_comments`](#wrap_comments).

## `condense_wildcard_suffixes`

Replace strings of _ wildcards by a single .. in tuple patterns

- **Default value**: `false`
- **Possible values**: `true`, `false`

#### `false` (default):

```rust
let (lorem, ipsum, _, _) = (1, 2, 3, 4);
```

#### `true`:

```rust
let (lorem, ipsum, ..) = (1, 2, 3, 4);
```

## `control_style`

Indent style for control flow statements

- **Default value**: `"Rfc"`
- **Possible values**: `"Rfc"`, `"Legacy"`

#### `"Rfc"` (default):

```rust
if lorem_ipsum &&
    dolor_sit &&
    amet_consectetur
{
    // ...
}
```

#### `"Legacy"`:

```rust
if lorem_ipsum &&
   dolor_sit &&
   amet_consectetur {
    // ...
}
```

See also: [`control_brace_style`](#control_brace_style).

## `control_brace_style`

Brace style for control flow constructs

- **Default value**: `"AlwaysSameLine"`
- **Possible values**: `"AlwaysNextLine"`, `"AlwaysSameLine"`, `"ClosingNextLine"`

#### `"AlwaysSameLine"` (default):

```rust
if lorem {
    println!("ipsum!");
} else {
    println!("dolor!");
}
```

#### `"AlwaysNextLine"`:

```rust
if lorem
{
    println!("ipsum!");
}
else
{
    println!("dolor!");
}
```

#### `"ClosingNextLine"`:

```rust
if lorem {
    println!("ipsum!");
}
else {
    println!("dolor!");
}
```

## `disable_all_formatting`

Don't reformat anything

- **Default value**: `false`
- **Possible values**: `true`, `false`

## `error_on_line_overflow`

Error if unable to get all lines within max_width

- **Default value**: `true`
- **Possible values**: `true`, `false`

See also [`max_width`](#max_width).

## `fn_args_density`

Argument density in functions

- **Default value**: `"Tall"`
- **Possible values**: `"Compressed"`, `"CompressedIfEmpty"`, `"Tall"`, `"Vertical"`

#### `"Tall"` (default):

```rust
trait Lorem {
    fn lorem(ipsum: Ipsum, dolor: Dolor, sit: Sit, amet: Amet);

    fn lorem(ipsum: Ipsum, dolor: Dolor, sit: Sit, amet: Amet) {
        // body
    }

    fn lorem(
        ipsum: Ipsum,
        dolor: Dolor,
        sit: Sit,
        amet: Amet,
        consectetur: Consectetur,
        adipiscing: Adipiscing,
        elit: Elit,
    );

    fn lorem(
        ipsum: Ipsum,
        dolor: Dolor,
        sit: Sit,
        amet: Amet,
        consectetur: Consectetur,
        adipiscing: Adipiscing,
        elit: Elit,
    ) {
        // body
    }
}
```

#### `"Compressed"`:

```rust
trait Lorem {
    fn lorem(ipsum: Ipsum, dolor: Dolor, sit: Sit, amet: Amet);

    fn lorem(ipsum: Ipsum, dolor: Dolor, sit: Sit, amet: Amet) {
        // body
    }

    fn lorem(
        ipsum: Ipsum, dolor: Dolor, sit: Sit, amet: Amet, consectetur: Consectetur,
        adipiscing: Adipiscing, elit: Elit,
    );

    fn lorem(
        ipsum: Ipsum, dolor: Dolor, sit: Sit, amet: Amet, consectetur: Consectetur,
        adipiscing: Adipiscing, elit: Elit,
    ) {
        // body
    }
}
```

#### `"CompressedIfEmpty"`:

```rust
trait Lorem {
    fn lorem(ipsum: Ipsum, dolor: Dolor, sit: Sit, amet: Amet);

    fn lorem(ipsum: Ipsum, dolor: Dolor, sit: Sit, amet: Amet) {
        // body
    }

    fn lorem(
        ipsum: Ipsum, dolor: Dolor, sit: Sit, amet: Amet, consectetur: Consectetur,
        adipiscing: Adipiscing, elit: Elit,
    );

    fn lorem(
        ipsum: Ipsum,
        dolor: Dolor,
        sit: Sit,
        amet: Amet,
        consectetur: Consectetur,
        adipiscing: Adipiscing,
        elit: Elit,
    ) {
        // body
    }
}
```

#### `"Vertical"`:

```rust
trait Lorem {
    fn lorem(ipsum: Ipsum,
             dolor: Dolor,
             sit: Sit,
             amet: Amet);

    fn lorem(ipsum: Ipsum,
             dolor: Dolor,
             sit: Sit,
             amet: Amet) {
        // body
    }

    fn lorem(ipsum: Ipsum,
             dolor: Dolor,
             sit: Sit,
             amet: Amet,
             consectetur: Consectetur,
             adipiscing: Adipiscing,
             elit: Elit);

    fn lorem(ipsum: Ipsum,
             dolor: Dolor,
             sit: Sit,
             amet: Amet,
             consectetur: Consectetur,
             adipiscing: Adipiscing,
             elit: Elit) {
        // body
    }
}
```

## `fn_args_layout`

Layout of function arguments and tuple structs

- **Default value**: `"Block"`
- **Possible values**: `"Block"`, `"Visual"`

#### `"Block"` (default):

```rust
fn lorem() {}

fn lorem(ipsum: usize) {}

fn lorem(
    ipsum: usize,
    dolor: usize,
    sit: usize,
    amet: usize,
    consectetur: usize,
    adipiscing: usize,
    elit: usize,
) {
    // body
}
```

#### `"Visual"`:

```rust
fn lorem() {}

fn lorem(ipsum: usize) {}

fn lorem(ipsum: usize,
         dolor: usize,
         sit: usize,
         amet: usize,
         consectetur: usize,
         adipiscing: usize,
         elit: usize) {
    // body
}
```

## `fn_args_paren_newline`

If function argument parenthesis goes on a newline

- **Default value**: `false`
- **Possible values**: `true`, `false`

#### `false` (default):

```rust
fn lorem(
    ipsum: Ipsum,
    dolor: Dolor,
    sit: Sit,
    amet: Amet,
) -> DolorSitAmetConsecteturAdipiscingElitLoremIpsumDolorSitAmetConsecteturAdipiscingElit {
    // body
}
```

#### `true`:

```rust
fn lorem
    (
    ipsum: Ipsum,
    dolor: Dolor,
    sit: Sit,
    amet: Amet,
) -> DolorSitAmetConsecteturAdipiscingElitLoremIpsumDolorSitAmetConsecteturAdipiscingElit {
    // body
}
```

## `fn_brace_style`

Brace style for functions

- **Default value**: `"SameLineWhere"`
- **Possible values**: `"AlwaysNextLine"`, `"PreferSameLine"`, `"SameLineWhere"`

#### `"SameLineWhere"` (default):

```rust
fn lorem() {
    // body
}

fn lorem(ipsum: usize) {
    // body
}

fn lorem<T>(ipsum: T)
where
    T: Add + Sub + Mul + Div,
{
    // body
}
```

#### `"AlwaysNextLine"`:

```rust
fn lorem()
{
    // body
}

fn lorem(ipsum: usize)
{
    // body
}

fn lorem<T>(ipsum: T)
where
    T: Add + Sub + Mul + Div,
{
    // body
}
```

#### `"PreferSameLine"`:

```rust
fn lorem() {
    // body
}

fn lorem(ipsum: usize) {
    // body
}

fn lorem<T>(ipsum: T)
where
    T: Add + Sub + Mul + Div, {
    // body
}
```

## `fn_call_style`

Indentation for function calls, etc.

- **Default value**: `"Block"`
- **Possible values**: `"Block"`, `"Visual"`

#### `"Block"` (default):

```rust
lorem(
    "lorem",
    "ipsum",
    "dolor",
    "sit",
    "amet",
    "consectetur",
    "adipiscing",
    "elit",
);
```

#### `"Visual"`:

```rust
lorem("lorem",
      "ipsum",
      "dolor",
      "sit",
      "amet",
      "consectetur",
      "adipiscing",
      "elit");
```

## `fn_call_width`

Maximum width of the args of a function call before falling back to vertical formatting

- **Default value**: `60`
- **Possible values**: any positive integer

**Note:** A value of `0` results in vertical formatting being applied regardless of a line's width.

#### Function call shorter than `fn_call_width`:
```rust
lorem("lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing", "elit");
```

#### Function call longer than `fn_call_width`:

See [`fn_call_style`](#fn_call_style).

## `fn_empty_single_line`

Put empty-body functions on a single line

- **Default value**: `true`
- **Possible values**: `true`, `false`

#### `true` (default):

```rust
fn lorem() {}
```

#### `false`:

```rust
fn lorem() {
}
```

See also [`control_brace_style`](#control_brace_style).

## `fn_return_indent`

Location of return type in function declaration

- **Default value**: `"WithArgs"`
- **Possible values**: `"WithArgs"`, `"WithWhereClause"`

#### `"WithArgs"` (default):

```rust
fn lorem(ipsum: Ipsum,
         dolor: Dolor,
         sit: Sit,
         amet: Amet,
         consectetur: Consectetur,
         adipiscing: Adipiscing)
         -> Elit
    where Ipsum: Eq
{
    // body
}

```

#### `"WithWhereClause"`:

```rust
fn lorem(ipsum: Ipsum,
         dolor: Dolor,
         sit: Sit,
         amet: Amet,
         consectetur: Consectetur,
         adipiscing: Adipiscing)
    -> Elit
    where Ipsum: Eq
{
    // body
}

```

**Note**: This option only takes effect when `fn_call_style` is set to `"Visual"`.

## `fn_single_line`

Put single-expression functions on a single line

- **Default value**: `false`
- **Possible values**: `true`, `false`

#### `false` (default):

```rust
fn lorem() -> usize {
    42
}

fn lorem() -> usize {
    let ipsum = 42;
    ipsum
}
```

#### `true`:

```rust
fn lorem() -> usize { 42 }

fn lorem() -> usize {
    let ipsum = 42;
    ipsum
}
```

See also [`control_brace_style`](#control_brace_style).

## `force_explicit_abi`

Always print the abi for extern items

- **Default value**: `true`
- **Possible values**: `true`, `false`

**Note:** Non-"C" ABIs are always printed. If `false` then "C" is removed.

#### `true` (default):

```rust
extern "C" {
    pub static lorem: c_int;
}
```

#### `false`:

```rust
extern {
    pub static lorem: c_int;
}
```

## `force_format_strings`

Always format string literals

- **Default value**: `false`
- **Possible values**: `true`, `false`

See [`format_strings`](#format_strings).

See also [`max_width`](#max_width).

## `format_strings`

Format string literals where necessary

- **Default value**: `false`
- **Possible values**: `true`, `false`

#### `false` (default):

```rust
let lorem = "ipsum dolor sit amet consectetur adipiscing elit lorem ipsum dolor sit";
```

#### `true`:

```rust
let lorem =
    "ipsum dolor sit amet consectetur \
     adipiscing elit lorem ipsum dolor sit";
```

See also [`force_format_strings`](#force_format_strings), [`max_width`](#max_width).

## `generics_indent`

Indentation of generics

- **Default value**: `"Block"`
- **Possible values**: `"Block"`, `"Visual"`

#### `"Block"` (default):

```rust
fn lorem<
    Ipsum: Eq = usize,
    Dolor: Eq = usize,
    Sit: Eq = usize,
    Amet: Eq = usize,
    Adipiscing: Eq = usize,
    Consectetur: Eq = usize,
    Elit: Eq = usize
>(
    ipsum: Ipsum,
    dolor: Dolor,
    sit: Sit,
    amet: Amet,
    adipiscing: Adipiscing,
    consectetur: Consectetur,
    elit: Elit,
) -> T {
    // body
}
```

#### `"Visual"`:

```rust
fn lorem<Ipsum: Eq = usize,
         Dolor: Eq = usize,
         Sit: Eq = usize,
         Amet: Eq = usize,
         Adipiscing: Eq = usize,
         Consectetur: Eq = usize,
         Elit: Eq = usize>
    (ipsum: Ipsum,
     dolor: Dolor,
     sit: Sit,
     amet: Amet,
     adipiscing: Adipiscing,
     consectetur: Consectetur,
     elit: Elit)
     -> T {
    // body
}
```

## `hard_tabs`

Use tab characters for indentation, spaces for alignment

- **Default value**: `false`
- **Possible values**: `true`, `false`

#### `false` (default):

```rust
fn lorem() -> usize {
    42 // spaces before 42
}
```

#### `true`:

```rust
fn lorem() -> usize {
	42 // tabs before 42
}
```

See also: [`tab_spaces`](#tab_spaces).

## `impl_empty_single_line`

Put empty-body implementations on a single line

- **Default value**: `true`
- **Possible values**: `true`, `false`

#### `true` (default):

```rust
impl Lorem {}
```

#### `false`:

```rust
impl Lorem {
}
```

See also [`item_brace_style`](#item_brace_style).

## `indent_match_arms`

Indent match arms instead of keeping them at the same indentation level as the match keyword

- **Default value**: `true`
- **Possible values**: `true`, `false`

#### `true` (default):

```rust
match lorem {
    Lorem::Ipsum => (),
    Lorem::Dolor => (),
    Lorem::Sit => (),
    Lorem::Amet => (),
}
```

#### `false`:

```rust
match lorem {
Lorem::Ipsum => (),
Lorem::Dolor => (),
Lorem::Sit => (),
Lorem::Amet => (),
}
```

See also: [`match_block_trailing_comma`](#match_block_trailing_comma), [`wrap_match_arms`](#wrap_match_arms).

## `imports_indent`

Indent style of imports

- **Default Value**: `"Visual"`
- **Possible values**: `"Block"`, `"Visual"`

#### `"Visual"` (default):

```rust
use foo::{xxx,
          yyy,
          zzz};
```

#### `"Block"`:

```rust
use foo::{
    xxx,
    yyy,
    zzz,
};
```

See also: [`imports_layout`](#imports_layout).

## `imports_layout`

Item layout inside a imports block

- **Default value**: "Mixed"
- **Possible values**: "Horizontal", "HorizontalVertical", "Mixed", "Vertical"

#### `"Mixed"` (default):

```rust
use foo::{xxx, yyy, zzz};

use foo::{aaa, bbb, ccc,
          ddd, eee, fff};
```

#### `"Horizontal"`:

**Note**: This option forces to put everything on one line and may exceeds `max_width`.

```rust
use foo::{xxx, yyy, zzz};

use foo::{aaa, bbb, ccc, ddd, eee, fff};
```

#### `"HorizontalVertical"`

```rust
use foo::{xxx, yyy, zzz};

use foo::{aaa,
          bbb,
          ccc,
          ddd,
          eee,
          fff};
```

#### `"Vertical"`

```rust
use foo::{xxx,
          yyy,
          zzz};

use foo::{aaa,
          bbb,
          ccc,
          ddd,
          eee,
          fff};
```

## `item_brace_style`

Brace style for structs and enums

- **Default value**: `"SameLineWhere"`
- **Possible values**: `"AlwaysNextLine"`, `"PreferSameLine"`, `"SameLineWhere"`

#### `"SameLineWhere"` (default):

```rust
struct Lorem {
    ipsum: bool,
}

struct Dolor<T>
    where T: Eq
{
    sit: T,
}
```

#### `"AlwaysNextLine"`:

```rust
struct Lorem
{
    ipsum: bool,
}

struct Dolor<T>
    where T: Eq
{
    sit: T,
}
```

#### `"PreferSameLine"`:

```rust
struct Lorem {
    ipsum: bool,
}

struct Dolor<T>
    where T: Eq {
    sit: T,
}
```

## `match_arm_forces_newline`

Consistently put match arms (block based or not) in a newline.

- **Default value**: `false`
- **Possible values**: `true`, `false`

#### `false` (default):

```rust
match x {
    // a non-empty block
    X0 => {
        f();
    }
    // an empty block
    X1 => {}
    // a non-block
    X2 => println!("ok"),
}
```

#### `true`:

```rust
match x {
    // a non-empty block
    X0 => {
        f();
    }
    // an empty block
    X1 =>
        {}
    // a non-block
    X2 => {
        println!("ok")
    }
}
```

See also: [`wrap_match_arms`](#wrap_match_arms).

## `match_block_trailing_comma`

Put a trailing comma after a block based match arm (non-block arms are not affected)

- **Default value**: `false`
- **Possible values**: `true`, `false`

#### `false` (default):

```rust
match lorem {
    Lorem::Ipsum => {
        println!("ipsum");
    }
    Lorem::Dolor => println!("dolor"),
}
```

#### `true`:

```rust
match lorem {
    Lorem::Ipsum => {
        println!("ipsum");
    },
    Lorem::Dolor => println!("dolor"),
}
```

See also: [`indent_match_arms`](#indent_match_arms), [`trailing_comma`](#trailing_comma), [`wrap_match_arms`](#wrap_match_arms).

## `match_pattern_separator_break_point`

Put a match sub-patterns' separator (`|`) in front or back.

- **Default value**: `"Back"`
- **Possible values**: `"Back"`, `"Front"`

#### `"Back"` (default):

```rust
match m {
    Variant::Tag |
    Variant::Tag2 |
    Variant::Tag3 |
    Variant::Tag4 |
    Variant::Tag5 |
    Variant::Tag6 => {}
}
```

#### `Front`:

```rust
match m {
    Variant::Tag
    | Variant::Tag2
    | Variant::Tag3
    | Variant::Tag4
    | Variant::Tag5
    | Variant::Tag6 => {}
}
```

## `max_width`

Maximum width of each line

- **Default value**: `100`
- **Possible values**: any positive integer

See also [`error_on_line_overflow`](#error_on_line_overflow).

## `merge_derives`

Merge multiple derives into a single one.

- **Default value**: `true`
- **Possible values**: `true`, `false`

#### `true` (default):

```rust
#[derive(Eq, PartialEq, Debug, Copy, Clone)]
pub enum Foo {}
```

#### `false`:

```rust
#[derive(Eq, PartialEq)]
#[derive(Debug)]
#[derive(Copy, Clone)]
pub enum Foo {}
```

## `multiline_closure_forces_block`

Force multiline closure bodies to be wrapped in a block

- **Default value**: `false`
- **Possible values**: `false`, `true`

#### `false` (default):

```rust
result.and_then(|maybe_value| match maybe_value {
    None => ...,
    Some(value) => ...,
})
```

#### `true`:

```rust

result.and_then(|maybe_value| {
    match maybe_value {
        None => ...,
        Some(value) => ...,
    }
})
```

## `multiline_match_arm_forces_block`

Force multiline match arm bodies to be wrapped in a block

- **Default value**: `false`
- **Possible values**: `false`, `true`

#### `false` (default):

```rust
match lorem {
    None => if ipsum {
        println!("Hello World");
    },
    Some(dolor) => ...,
}
```

#### `true`:

```rust
match lorem {
    None => {
        if ipsum {
            println!("Hello World");
        }
    }
    Some(dolor) => ...,
}
```

## `newline_style`

Unix or Windows line endings

- **Default value**: `"Unix"`
- **Possible values**: `"Native"`, `"Unix"`, `"Windows"`

## `normalize_comments`

Convert /* */ comments to // comments where possible

- **Default value**: `false`
- **Possible values**: `true`, `false`

#### `false` (default):

```rust
// Lorem ipsum:
fn dolor() -> usize {}

/* sit amet: */
fn adipiscing() -> usize {}
```

#### `true`:

```rust
// Lorem ipsum:
fn dolor() -> usize {}

// sit amet:
fn adipiscing() -> usize {}
```

## `reorder_imported_names`

Reorder lists of names in import statements alphabetically

- **Default value**: `false`
- **Possible values**: `true`, `false`

#### `false` (default):

```rust
use super::{lorem, ipsum, dolor, sit};
```

#### `true`:

```rust
use super::{dolor, ipsum, lorem, sit};
```

See also [`reorder_imports`](#reorder_imports).

## `reorder_imports`

Reorder import statements alphabetically

- **Default value**: `false`
- **Possible values**: `true`, `false`

#### `false` (default):

```rust
use lorem;
use ipsum;
use dolor;
use sit;
```

#### `true`:

```rust
use dolor;
use ipsum;
use lorem;
use sit;
```

See also [`reorder_imported_names`](#reorder_imported_names), [`reorder_imports_in_group`](#reorder_imports_in_group).

## `reorder_imports_in_group`

Reorder import statements in group

- **Default value**: `false`
- **Possible values**: `true`, `false`

**Note:** This option takes effect only when [`reorder_imports`](#reorder_imports) is set to `true`.

#### `false` (default):

```rust
use std::mem;
use std::io;

use lorem;
use ipsum;
use dolor;
use sit;
```

#### `true`:

```rust
use std::io;
use std::mem;

use dolor;
use ipsum;
use lorem;
use sit;
```

See also [`reorder_imports`](#reorder_imports).

## `single_line_if_else_max_width`

Maximum line length for single line if-else expressions.

- **Default value**: `50`
- **Possible values**: any positive integer

**Note:** A value of `0` results in if-else expressions being broken regardless of their line's width.

#### Lines shorter than `single_line_if_else_max_width`:
```rust
let lorem = if ipsum { dolor } else { sit };
```

#### Lines longer than `single_line_if_else_max_width`:
```rust
let lorem = if ipsum {
    dolor
} else {
    sit
};
```

See also: [`control_brace_style`](#control_brace_style).

## `skip_children`

Don't reformat out of line modules

- **Default value**: `false`
- **Possible values**: `true`, `false`

## `space_after_bound_colon`

Leave a space after the colon in a trait or lifetime bound

- **Default value**: `true`
- **Possible values**: `true`, `false`

#### `true` (default):

```rust
fn lorem<T: Eq>(t: T) {
    // body
}
```

#### `false`:

```rust
fn lorem<T:Eq>(t: T) {
    // body
}
```

See also: [`space_before_bound`](#space_before_bound).

## `struct_field_align_threshold`

The maximum diff of width between struct fields to be aligned with each other.

- **Default value** : 0
- **Possible values**: any positive integer

#### `0` (default):

```rust
struct Foo {
    x: u32,
    yy: u32,
    zzz: u32,
}
```

#### `20`:

```rust
struct Foo {
    x:   u32,
    yy:  u32,
    zzz: u32,
}
```

## `space_after_struct_lit_field_colon`

Leave a space after the colon in a struct literal field

- **Default value**: `true`
- **Possible values**: `true`, `false`

#### `true` (default):

```rust
let lorem = Lorem {
    ipsum: dolor,
    sit: amet,
};
```

#### `false`:

```rust
let lorem = Lorem {
    ipsum:dolor,
    sit:amet,
};
```

See also: [`space_before_struct_lit_field_colon`](#space_before_struct_lit_field_colon).

## `space_after_type_annotation_colon`

Leave a space after the colon in a type annotation

- **Default value**: `true`
- **Possible values**: `true`, `false`

#### `true` (default):

```rust
fn lorem<T: Eq>(t: T) {
    let ipsum: Dolor = sit;
}
```

#### `false`:

```rust
fn lorem<T: Eq>(t:T) {
    let ipsum:Dolor = sit;
}
```

See also: [`space_before_type_annotation`](#space_before_type_annotation).

## `space_before_bound`

Leave a space before the colon in a trait or lifetime bound

- **Default value**: `false`
- **Possible values**: `true`, `false`

#### `false` (default):

```rust
fn lorem<T: Eq>(t: T) {
    let ipsum: Dolor = sit;
}
```

#### `true`:

```rust
fn lorem<T : Eq>(t: T) {
    let ipsum: Dolor = sit;
}
```

See also: [`space_after_bound_colon`](#space_after_bound_colon).

## `space_before_struct_lit_field_colon`

Leave a space before the colon in a struct literal field

- **Default value**: `false`
- **Possible values**: `true`, `false`

#### `false` (default):

```rust
let lorem = Lorem {
    ipsum: dolor,
    sit: amet,
};
```

#### `true`:

```rust
let lorem = Lorem {
    ipsum : dolor,
    sit : amet,
};
```

See also: [`space_after_struct_lit_field_colon`](#space_after_struct_lit_field_colon).

## `space_before_type_annotation`

Leave a space before the colon in a type annotation

- **Default value**: `false`
- **Possible values**: `true`, `false`

#### `false` (default):

```rust
fn lorem<T: Eq>(t: T) {
    let ipsum: Dolor = sit;
}
```

#### `true`:

```rust
fn lorem<T: Eq>(t : T) {
    let ipsum : Dolor = sit;
}
```

See also: [`space_after_type_annotation_colon`](#space_after_type_annotation_colon).

## `spaces_around_ranges`

Put spaces around the .. and ... range operators

- **Default value**: `false`
- **Possible values**: `true`, `false`

#### `false` (default):

```rust
let lorem = 0..10;
```

#### `true`:

```rust
let lorem = 0 .. 10;
```

## `spaces_within_angle_brackets`

Put spaces within non-empty generic arguments

- **Default value**: `false`
- **Possible values**: `true`, `false`

#### `false` (default):

```rust
fn lorem<T: Eq>(t: T) {
    // body
}
```

#### `true`:

```rust
fn lorem< T: Eq >(t: T) {
    // body
}
```

See also: [`spaces_within_parens`](#spaces_within_parens), [`spaces_within_square_brackets`](#spaces_within_square_brackets).

## `spaces_within_parens`

Put spaces within non-empty parentheses

- **Default value**: `false`
- **Possible values**: `true`, `false`

#### `false` (default):

```rust
fn lorem<T: Eq>(t: T) {
    let lorem = (ipsum, dolor);
}
```

#### `true`:

```rust
fn lorem<T: Eq>( t: T ) {
    let lorem = ( ipsum, dolor );
}
```

See also: [`spaces_within_angle_brackets`](#spaces_within_angle_brackets), [`spaces_within_square_brackets`](#spaces_within_square_brackets).

## `spaces_within_square_brackets`

Put spaces within non-empty square brackets

- **Default value**: `false`
- **Possible values**: `true`, `false`

#### `false` (default):

```rust
let lorem: [usize; 2] = [ipsum, dolor];
```

#### `true`:

```rust
let lorem: [ usize; 2 ] = [ ipsum, dolor ];
```

See also: [`spaces_within_parens`](#spaces_within_parens), [`spaces_within_angle_brackets`](#spaces_within_angle_brackets).

## `struct_lit_multiline_style`

Multiline style on literal structs

- **Default value**: `"PreferSingle"`
- **Possible values**: `"ForceMulti"`, `"PreferSingle"`

#### `"PreferSingle"` (default):

```rust
let lorem = Lorem { ipsum: dolor, sit: amet };
```

#### `"ForceMulti"`:

```rust
let lorem = Lorem {
    ipsum: dolor,
    sit: amet,
};
```

See also: [`struct_lit_style`](#struct_lit_style), [`struct_lit_width`](#struct_lit_width).

## `struct_lit_style`

Style of struct definition

- **Default value**: `"Block"`
- **Possible values**: `"Block"`, `"Visual"`

#### `"Block"` (default):

```rust
let lorem = Lorem {
    ipsum: dolor,
    sit: amet,
};
```

#### `"Visual"`:

```rust
let lorem = Lorem { ipsum: dolor,
                    sit: amet, };
```

See also: [`struct_lit_multiline_style`](#struct_lit_multiline_style), [`struct_lit_style`](#struct_lit_style).

## `struct_lit_width`

Maximum width in the body of a struct lit before falling back to vertical formatting

- **Default value**: `18`
- **Possible values**: any positive integer

**Note:** A value of `0` results in vertical formatting being applied regardless of a line's width.

#### Lines shorter than `struct_lit_width`:
```rust
let lorem = Lorem { ipsum: dolor, sit: amet };
```

#### Lines longer than `struct_lit_width`:
See [`struct_lit_style`](#struct_lit_style).

See also: [`struct_lit_multiline_style`](#struct_lit_multiline_style), [`struct_lit_style`](#struct_lit_style).

## `struct_variant_width`

Maximum width in the body of a struct variant before falling back to vertical formatting

- **Default value**: `35`
- **Possible values**: any positive integer

**Note:** A value of `0` results in vertical formatting being applied regardless of a line's width.

#### Struct variants shorter than `struct_variant_width`:
```rust
enum Lorem {
    Ipsum,
    Dolor(bool),
    Sit { amet: Consectetur, adipiscing: Elit },
}
```

#### Struct variants longer than `struct_variant_width`:
```rust
enum Lorem {
    Ipsum,
    Dolor(bool),
    Sit {
        amet: Consectetur,
        adipiscing: Elit,
    },
}
```

## `tab_spaces`

Number of spaces per tab

- **Default value**: `4`
- **Possible values**: any positive integer

#### `4` (default):

```rust
fn lorem() {
    let ipsum = dolor();
    let sit = vec![
        "amet consectetur adipiscing elit."
    ];
}
```

#### `2`:

```rust
fn lorem() {
  let ipsum = dolor();
  let sit = vec![
    "amet consectetur adipiscing elit."
  ];
}
```

See also: [`hard_tabs`](#hard_tabs).

## `take_source_hints`

Retain some formatting characteristics from the source code

- **Default value**: `false`
- **Possible values**: `true`, `false`

#### `false` (default):

```rust
lorem
    .ipsum()
    .dolor(|| { sit.amet().consectetur().adipiscing().elit(); });
```

#### `true`:

```rust
lorem
    .ipsum()
    .dolor(|| {
               sit.amet()
                   .consectetur()
                   .adipiscing()
                   .elit();
           });
```

Note: This only applies if the call chain within the inner closure had already been formatted on separate lines before running rustfmt.

## `trailing_comma`

How to handle trailing commas for lists

- **Default value**: `"Vertical"`
- **Possible values**: `"Always"`, `"Never"`, `"Vertical"`

#### `"Vertical"` (default):

```rust
let Lorem { ipsum, dolor, sit } = amet;
let Lorem {
    ipsum,
    dolor,
    sit,
    amet,
    consectetur,
    adipiscing,
} = elit;
```

#### `"Always"`:

```rust
let Lorem { ipsum, dolor, sit, } = amet;
let Lorem {
    ipsum,
    dolor,
    sit,
    amet,
    consectetur,
    adipiscing,
} = elit;
```

#### `"Never"`:

```rust
let Lorem { ipsum, dolor, sit } = amet;
let Lorem {
    ipsum,
    dolor,
    sit,
    amet,
    consectetur,
    adipiscing
} = elit;
```

See also: [`match_block_trailing_comma`](#match_block_trailing_comma).

## `trailing_semicolon`

Add trailing semicolon after break, continue and return

- **Default value**: `true`
- **Possible values**: `true`, `false`

#### `true` (default):
```rust
fn foo() -> usize {
    return 0;
}
```

#### `false`:
```rust
fn foo() -> usize {
    return 0
}
```

## `type_punctuation_density`

Determines if `+` or `=` are wrapped in spaces in the punctuation of types

- **Default value**: `"Wide"`
- **Possible values**: `"Compressed"`, `"Wide"`

#### `"Wide"` (default):

```rust
fn lorem<Ipsum: Dolor + Sit = Amet>() {
	// body
}
```

#### `"Compressed"`:

```rust
fn lorem<Ipsum: Dolor+Sit=Amet>() {
	// body
}
```

## `use_try_shorthand`

Replace uses of the try! macro by the ? shorthand

- **Default value**: `false`
- **Possible values**: `true`, `false`

#### `false` (default):

```rust
let lorem = try!(ipsum.map(|dolor|dolor.sit()));
```

#### `true`:

```rust
let lorem = ipsum.map(|dolor| dolor.sit())?;
```

## `where_density`

Density of a where clause.

- **Default value**: `"CompressedIfEmpty"`
- **Possible values**: `"Compressed"`, `"CompressedIfEmpty"`, `"Tall"`, `"Vertical"`

#### `"CompressedIfEmpty"` (default):

```rust
trait Lorem {
    fn ipsum<Dolor>(dolor: Dolor) -> Sit
    where Dolor: Eq;

    fn ipsum<Dolor>(dolor: Dolor) -> Sit
    where
        Dolor: Eq,
    {
        // body
    }
}
```

#### `"Compressed"`:

```rust
trait Lorem {
    fn ipsum<Dolor>(dolor: Dolor) -> Sit
    where Dolor: Eq;

    fn ipsum<Dolor>(dolor: Dolor) -> Sit
    where Dolor: Eq {
        // body
    }
}
```

#### `"Tall"`:

```rust
trait Lorem {
    fn ipsum<Dolor>(dolor: Dolor) -> Sit
    where
        Dolor: Eq;

    fn ipsum<Dolor>(dolor: Dolor) -> Sit
    where
        Dolor: Eq,
    {
        // body
    }
}
```

**Note:** `where_density = "Tall"` currently produces the same output as `where_density = "Vertical"`.

#### `"Vertical"`:

```rust
trait Lorem {
    fn ipsum<Dolor>(dolor: Dolor) -> Sit
        where Dolor: Eq;

    fn ipsum<Dolor>(dolor: Dolor) -> Sit
        where Dolor: Eq
    {
        // body
    }
}
```

**Note:** `where_density = "Vertical"` currently produces the same output as `where_density = "Tall"`.

See also: [`where_layout`](#where_layout), [`where_pred_indent`](#where_pred_indent), [`where_style`](#where_style).

## `where_layout`

Element layout inside a where clause

- **Default value**: `"Vertical"`
- **Possible values**: `"Horizontal"`, `"HorizontalVertical"`, `"Mixed"`, `"Vertical"`

#### `"Vertical"` (default):

```rust
fn lorem<Ipsum, Dolor>(ipsum: Ipsum, dolor: Dolor)
    where Ipsum: IpsumDolorSitAmet,
          Dolor: DolorSitAmetConsectetur
{
    // body
}

fn lorem<Ipsum, Dolor, Sit, Amet>(ipsum: Ipsum, dolor: Dolor, sit: Sit, amet: Amet)
    where Ipsum: IpsumDolorSitAmet,
          Dolor: DolorSitAmetConsectetur,
          Sit: SitAmetConsecteturAdipiscing,
          Amet: AmetConsecteturAdipiscingElit
{
    // body
}
```

#### `"Horizontal"`:

```rust
fn lorem<Ipsum, Dolor>(ipsum: Ipsum, dolor: Dolor)
    where Ipsum: IpsumDolorSitAmet, Dolor: DolorSitAmetConsectetur
{
    // body
}

fn lorem<Ipsum, Dolor, Sit, Amet>(ipsum: Ipsum, dolor: Dolor, sit: Sit, amet: Amet)
    where Ipsum: IpsumDolorSitAmet, Dolor: DolorSitAmetConsectetur, Sit: SitAmetConsecteturAdipiscing, Amet: AmetConsecteturAdipiscingElit
{
    // body
}
```

#### `"HorizontalVertical"`:

```rust
fn lorem<Ipsum, Dolor>(ipsum: Ipsum, dolor: Dolor)
    where Ipsum: IpsumDolorSitAmet, Dolor: DolorSitAmetConsectetur
{
    // body
}

fn lorem<Ipsum, Dolor, Sit, Amet>(ipsum: Ipsum, dolor: Dolor, sit: Sit, amet: Amet)
    where Ipsum: IpsumDolorSitAmet,
          Dolor: DolorSitAmetConsectetur,
          Sit: SitAmetConsecteturAdipiscing,
          Amet: AmetConsecteturAdipiscingElit
{
    // body
}
```

#### `"Mixed"`:

```rust
fn lorem<Ipsum, Dolor>(ipsum: Ipsum, dolor: Dolor)
    where Ipsum: IpsumDolorSitAmet, Dolor: DolorSitAmetConsectetur
{
    // body
}

fn lorem<Ipsum, Dolor, Sit, Amet>(ipsum: Ipsum, dolor: Dolor, sit: Sit, amet: Amet)
    where Ipsum: IpsumDolorSitAmet, Dolor: DolorSitAmetConsectetur,
          Sit: SitAmetConsecteturAdipiscing, Amet: AmetConsecteturAdipiscingElit
{
    // body
}
```

**Note**: This option only takes effect when `where_style` is set to `"Legacy"`.

See also: [`where_density`](#where_density), [`where_pred_indent`](#where_pred_indent), [`where_style`](#where_style).

## `where_pred_indent`

Indentation style of a where predicate

- **Default value**: `"Visual"`
- **Possible values**: `"Block"`, `"Visual"`

#### `"Visual"` (default):

```rust
fn lorem<Ipsum, Dolor, Sit, Amet>() -> T
    where Ipsum: Eq,
          Dolor: Eq,
          Sit: Eq,
          Amet: Eq
{
    // body
}
```

#### `"Block"`:

```rust
fn lorem<Ipsum, Dolor, Sit, Amet>() -> T
    where Ipsum: Eq,
        Dolor: Eq,
        Sit: Eq,
        Amet: Eq
{
    // body
}
```

**Note**: This option only takes effect when `where_style` is set to `"Legacy"`.

See also: [`where_density`](#where_density), [`where_layout`](#where_layout), [`where_style`](#where_style).

## `where_style`

Overall strategy for where clauses

- **Default value**: `"Rfc"`
- **Possible values**: `"Rfc"`, `"Legacy"`

#### `"Rfc"` (default):

```rust
fn lorem<Ipsum, Dolor, Sit, Amet>() -> T
where
    Ipsum: Eq,
    Dolor: Eq,
    Sit: Eq,
    Amet: Eq,
{
    // body
}
```

#### `"Legacy"`:

```rust
fn lorem<Ipsum, Dolor, Sit, Amet>() -> T
    where Ipsum: Eq,
          Dolor: Eq,
          Sit: Eq,
          Amet: Eq
{
    // body
}
```

See also: [`where_density`](#where_density), [`where_layout`](#where_layout), [`where_pred_indent`](#where_pred_indent).

## `wrap_comments`

Break comments to fit on the line

- **Default value**: `false`
- **Possible values**: `true`, `false`

#### `false` (default):

```rust
// Lorem ipsum dolor sit amet, consectetur adipiscing elit, sed do eiusmod tempor incididunt ut labore et dolore magna aliqua. Ut enim ad minim veniam, quis nostrud exercitation ullamco laboris nisi ut aliquip ex ea commodo consequat.
```

#### `true`:

```rust
// Lorem ipsum dolor sit amet, consectetur adipiscing elit,
// sed do eiusmod tempor incididunt ut labore et dolore
// magna aliqua. Ut enim ad minim veniam, quis nostrud
// exercitation ullamco laboris nisi ut aliquip ex ea
// commodo consequat.
```

## `wrap_match_arms`

Wrap the body of arms in blocks when it does not fit on the same line with the pattern of arms

- **Default value**: `true`
- **Possible values**: `true`, `false`

#### `true` (default):

```rust
match lorem {
    true => {
        foooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo(x)
    }
    false => println!("{}", sit),
}
```

#### `false`:

```rust
match lorem {
    true =>
        foooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo(x),
    false => println!("{}", sit),
}
```

See also: [`indent_match_arms`](#indent_match_arms), [`match_block_trailing_comma`](#match_block_trailing_comma).

## `write_mode`

What Write Mode to use when none is supplied: Replace, Overwrite, Display, Diff, Coverage

- **Default value**: `"Overwrite"`
- **Possible values**: `"Checkstyle"`, `"Coverage"`, `"Diff"`, `"Display"`, `"Overwrite"`, `"Plain"`, `"Replace"`
