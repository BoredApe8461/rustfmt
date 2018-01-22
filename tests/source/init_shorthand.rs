// Use field initialization shorthand if possible.

fn main() {
    let a = Foo {
        x: x,
        y: y,
        z: z,
    };

    let b = Bar {
        x: x,
        y: y,
        #[attr]
        z: z,
        #[rustfmt_skip]
        skipped: skipped,
    };
}
