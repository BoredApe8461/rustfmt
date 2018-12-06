# Configuring Rustfmt

Rustfmt is designed to be very configurable. You can create a TOML file called `rustfmt.toml` or `.rustfmt.toml`, place it in the project or any other parent directory and it will apply the options in that file.

A possible content of `rustfmt.toml` or `.rustfmt.toml` might look like this:

```toml
indent_style = "Block"
reorder_imports = false
```

Each configuration option is either stable or unstable.
Stable options can be used directly, while unstable options are opt-in.
To enable unstable options, set `unstable_features = true` in `rustfmt.toml` or pass `--unstable-features` to rustfmt.

# Configuration Options

Below you find a detailed visual guide on all the supported configuration options of rustfmt:


## `indent_style`

Indent on expressions or items.

- **Default value**: `"Block"`
- **Possible values**: `"Block"`, `"Visual"`
- **Stable**: No

### Array

#### `"Block"` (default):

```rust
fn main() {
    let lorem = vec![
        "ipsum",
        "dolor",
        "sit",
        "amet",
        "consectetur",
        "adipiscing",
        "elit",
    ];
}
```

#### `"Visual"`:

```rust
fn main() {
    let lorem = vec!["ipsum",
                     "dolor",
                     "sit",
                     "amet",
                     "consectetur",
                     "adipiscing",
                     "elit"];
}
```

### Control flow

#### `"Block"` (default):

```rust
fn main() {
    if lorem_ipsum
        && dolor_sit
        && amet_consectetur
        && lorem_sit
        && dolor_consectetur
        && amet_ipsum
        && lorem_consectetur
    {
        // ...
    }
}
```

#### `"Visual"`:

```rust
fn main() {
    if lorem_ipsum
       && dolor_sit
       && amet_consectetur
       && lorem_sit
       && dolor_consectetur
       && amet_ipsum
       && lorem_consectetur
    {
        // ...
    }
}
```

