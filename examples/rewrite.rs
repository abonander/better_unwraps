#![feature(proc_macro)]

// FIXME https://github.com/rust-lang/rust/issues/41211
// #![rewrite_unwraps(inject)]

// This is not necessary if you're using `#![rewrite_unwraps(inject)]` at the crate root.
// However, attribute proc macro invocations at the crate root currently ICE, see previous FIXME.
#[macro_use] extern crate better_unwraps;

extern crate rewrite_unwraps;

// Procedural macros are imported like regular items
use rewrite_unwraps::rewrite_unwraps;

fn main() {
    foo::foo();
}

// Rewrites all unwraps to supply the file and line of the call site
// However, generated expansions of `line!()` point to this attribute instead
// of the proper line, so this doesn't quite work yet.
#[rewrite_unwraps]
mod foo {
    pub fn foo() {
        let val: Option<i32> = None;
        val.unwrap();
    }
}
