// rustfmt-normalize_comments: true
// rustfmt-fn_args_layout: Block
// rustfmt-fn_args_density: Vertical
// rustfmt-fn_arg_indent: Block
// rustfmt-fn_brace_style: AlwaysNextLine

// Case with only one variable.
fn foo(a: u8,) -> u8
{
    bar()
}

// Case with 2 variables and some pre-comments.
fn foo(
    a: u8, // Comment 1
    b: u8, // Comment 2
) -> u8
{
    bar()
}

// Case with 2 variables and some post-comments.
fn foo(
    // Comment 1
    a: u8,
    // Comment 2
    b: u8,
) -> u8
{
    bar()
}

trait Test {
    fn foo(a: u8,)
    {
    }

    fn bar(a: u8,) -> String
    {
    }
}
