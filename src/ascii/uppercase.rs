#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Converts the given slice to its ASCII upper case equivalent in-place.
///
/// ASCII letters 'a' to 'z' are mapped to 'A' to 'Z', but non-ASCII letters are
/// unchanged.
///
/// This function can be used to implement [`String#upcase!`] for ASCII strings
/// in Ruby.
///
/// To return a new uppercased value without modifying the existing one, use
/// [`to_ascii_uppercase`].
///
/// See also [`<[u8]>::make_ascii_uppercase`][slice-primitive].
///
/// # Examples
///
/// ```
/// # use roe::make_ascii_uppercase;
/// let mut buf = *b"ABCxyz";
/// make_ascii_uppercase(&mut buf);
/// assert_eq!(buf, *b"ABCXYZ");
///
/// let mut buf = *b"1234%&*";
/// make_ascii_uppercase(&mut buf);
/// assert_eq!(buf, *b"1234%&*");
/// ```
///
/// [`String#upcase!`]: https://ruby-doc.org/core-2.6.3/String.html#method-i-upcase-21
/// [slice-primitive]: https://doc.rust-lang.org/std/primitive.u8.html#method.make_ascii_uppercase
#[inline]
#[allow(clippy::module_name_repetitions)]
pub fn make_ascii_uppercase<T: AsMut<[u8]>>(slice: &mut T) {
    let slice = slice.as_mut();
    slice.make_ascii_uppercase();
}

/// Returns a vector containing a copy of the given slice where each byte is
/// mapped to its ASCII upper case equivalent.
///
/// ASCII letters 'a' to 'z' are mapped to 'A' to 'Z', but non-ASCII letters are
/// unchanged.
///
/// This function can be used to implement [`String#upcase`] and
/// [`Symbol#upcase`] for ASCII strings in Ruby.
///
/// To uppercase the value in-place, use [`make_ascii_uppercase`].
///
/// See also [`<[u8]>::to_ascii_uppercase`][slice-primitive].
///
/// # Examples
///
/// ```
/// # use roe::to_ascii_uppercase;
/// assert_eq!(to_ascii_uppercase("ABCxyz"), &b"ABCXYZ"[..]);
/// assert_eq!(to_ascii_uppercase("1234%&*"), &b"1234%&*"[..]);
/// ```
///
/// [`String#upcase`]: https://ruby-doc.org/core-2.6.3/String.html#method-i-upcase
/// [`Symbol#upcase`]: https://ruby-doc.org/core-2.6.3/Symbol.html#method-i-upcase
/// [slice-primitive]: https://doc.rust-lang.org/std/primitive.slice.html#method.to_ascii_uppercase
#[inline]
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[allow(clippy::module_name_repetitions)]
pub fn to_ascii_uppercase<T: AsRef<[u8]>>(slice: T) -> Vec<u8> {
    let slice = slice.as_ref();
    slice.to_ascii_uppercase()
}

#[cfg(test)]
mod tests {
    #[test]
    fn make_ascii_uppercase_empty() {
        let mut buf = *b"";
        super::make_ascii_uppercase(&mut buf);
        assert_eq!(buf, *b"");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn to_ascii_uppercase_empty() {
        assert_eq!(super::to_ascii_uppercase(""), b"");
    }
}
