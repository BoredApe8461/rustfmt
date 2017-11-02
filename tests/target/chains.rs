// rustfmt-normalize_comments: true
// rustfmt-single_line_if_else_max_width: 0
// rustfmt-chain_one_line_max: 100
// Test chain formatting.

fn main() {
    let a = b.c.d.1.foo(|x| x + 1);

    bbbbbbbbbbbbbbbbbbb.ccccccccccccccccccccccccccccccccccccc.ddddddddddddddddddddddddddd();

    bbbbbbbbbbbbbbbbbbb
        .ccccccccccccccccccccccccccccccccccccc
        .ddddddddddddddddddddddddddd
        .eeeeeeee();

    let f = fooooooooooooooooooooooooooooooooooooooooooooooooooo
        .baaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaar;

    // Test case where first chain element isn't a path, but is shorter than
    // the size of a tab.
    x().y(|| match cond() {
        true => (),
        false => (),
    });

    loong_func().quux(move || {
        if true {
            1
        } else {
            2
        }
    });

    some_fuuuuuuuuunction().method_call_a(aaaaa, bbbbb, |c| {
        let x = c;
        x
    });

    some_fuuuuuuuuunction()
        .method_call_a(aaaaa, bbbbb, |c| {
            let x = c;
            x
        })
        .method_call_b(aaaaa, bbbbb, |c| {
            let x = c;
            x
        });

    fffffffffffffffffffffffffffffffffff(a, {
        SCRIPT_TASK_ROOT.with(|root| {
            *root.borrow_mut() = Some(&script_task);
        });
    });

    let suuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuuum =
        xxxxxxx.map(|x| x + 5).map(|x| x / 2).fold(0, |acc, x| acc + x);

    aaaaaaaaaaaaaaaa
        .map(|x| {
            x += 1;
            x
        })
        .filter(some_mod::some_filter)
}

fn floaters() {
    let z = Foo {
        field1: val1,
        field2: val2,
    };

    let x = Foo {
        field1: val1,
        field2: val2,
    }.method_call()
        .method_call();

    let y = if cond {
        val1
    } else {
        val2
    }.method_call();

    {
        match x {
            PushParam => {
                // params are 1-indexed
                stack.push(
                    mparams[match cur.to_digit(10) {
                                Some(d) => d as usize - 1,
                                None => return Err("bad param number".to_owned()),
                            }].clone(),
                );
            }
        }
    }

    if cond {
        some();
    } else {
        none();
    }.bar()
        .baz();

    Foo { x: val }
        .baz(|| {
            force();
            multiline();
        })
        .quux();

    Foo {
        y: i_am_multi_line,
        z: ok,
    }.baz(|| {
        force();
        multiline();
    })
        .quux();

    a + match x {
        true => "yay!",
        false => "boo!",
    }.bar()
}

fn is_replaced_content() -> bool {
    constellat.send(ConstellationMsg::ViewportConstrained(self.id, constraints)).unwrap();
}

fn issue587() {
    a.b::<()>(c);

    std::mem::transmute(dl.symbol::<()>("init").unwrap())
}

fn try_shorthand() {
    let x = expr?;
    let y = expr.kaas()?.test();
    let loooooooooooooooooooooooooooooooooooooooooong =
        does_this?.look?.good?.should_we_break?.after_the_first_question_mark?;
    let yyyy = expr?.another?.another?.another?.another?.another?.another?.another?.another?.test();
    let zzzz = expr?.another?.another?.another?.another?;
    let aaa = x??????????????????????????????????????????????????????????????????????????;

    let y = a.very
        .loooooooooooooooooooooooooooooooooooooong()
        .chain()
        .inside()
        .weeeeeeeeeeeeeee()?
        .test()
        .0
        .x;

    parameterized(f, substs, def_id, Ns::Value, &[], |tcx| {
        tcx.lookup_item_type(def_id).generics
    })?;
    fooooooooooooooooooooooooooo()?
        .bar()?
        .baaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaz()?;
}

fn issue_1004() {
    match *self {
        ty::ImplOrTraitItem::MethodTraitItem(ref i) => write!(f, "{:?}", i),
        ty::ImplOrTraitItem::ConstTraitItem(ref i) => write!(f, "{:?}", i),
        ty::ImplOrTraitItem::TypeTraitItem(ref i) => write!(f, "{:?}", i),
    }?;

    ty::tls::with(|tcx| {
        let tap = ty::Binder(TraitAndProjections(principal, projections));
        in_binder(f, tcx, &ty::Binder(""), Some(tap))
    })?;
}

fn issue1392() {
    test_method(
        r#"
        if foo {
            a();
        }
        else {
            b();
        }
        "#.trim(),
    );
}

// #2067
impl Settings {
    fn save(&self) -> Result<()> {
        let mut file = File::create(&settings_path)
            .chain_err(|| ErrorKind::WriteError(settings_path.clone()))?;
    }
}
