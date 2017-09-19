// rustfmt-wrap_comments: true
// rustfmt-error_on_line_overflow: false
// Enums test

#[atrr]
pub enum Test {
    A,
    B(u32, A /* comment */, SomeType),
    /// Doc comment
    C,
}

pub enum Foo<'a, Y: Baz>
where
    X: Whatever,
{
    A,
}

enum EmtpyWithComment {
    // Some comment
}

// C-style enum
enum Bar {
    A = 1,
    #[someAttr(test)] B = 2, // comment
    C,
}

enum LongVariants {
    First(
        LOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOOONG, // comment
        VARIANT,
    ),
    // This is the second variant
    Second,
}

enum StructLikeVariants {
    Normal(u32, String),
    StructLike {
        x: i32, // Test comment
        // Pre-comment
        #[Attr50] y: SomeType, // Aanother Comment
    },
    SL { a: A },
}

enum X {
    CreateWebGLPaintTask(
        Size2D<i32>,
        GLContextAttributes,
        IpcSender<Result<(IpcSender<CanvasMsg>, usize), String>>,
    ), // This is a post comment
}

pub enum EnumWithAttributes {
    // This is a pre comment
    // AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
    TupleVar(usize, usize, usize), /* AAAA AAAAAAAAAAAAAAAAAAAAA AAAAAAAAAAAAAAAAAAAAAAAA
                                    * AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA */
    // Pre Comment
    #[rustfmt_skip]
    SkippedItem(String,String,), // Post-comment
    #[another_attr]
    #[attr2]
    ItemStruct { x: usize, y: usize }, /* Comment AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA */
    // And another
    ForcedPreflight, /* AAAAAAAAAAAAAAA AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                      * AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA */
}

pub enum SingleTuple {
    // Pre Comment AAAAAAAAAAAAAAAAAAAAAAA AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
    // AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
    Match(usize, usize, String), /* Post-comment AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA */
}

pub enum SingleStruct {
    Match { name: String, loc: usize }, // Post-comment
}

pub enum GenericEnum<I, T>
where
    I: Iterator<Item = T>,
{
    // Pre Comment
    Left { list: I, root: T },  // Post-comment
    Right { list: I, root: T }, // Post Comment
}


enum EmtpyWithComment {
    // Some comment
}

enum TestFormatFails {
    AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA,
}

fn nested_enum_test() {
    if true {
        enum TestEnum {
            One(
                usize,
                usize,
                usize,
                usize,
                usize,
                usize,
                usize,
                usize,
                usize,
                usize,
                usize,
                usize,
                usize,
                usize,
                usize,
                usize,
            ), /* AAAAAAAAAAAAAAA AAAAAAAAAAAAAAAAAAAAA AAAAAAAAAAAAAAAAAAAAAAAAAAA
                * AAAAAAAAAAAAAAAAAAAAAA */
            Two, /* AAAAAAAAAAAAAAAAAA  AAAAAAAAAAAAAAAAAAAAAA AAAAAAAAAAAAAAAAAAAAAAAAAAAAAA
                  * AAAAAAAAAAAAAAAAAA */
        }
        enum TestNestedFormatFail {
            AAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAAA,
        }
    }
}

pub struct EmtpyWithComment {
    // FIXME: Implement this struct
}

// #1115
pub enum Bencoding<'i> {
    Str(&'i [u8]),
    Int(i64),
    List(Vec<Bencoding<'i>>),
    /// A bencoded dict value. The first element the slice of bytes in the
    /// source that the dict is
    /// composed of. The second is the dict, decoded into an ordered map.
    // TODO make Dict "structlike" AKA name the two values.
    Dict(&'i [u8], BTreeMap<&'i [u8], Bencoding<'i>>),
}

// #1261
pub enum CoreResourceMsg {
    SetCookieForUrl(
        ServoUrl,
        #[serde(deserialize_with = "::hyper_serde::deserialize",
                serialize_with = "::hyper_serde::serialize")]
        Cookie,
        CookieSource,
    ),
}

enum Loooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong
{}
enum Looooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong
{}
enum Loooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong
{}
enum Loooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooooong
{
    Foo,
}

// #1046
pub enum Entry<'a, K: 'a, V: 'a> {
    Vacant(#[stable(feature = "rust1", since = "1.0.0")] VacantEntry<'a, K, V>),
    Occupied(#[stable(feature = "rust1", since = "1.0.0")] OccupiedEntry<'a, K, V>),
}
