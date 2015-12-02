
type PrivateTest<'a, I> = (Box<Parser<Input = I, Output = char> + 'a>,
                           Box<Parser<Input = I, Output = char> + 'a>);

pub type PublicTest<'a, I, O> = Result<Vec<MyLongType>,
                                       Box<Parser<Input = I, Output = char> + 'a>,
                                       Box<Parser<Input = I, Output = char> + 'a>>;

pub type LongGenericListTest<'a,
                             'b,
                             'c,
                             'd,
                             LONGPARAMETERNAME,
                             LONGPARAMETERNAME,
                             LONGPARAMETERNAME,
                             A,
                             B,
                             C> = Option<Vec<MyType>>;

pub type Exactly100CharsTest<'a, 'b, 'c, 'd, LONGPARAMETERNAME, LONGPARAMETERNAME, A, B> = Vec<i32>;

pub type Exactly101CharsTest<'a, 'b, 'c, 'd, LONGPARAMETERNAME, LONGPARAMETERNAME, A, B> =
    Vec<Test>;

pub type Exactly100CharsToEqualTest<'a, 'b, 'c, 'd, LONGPARAMETERNAME, LONGPARAMETERNAME, A, B, C> =
    Vec<i32>;

pub type GenericsFitButNotEqualTest<'a,
                                    'b,
                                    'c,
                                    'd,
                                    LONGPARAMETERNAME,
                                    LONGPARAMETERNAME,
                                    A1,
                                    B,
                                    C> = Vec<i32>;

pub type CommentTest<// Lifetime
                     'a, // Type
                     T> = ();


pub type WithWhereClause<LONGPARAMETERNAME, T>
    where T: Clone,
          LONGPARAMETERNAME: Clone + Eq + OtherTrait = Option<T>;

pub type Exactly100CharstoEqualWhereTest<T, U, PARAMET> where T: Clone + Ord + Eq + SomeOtherTrait =
    Option<T>;

pub type Exactly101CharstoEqualWhereTest<T, U, PARAMETE>
    where T: Clone + Ord + Eq + SomeOtherTrait = Option<T>;
