// rustfmt-where_pred_indent: Tabbed
// Test different indents.

fn qux()
    where X: TTTTTTTTTTTTTTTTTTTTTTTTTTTT,
        X: TTTTTTTTTTTTTTTTTTTTTTTTTTTT,
        X: TTTTTTTTTTTTTTTTTTTTTTTTTTTT,
        X: TTTTTTTTTTTTTTTTTTTTTTTTTTTT
{
    baz();
}