See also: [`control_brace_style`](#control_brace_style).

### Function arguments

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

### Function calls

#### `"Block"` (default):

```rust
fn main() {
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
}
```

#### `"Visual"`:

```rust
fn main() {
    lorem("lorem",
          "ipsum",
          "dolor",
          "sit",
          "amet",
          "consectetur",
          "adipiscing",
          "elit");
}
```

### Generics

#### `"Block"` (default):

```rust
fn lorem<
    Ipsum: Eq = usize,
    Dolor: Eq = usize,
    Sit: Eq = usize,
    Amet: Eq = usize,
    Adipiscing: Eq = usize,
    Consectetur: Eq = usize,
    Elit: Eq = usize,
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
         Elit: Eq = usize>(
    ipsum: Ipsum,
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

#### Struct

#### `"Block"` (default):

```rust
fn main() {
    let lorem = Lorem {
        ipsum: dolor,
        sit: amet,
    };
}
```

#### `"Visual"`:

```rust
fn main() {
    let lorem = Lorem { ipsum: dolor,
                        sit: amet };
}
```

See also: [`struct_lit_single_line`](#struct_lit_single_line), [`indent_style`](#indent_style).

### Where predicates

#### `"Block"` (default):

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

#### `"Visual"`:

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

## `use_small_heuristics`

Whether to use different formatting for items and expressions if they satisfy a heuristic notion of 'small'.

- **Default value**: `Default`
- **Possible values**: `Default`, `Off`, `Max`
- **Stable**: Yes

#### `Default` (default):

```rust
enum Lorem {
    Ipsum,
    Dolor(bool),
    Sit { amet: Consectetur, adipiscing: Elit },
}

fn main() {
    lorem(
        "lorem",
        "ipsum",
        "dolor",
        "sit",
        "amet",
        "consectetur",
        "adipiscing",
    );

    let lorem = Lorem {
        ipsum: dolor,
        sit: amet,
    };
    let lorem = Lorem { ipsum: dolor };

    let lorem = if ipsum { dolor } else { sit };
}
```

#### `Off`:

```rust
enum Lorem {
    Ipsum,
    Dolor(bool),
    Sit {
        amet: Consectetur,
        adipiscing: Elit,
    },
}

fn main() {
    lorem("lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing");

    let lorem = Lorem {
        ipsum: dolor,
        sit: amet,
    };

    let lorem = if ipsum {
        dolor
    } else {
        sit
    };
}
```

#### `Max`:

```rust
enum Lorem {
    Ipsum,
    Dolor(bool),
    Sit { amet: Consectetur, adipiscing: Elit },
}

fn main() {
    lorem("lorem", "ipsum", "dolor", "sit", "amet", "consectetur", "adipiscing");

    let lorem = Lorem { ipsum: dolor, sit: amet };

    let lorem = if ipsum { dolor } else { sit };
}
```

## `binop_separator`

Where to put a binary operator when a binary expression goes multiline.

- **Default value**: `"Front"`
- **Possible values**: `"Front"`, `"Back"`
- **Stable**: No

#### `"Front"` (default):

```rust
fn main() {
    let or = foofoofoofoofoofoofoofoofoofoofoofoofoofoofoofoo
        || barbarbarbarbarbarbarbarbarbarbarbarbarbarbarbar;

    let sum = 123456789012345678901234567890
        + 123456789012345678901234567890
        + 123456789012345678901234567890;

    let range = aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa
        ..bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb;
}
```

#### `"Back"`:

```rust
fn main() {
    let or = foofoofoofoofoofoofoofoofoofoofoofoofoofoofoofoo ||
        barbarbarbarbarbarbarbarbarbarbarbarbarbarbarbar;

    let sum = 123456789012345678901234567890 +
        123456789012345678901234567890 +
        123456789012345678901234567890;

    let range = aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa..
        bbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbbb;
}
```

## `combine_control_expr`

Combine control expressions with function calls.

- **Default value**: `true`
- **Possible values**: `true`, `false`
- **Stable**: No

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
fn example() {
    // If
    foo!(
        if x {
            foo();
        } else {
            bar();
        }
    );

    // IfLet
    foo!(
        if let Some(..) = x {
            foo();
        } else {
            bar();
        }
    );

    // While
    foo!(
        while x {
            foo();
            bar();
        }
    );

    // WhileLet
    foo!(
        while let Some(..) = x {
            foo();
            bar();
        }
    );

    // ForLoop
    foo!(
        for x in y {
            foo();
            bar();
        }
    );

    // Loop
    foo!(
        loop {
            foo();
            bar();
        }
    );
}
```

## `comment_width`

Maximum length of comments. No effect unless`wrap_comments = true`.

- **Default value**: `80`
- **Possible values**: any positive integer
- **Stable**: No

**Note:** A value of `0` results in [`wrap_comments`](#wrap_comments) being applied regardless of a line's width.

#### `80` (default; comments shorter than `comment_width`):
```rust
// Lorem ipsum dolor sit amet, consectetur adipiscing elit.
```

#### `60` (comments longer than `comment_width`):
```rust
// Lorem ipsum dolor sit amet,
// consectetur adipiscing elit.
```

See also [`wrap_comments`](#wrap_comments).

## `condense_wildcard_suffixes`

Replace strings of _ wildcards by a single .. in tuple patterns

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: No

#### `false` (default):

```rust
fn main() {
    let (lorem, ipsum, _, _) = (1, 2, 3, 4);
    let (lorem, ipsum, ..) = (1, 2, 3, 4);
}
```

#### `true`:

```rust
fn main() {
    let (lorem, ipsum, ..) = (1, 2, 3, 4);
}
```

## `control_brace_style`

Brace style for control flow constructs

- **Default value**: `"AlwaysSameLine"`
- **Possible values**: `"AlwaysNextLine"`, `"AlwaysSameLine"`, `"ClosingNextLine"`
- **Stable**: No

#### `"AlwaysSameLine"` (default):

```rust
fn main() {
    if lorem {
        println!("ipsum!");
    } else {
        println!("dolor!");
    }
}
```

#### `"AlwaysNextLine"`:

```rust
fn main() {
    if lorem
    {
        println!("ipsum!");
    }
    else
    {
        println!("dolor!");
    }
}
```

#### `"ClosingNextLine"`:

```rust
fn main() {
    if lorem {
        println!("ipsum!");
    }
    else {
        println!("dolor!");
    }
}
```

## `disable_all_formatting`

Don't reformat anything

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: No

## `error_on_line_overflow`

Error if Rustfmt is unable to get all lines within `max_width`, except for comments and string
literals. If this happens, then it is a bug in Rustfmt. You might be able to work around the bug by
refactoring your code to avoid long/complex expressions, usually by extracting a local variable or
using a shorter name.

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: No

See also [`max_width`](#max_width).

## `error_on_unformatted`

Error if unable to get comments or string literals within `max_width`, or they are left with
trailing whitespaces.

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: No

## `fn_args_density`

Argument density in functions

- **Default value**: `"Tall"`
- **Possible values**: `"Compressed"`, `"Tall"`, `"Vertical"`
- **Stable**: No

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

#### `"Vertical"`:

```rust
trait Lorem {
    fn lorem(
        ipsum: Ipsum,
        dolor: Dolor,
        sit: Sit,
        amet: Amet,
    );

    fn lorem(
        ipsum: Ipsum,
        dolor: Dolor,
        sit: Sit,
        amet: Amet,
    ) {
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


## `brace_style`

Brace style for items

- **Default value**: `"SameLineWhere"`
- **Possible values**: `"AlwaysNextLine"`, `"PreferSameLine"`, `"SameLineWhere"`
- **Stable**: No

### Functions

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

### Structs and enums

#### `"SameLineWhere"` (default):

```rust
struct Lorem {
    ipsum: bool,
}

struct Dolor<T>
where
    T: Eq,
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
where
    T: Eq,
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
where
    T: Eq, {
    sit: T,
}
```


## `empty_item_single_line`

Put empty-body functions and impls on a single line

- **Default value**: `true`
- **Possible values**: `true`, `false`
- **Stable**: No

#### `true` (default):

```rust
fn lorem() {}

impl Lorem {}
```

#### `false`:

```rust
fn lorem() {
}

impl Lorem {
}
```

See also [`brace_style`](#brace_style), [`control_brace_style`](#control_brace_style).


## `enum_discrim_align_threshold`

The maximum length of enum variant having discriminant, that gets vertically aligned with others.
Variants without discriminants would be ignored for the purpose of alignment.

Note that this is not how much whitespace is inserted, but instead the longest variant name that
doesn't get ignored when aligning.

- **Default value** : 0
- **Possible values**: any positive integer
- **Stable**: No

#### `0` (default):

```rust
enum Bar {
    A = 0,
    Bb = 1,
    RandomLongVariantGoesHere = 10,
    Ccc = 71,
}

enum Bar {
    VeryLongVariantNameHereA = 0,
    VeryLongVariantNameHereBb = 1,
    VeryLongVariantNameHereCcc = 2,
}
```

#### `20`:

```rust
enum Foo {
    A   = 0,
    Bb  = 1,
    RandomLongVariantGoesHere = 10,
    Ccc = 2,
}

enum Bar {
    VeryLongVariantNameHereA = 0,
    VeryLongVariantNameHereBb = 1,
    VeryLongVariantNameHereCcc = 2,
}
```


## `fn_single_line`

Put single-expression functions on a single line

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: No

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


## `where_single_line`

Forces the `where` clause to be laid out on a single line.

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: No

#### `false` (default):

```rust
impl<T> Lorem for T
where
    Option<T>: Ipsum,
{
    // body
}
```

#### `true`:

```rust
impl<T> Lorem for T
where Option<T>: Ipsum
{
    // body
}
```

See also [`brace_style`](#brace_style), [`control_brace_style`](#control_brace_style).


## `force_explicit_abi`

Always print the abi for extern items

- **Default value**: `true`
- **Possible values**: `true`, `false`
- **Stable**: Yes

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

## `format_strings`

Format string literals where necessary

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: No

#### `false` (default):

```rust
fn main() {
    let lorem =
        "ipsum dolor sit amet consectetur adipiscing elit lorem ipsum dolor sit amet consectetur adipiscing";
}
```

#### `true`:

```rust
fn main() {
    let lorem = "ipsum dolor sit amet consectetur adipiscing elit lorem ipsum dolor sit amet \
                 consectetur adipiscing";
}
```

See also [`max_width`](#max_width).

## `format_macro_matchers`

Format the metavariable matching patterns in macros.

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: No

#### `false` (default):

```rust
macro_rules! foo {
    ($a: ident : $b: ty) => {
        $a(42): $b;
    };
    ($a: ident $b: ident $c: ident) => {
        $a = $b + $c;
    };
}
```

#### `true`:

```rust
macro_rules! foo {
    ($a:ident : $b:ty) => {
        $a(42): $b;
    };
    ($a:ident $b:ident $c:ident) => {
        $a = $b + $c;
    };
}
```

See also [`format_macro_bodies`](#format_macro_bodies).


## `format_macro_bodies`

Format the bodies of macros.

- **Default value**: `true`
- **Possible values**: `true`, `false`
- **Stable**: No

#### `true` (default):

```rust
macro_rules! foo {
    ($a: ident : $b: ty) => {
        $a(42): $b;
    };
    ($a: ident $b: ident $c: ident) => {
        $a = $b + $c;
    };
}
```

#### `false`:

```rust
macro_rules! foo {
    ($a: ident : $b: ty) => { $a(42): $b; };
    ($a: ident $b: ident $c: ident) => { $a=$b+$c; };
}
```

See also [`format_macro_matchers`](#format_macro_matchers).


## `hard_tabs`

Use tab characters for indentation, spaces for alignment

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: Yes

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


## `imports_indent`

Indent style of imports

- **Default Value**: `"Block"`
- **Possible values**: `"Block"`, `"Visual"`
- **Stable**: No

#### `"Block"` (default):

```rust
use foo::{
    xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx, yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy,
    zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz,
};
```

#### `"Visual"`:

```rust
use foo::{xxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxxx, yyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyyy,
          zzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzzz};
```

See also: [`imports_layout`](#imports_layout).

## `imports_layout`

Item layout inside a imports block

- **Default value**: "Mixed"
- **Possible values**: "Horizontal", "HorizontalVertical", "Mixed", "Vertical"
- **Stable**: No

#### `"Mixed"` (default):

```rust
use foo::{xxxxxxxxxxxxxxxxxx, yyyyyyyyyyyyyyyyyy, zzzzzzzzzzzzzzzzzz};

use foo::{
    aaaaaaaaaaaaaaaaaa, bbbbbbbbbbbbbbbbbb, cccccccccccccccccc, dddddddddddddddddd,
    eeeeeeeeeeeeeeeeee, ffffffffffffffffff,
};
```

#### `"Horizontal"`:

**Note**: This option forces all imports onto one line and may exceed `max_width`.

```rust
use foo::{xxx, yyy, zzz};

use foo::{aaa, bbb, ccc, ddd, eee, fff};
```

#### `"HorizontalVertical"`:

```rust
use foo::{xxxxxxxxxxxxxxxxxx, yyyyyyyyyyyyyyyyyy, zzzzzzzzzzzzzzzzzz};

use foo::{
    aaaaaaaaaaaaaaaaaa,
    bbbbbbbbbbbbbbbbbb,
    cccccccccccccccccc,
    dddddddddddddddddd,
    eeeeeeeeeeeeeeeeee,
    ffffffffffffffffff,
};
```

#### `"Vertical"`:

```rust
use foo::{
    xxx,
    yyy,
    zzz,
};

use foo::{
    aaa,
    bbb,
    ccc,
    ddd,
    eee,
    fff,
};
```

## `merge_imports`

Merge multiple imports into a single nested import.

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: No

#### `false` (default):

```rust
use foo::{a, c, d};
use foo::{b, g};
use foo::{e, f};
```

#### `true`:

```rust
use foo::{a, b, c, d, e, f, g};
```


## `match_block_trailing_comma`

Put a trailing comma after a block based match arm (non-block arms are not affected)

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: No

#### `false` (default):

```rust
fn main() {
    match lorem {
        Lorem::Ipsum => {
            println!("ipsum");
        }
        Lorem::Dolor => println!("dolor"),
    }
}
```

#### `true`:

```rust
fn main() {
    match lorem {
        Lorem::Ipsum => {
            println!("ipsum");
        },
        Lorem::Dolor => println!("dolor"),
    }
}
```

See also: [`trailing_comma`](#trailing_comma), [`match_arm_blocks`](#match_arm_blocks).

## `max_width`

Maximum width of each line

- **Default value**: `100`
- **Possible values**: any positive integer
- **Stable**: Yes

See also [`error_on_line_overflow`](#error_on_line_overflow).

## `merge_derives`

Merge multiple derives into a single one.

- **Default value**: `true`
- **Possible values**: `true`, `false`
- **Stable**: Yes

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

## `force_multiline_blocks`

Force multiline closure and match arm bodies to be wrapped in a block

- **Default value**: `false`
- **Possible values**: `false`, `true`
- **Stable**: No

#### `false` (default):

```rust
fn main() {
    result.and_then(|maybe_value| match maybe_value {
        None => foo(),
        Some(value) => bar(),
    });

    match lorem {
        None => |ipsum| {
            println!("Hello World");
        },
        Some(dolor) => foo(),
    }
}
```

#### `true`:

```rust
fn main() {
    result.and_then(|maybe_value| {
        match maybe_value {
            None => foo(),
            Some(value) => bar(),
        }
    });

    match lorem {
        None => {
            |ipsum| {
                println!("Hello World");
            }
        }
        Some(dolor) => foo(),
    }
}
```


## `newline_style`

Unix or Windows line endings

- **Default value**: `"Auto"`
- **Possible values**: `"Auto"`, `"Native"`, `"Unix"`, `"Windows"`
- **Stable**: Yes

#### `Auto` (default):

The newline style is detected automatically on a per-file basis. Files
with mixed line endings will be converted to the first detected line
ending style.

#### `Native`

Line endings will be converted to `\r\n` on Windows and `\n` on all
other platforms.

#### `Unix`

Line endings will be converted to `\n`.

#### `Windows`

Line endings will be converted to `\r\n`.

## `normalize_comments`

Convert /* */ comments to // comments where possible

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: No

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

## `remove_nested_parens`

Remove nested parens.

- **Default value**: `true`,
- **Possible values**: `true`, `false`
- **Stable**: Yes


#### `true` (default):
```rust
fn main() {
    (foo());
}
```

#### `false`:
```rust
fn main() {
    ((((foo()))));
}
```


## `reorder_imports`

Reorder import and extern crate statements alphabetically in groups (a group is
separated by a newline).

- **Default value**: `true`
- **Possible values**: `true`, `false`
- **Stable**: Yes

#### `true` (default):

```rust
use dolor;
use ipsum;
use lorem;
use sit;
```

#### `false`:

```rust
use lorem;
use ipsum;
use dolor;
use sit;
```


## `reorder_modules`

Reorder `mod` declarations alphabetically in group.

- **Default value**: `true`
- **Possible values**: `true`, `false`
- **Stable**: Yes

#### `true` (default)

```rust
mod a;
mod b;

mod dolor;
mod ipsum;
mod lorem;
mod sit;
```

#### `false`

```rust
mod b;
mod a;

mod lorem;
mod ipsum;
mod dolor;
mod sit;
```

**Note** `mod` with `#[macro_export]` will not be reordered since that could change the semantics
of the original source code.

## `reorder_impl_items`

Reorder impl items. `type` and `const` are put first, then macros and methods.

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: No

#### `false` (default)

```rust
struct Dummy;

impl Iterator for Dummy {
    fn next(&mut self) -> Option<Self::Item> {
        None
    }

    type Item = i32;
}
```

#### `true`

```rust
struct Dummy;

impl Iterator for Dummy {
    type Item = i32;

    fn next(&mut self) -> Option<Self::Item> {
        None
    }
}
```

## `report_todo`

Report `TODO` items in comments.

- **Default value**: `"Never"`
- **Possible values**: `"Always"`, `"Unnumbered"`, `"Never"`
- **Stable**: No

Warns about any comments containing `TODO` in them when set to `"Always"`. If
it contains a `#X` (with `X` being a number) in parentheses following the
`TODO`, `"Unnumbered"` will ignore it.

See also [`report_fixme`](#report_fixme).

## `report_fixme`

Report `FIXME` items in comments.

- **Default value**: `"Never"`
- **Possible values**: `"Always"`, `"Unnumbered"`, `"Never"`
- **Stable**: No

Warns about any comments containing `FIXME` in them when set to `"Always"`. If
it contains a `#X` (with `X` being a number) in parentheses following the
`FIXME`, `"Unnumbered"` will ignore it.

See also [`report_todo`](#report_todo).


## `skip_children`

Don't reformat out of line modules

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: No

## `space_after_colon`

Leave a space after the colon.

- **Default value**: `true`
- **Possible values**: `true`, `false`
- **Stable**: No

#### `true` (default):

```rust
fn lorem<T: Eq>(t: T) {
    let lorem: Dolor = Lorem {
        ipsum: dolor,
        sit: amet,
    };
}
```

#### `false`:

```rust
fn lorem<T:Eq>(t:T) {
    let lorem:Dolor = Lorem {
        ipsum:dolor,
        sit:amet,
    };
}
```

See also: [`space_before_colon`](#space_before_colon).

## `space_before_colon`

Leave a space before the colon.

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: No

#### `false` (default):

```rust
fn lorem<T: Eq>(t: T) {
    let lorem: Dolor = Lorem {
        ipsum: dolor,
        sit: amet,
    };
}
```

#### `true`:

```rust
fn lorem<T : Eq>(t : T) {
    let lorem : Dolor = Lorem {
        ipsum : dolor,
        sit : amet,
    };
}
```

See also: [`space_after_colon`](#space_after_colon).

## `struct_field_align_threshold`

The maximum diff of width between struct fields to be aligned with each other.

- **Default value** : 0
- **Possible values**: any positive integer
- **Stable**: No

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

## `spaces_around_ranges`

Put spaces around the .., ..=, and ... range operators

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: No

#### `false` (default):

```rust
fn main() {
    let lorem = 0..10;
    let ipsum = 0..=10;

    match lorem {
        1..5 => foo(),
        _ => bar,
    }

    match lorem {
        1..=5 => foo(),
        _ => bar,
    }

    match lorem {
        1...5 => foo(),
        _ => bar,
    }
}
```

#### `true`:

```rust
fn main() {
    let lorem = 0 .. 10;
    let ipsum = 0 ..= 10;

    match lorem {
        1 .. 5 => foo(),
        _ => bar,
    }

    match lorem {
        1 ..= 5 => foo(),
        _ => bar,
    }

    match lorem {
        1 ... 5 => foo(),
        _ => bar,
    }
}
```

## `struct_lit_single_line`

Put small struct literals on a single line

- **Default value**: `true`
- **Possible values**: `true`, `false`
- **Stable**: No

#### `true` (default):

```rust
fn main() {
    let lorem = Lorem { foo: bar, baz: ofo };
}
```

#### `false`:

```rust
fn main() {
    let lorem = Lorem {
        foo: bar,
        baz: ofo,
    };
}
```

See also: [`indent_style`](#indent_style).


## `tab_spaces`

Number of spaces per tab

- **Default value**: `4`
- **Possible values**: any positive integer
- **Stable**: Yes

#### `4` (default):

```rust
fn lorem() {
    let ipsum = dolor();
    let sit = vec![
        "amet consectetur adipiscing elit amet",
        "consectetur adipiscing elit amet consectetur.",
    ];
}
```

#### `2`:

```rust
fn lorem() {
  let ipsum = dolor();
  let sit = vec![
    "amet consectetur adipiscing elit amet",
    "consectetur adipiscing elit amet consectetur.",
  ];
}
```

See also: [`hard_tabs`](#hard_tabs).


## `trailing_comma`

How to handle trailing commas for lists

- **Default value**: `"Vertical"`
- **Possible values**: `"Always"`, `"Never"`, `"Vertical"`
- **Stable**: No

#### `"Vertical"` (default):

```rust
fn main() {
    let Lorem { ipsum, dolor, sit } = amet;
    let Lorem {
        ipsum,
        dolor,
        sit,
        amet,
        consectetur,
        adipiscing,
    } = elit;
}
```

#### `"Always"`:

```rust
fn main() {
    let Lorem { ipsum, dolor, sit, } = amet;
    let Lorem {
        ipsum,
        dolor,
        sit,
        amet,
        consectetur,
        adipiscing,
    } = elit;
}
```

#### `"Never"`:

```rust
fn main() {
    let Lorem { ipsum, dolor, sit } = amet;
    let Lorem {
        ipsum,
        dolor,
        sit,
        amet,
        consectetur,
        adipiscing
    } = elit;
}
```

See also: [`match_block_trailing_comma`](#match_block_trailing_comma).

## `trailing_semicolon`

Add trailing semicolon after break, continue and return

- **Default value**: `true`
- **Possible values**: `true`, `false`
- **Stable**: No

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
- **Stable**: No

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

## `use_field_init_shorthand`

Use field initialize shorthand if possible.

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: Yes

#### `false` (default):

```rust
struct Foo {
    x: u32,
    y: u32,
    z: u32,
}

fn main() {
    let x = 1;
    let y = 2;
    let z = 3;
    let a = Foo { x: x, y: y, z: z };
}
```

#### `true`:

```rust
struct Foo {
    x: u32,
    y: u32,
    z: u32,
}

fn main() {
    let x = 1;
    let y = 2;
    let z = 3;
    let a = Foo { x, y, z };
}
```

## `use_try_shorthand`

Replace uses of the try! macro by the ? shorthand

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: Yes

#### `false` (default):

```rust
fn main() {
    let lorem = try!(ipsum.map(|dolor| dolor.sit()));
}
```

#### `true`:

```rust
fn main() {
    let lorem = ipsum.map(|dolor| dolor.sit())?;
}
```

## `format_doc_comments`

Format doc comments.

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: No

#### `false` (default):

```rust
/// Adds one to the number given.
///
/// # Examples
///
/// ```rust
/// let five=5;
///
/// assert_eq!(
///     6,
///     add_one(5)
/// );
/// # fn add_one(x: i32) -> i32 {
/// #     x + 1
/// # }
/// ```
fn add_one(x: i32) -> i32 {
    x + 1
}
```

#### `true`

```rust
/// Adds one to the number given.
///
/// # Examples
///
/// ```rust
/// let five = 5;
///
/// assert_eq!(6, add_one(5));
/// # fn add_one(x: i32) -> i32 {
/// #     x + 1
/// # }
/// ```
fn add_one(x: i32) -> i32 {
    x + 1
}
```

## `wrap_comments`

Break comments to fit on the line

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: No

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

## `match_arm_blocks`

Wrap the body of arms in blocks when it does not fit on the same line with the pattern of arms

- **Default value**: `true`
- **Possible values**: `true`, `false`
- **Stable**: No

#### `true` (default):

```rust
fn main() {
    match lorem {
        true => {
            foooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo(x)
        }
        false => println!("{}", sit),
    }
}
```

#### `false`:

```rust
fn main() {
    match lorem {
        true =>
            foooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooo(x),
        false => println!("{}", sit),
    }
}
```

See also: [`match_block_trailing_comma`](#match_block_trailing_comma).

## `overflow_delimited_expr`

When structs, slices, arrays, and block/array-like macros are used as the last
argument in an expression list, allow them to overflow (like blocks/closures)
instead of being indented on a new line.

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: No

#### `false` (default):

```rust
fn example() {
    foo(ctx, |param| {
        action();
        foo(param)
    });

    foo(
        ctx,
        Bar {
            x: value,
            y: value2,
        },
    );

    foo(
        ctx,
        &[
            MAROON_TOMATOES,
            PURPLE_POTATOES,
            ORGANE_ORANGES,
            GREEN_PEARS,
            RED_APPLES,
        ],
    );

    foo(
        ctx,
        vec![
            MAROON_TOMATOES,
            PURPLE_POTATOES,
            ORGANE_ORANGES,
            GREEN_PEARS,
            RED_APPLES,
        ],
    );
}
```

#### `true`:

```rust
fn example() {
    foo(ctx, |param| {
        action();
        foo(param)
    });

    foo(ctx, Bar {
        x: value,
        y: value2,
    });

    foo(ctx, &[
        MAROON_TOMATOES,
        PURPLE_POTATOES,
        ORGANE_ORANGES,
        GREEN_PEARS,
        RED_APPLES,
    ]);

    foo(ctx, vec![
        MAROON_TOMATOES,
        PURPLE_POTATOES,
        ORGANE_ORANGES,
        GREEN_PEARS,
        RED_APPLES,
    ]);
}
```

## `blank_lines_upper_bound`

Maximum number of blank lines which can be put between items. If more than this number of consecutive empty
lines are found, they are trimmed down to match this integer.

- **Default value**: `1`
- **Possible values**: *unsigned integer*
- **Stable**: No

### Example
Original Code:

```rust
#![rustfmt::skip]

fn foo() {
    println!("a");
}



fn bar() {
    println!("b");


    println!("c");
}
```

#### `1` (default):
```rust
fn foo() {
    println!("a");
}

fn bar() {
    println!("b");

    println!("c");
}
```

#### `2`:
```rust
fn foo() {
    println!("a");
}


fn bar() {
    println!("b");


    println!("c");
}
```

See also: [`blank_lines_lower_bound`](#blank_lines_lower_bound)

## `blank_lines_lower_bound`

Minimum number of blank lines which must be put between items. If two items have fewer blank lines between
them, additional blank lines are inserted.

- **Default value**: `0`
- **Possible values**: *unsigned integer*
- **Stable**: No

### Example
Original Code (rustfmt will not change it with the default value of `0`):

```rust
#![rustfmt::skip]

fn foo() {
    println!("a");
}
fn bar() {
    println!("b");
    println!("c");
}
```

#### `1`
```rust
fn foo() {

    println!("a");
}

fn bar() {

    println!("b");

    println!("c");
}
```


## `required_version`

Require a specific version of rustfmt. If you want to make sure that the
specific version of rustfmt is used in your CI, use this option.

- **Default value**: `CARGO_PKG_VERSION`
- **Possible values**: any published version (e.g. `"0.3.8"`)
- **Stable**: No

## `hide_parse_errors`

Do not show parse errors if the parser failed to parse files.

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: No

## `color`

Whether to use colored output or not.

- **Default value**: `"Auto"`
- **Possible values**: "Auto", "Always", "Never"
- **Stable**: No

## `unstable_features`

Enable unstable features on stable channel.

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: No

## `license_template_path`

Check whether beginnings of files match a license template.

- **Default value**: `""``
- **Possible values**: path to a license template file
- **Stable**: No

A license template is a plain text file which is matched literally against the
beginning of each source file, except for `{}`-delimited blocks, which are
matched as regular expressions. The following license template therefore
matches strings like `// Copyright 2017 The Rust Project Developers.`, `//
Copyright 2018 The Rust Project Developers.`, etc.:

```
// Copyright {\d+} The Rust Project Developers.
```

`\{`, `\}` and `\\` match literal braces / backslashes.

## `ignore`

Skip formatting the specified files and directories.

- **Default value**: format every files
- **Possible values**: See an example below
- **Stable**: No

### Example

If you want to ignore specific files, put the following to your config file:

```toml
ignore = [
    "src/types.rs",
    "src/foo/bar.rs",
]
```

If you want to ignore every file under `examples/`, put the following to your config file:

```toml
ignore = [
    "examples",
]
```

## `edition`

Specifies which edition is used by the parser.

- **Default value**: `2015`
- **Possible values**: `2015`, `2018`
- **Stable**: Yes

### Example

If you want to format code that requires edition 2018, add the following to your config file:

```toml
edition = "2018"
```

## `version`

Which version of the formatting rules to use. `Version::One` is backwards-compatible
with Rustfmt 1.0. Other versions are only backwards compatible within a major
version number.

- **Default value**: `One`
- **Possible values**: `One`, `Two`
- **Stable**: No

### Example

```toml
version = "Two"
```

## `normalize_doc_attributes`

Convert `#![doc]` and `#[doc]` attributes to `//!` and `///` doc comments.

- **Default value**: `false`
- **Possible values**: `true`, `false`
- **Stable**: No

#### `false` (default):

```rust
#![doc = "Example documentation"]

#[doc = "Example item documentation"]
pub enum Foo {}
```

#### `true`:

```rust
//! Example documentation

/// Example item documentation
pub enum Foo {}
```

## `emit_mode`

Internal option

## `make_backup`

Internal option, use `--backup`
