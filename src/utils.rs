use ::std::fmt::{Write, Display};
use ::std::borrow::Borrow;

pub fn comma_delimited<T: Display + ?Sized, B: Borrow<T>, I: IntoIterator<Item=B>>(iter: I) -> String {
    let mut iter = iter.into_iter();
    let mut ret = String::new();
    if let Some(b) = iter.next() {
        write!(ret, "{}", b.borrow()).unwrap();
        for b in iter {
            write!(ret, ",{}", b.borrow()).unwrap();
        }
    }
    ret
}

