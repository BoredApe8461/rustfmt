// rustfmt-normalize_comments: true

extern crate bar;
extern crate chrono;
extern crate dotenv;
extern crate foo;
extern crate foo;
extern crate foo as bar;
extern crate futures;
extern crate proc_macro;
// #2315
extern crate proc_macro2;

extern "C" {
    fn c_func(x: *mut *mut libc::c_void);

    fn c_func(
        x: XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX,
        y: YYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYYY,
    );

    #[test123]
    fn foo() -> uint64_t;

    pub fn bar();
}

extern "C" {
    fn DMR_GetDevice(
        pHDev: *mut HDEV,
        searchMode: DeviceSearchMode,
        pSearchString: *const c_char,
        devNr: c_uint,
        wildcard: c_char,
    ) -> TDMR_ERROR;

    fn quux() -> (); // Post comment

    pub type Foo;

    type Bar;
}

extern "Rust" {
    static ext: u32;
    // Some comment.
    pub static mut var: SomeType;
}

extern "C" {
    fn syscall(
        number: libc::c_long, // comment 1
        // comm 2
        ... // sup?
    ) -> libc::c_long;

    fn foo(x: *const c_char, ...) -> libc::c_long;
}

extern "C" {
    pub fn freopen(
        filename: *const c_char,
        mode: *const c_char,
        mode2: *const c_char,
        mode3: *const c_char,
        file: *mut FILE,
    ) -> *mut FILE;
}

extern "C" {}
