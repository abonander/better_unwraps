use std::fmt::{self, Display, Formatter, Write};

#[derive(Copy, Clone, Debug)]
pub struct CallSite(pub &'static (&'static str, u32));

impl Display for CallSite {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "call site: {}:{}", (self.0).0, (self.0).1)
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! call_site {
    () => ({
        static CALL_SITE: (&'static str, u32) = (file!(), line!());
        $crate::CallSite(&CALL_SITE)
    })
}

pub trait UnwrapExt {
    type Unwrapped;
    fn unwrap_ext(self, call_site: CallSite) -> Self::Unwrapped;
}

pub trait UnwrapErrExt {
    type Unwrapped;
    fn unwrap_err_ext(self, call_site: CallSite) -> Self::Unwrapped;
}

impl<T> UnwrapExt for Option<T> {
    type Unwrapped = T;

    fn unwrap_ext(self, call_site: CallSite) -> Self::Unwrapped {
        match self {
            Some(val) => val,
            None => panic!("({}) called `Option::unwrap_ext()` on a `None` value",
                           call_site)
        }
    }
}

impl<T, E> UnwrapExt for Result<T, E> where E: Display {
    type Unwrapped = T;

    fn unwrap_ext(self, call_site: CallSite) -> Self::Unwrapped {
        match self {
            Ok(val) => val,
            Err(e) => panic!("({}) called `Result::unwrap_ext()` on an `Err` value: {}",
                             call_site, e),
        }
    }
}

impl<T, E> UnwrapErrExt for Result<T, E> where T: Display {
    type Unwrapped = E;

    fn unwrap_err_ext(self, call_site: CallSite) -> Self::Unwrapped {
        match self {
            Ok(val) => panic!("({}) called `Result::unwrap_err_ext()` on an `Ok` value: {}",
                              call_site, val),
            Err(e) => e,
        }
    }
}
