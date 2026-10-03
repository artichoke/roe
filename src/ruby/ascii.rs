#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Converts the given slice to its ASCII capitalization equivalent in-place.
///
/// ASCII letters 'a' to 'z' are mapped to 'A' to 'Z' in the first byte;
/// subsequent bytes with ASCII letters 'A' to 'Z' are mapped to 'a' to 'z';
/// non-ASCII letters are unchanged.
///
/// This function can be used to implement [`String#capitalize!`] for ASCII
/// strings in Ruby.
///
#[cfg_attr(
    feature = "alloc",
    doc = "To return a new capitalized value without modifying the existing one, use [`to_ascii_capitalize`]."
)]
///
/// # Examples
///
/// ```
/// # use roe::ruby::make_ascii_capitalize;
/// let mut buf = *b"ABCxyz";
/// make_ascii_capitalize(&mut buf);
/// assert_eq!(buf, *b"Abcxyz");
///
/// let mut buf = *b"1234%&*";
/// make_ascii_capitalize(&mut buf);
/// assert_eq!(buf, *b"1234%&*");
///
/// let mut buf = *b"ABC1234%&*";
/// make_ascii_capitalize(&mut buf);
/// assert_eq!(buf, *b"Abc1234%&*");
///
/// let mut buf = *b"1234%&*abcXYZ";
/// make_ascii_capitalize(&mut buf);
/// assert_eq!(buf, *b"1234%&*abcxyz");
///
/// let mut buf = *b"ABC, XYZ";
/// make_ascii_capitalize(&mut buf);
/// assert_eq!(buf, *b"Abc, xyz");
/// ```
///
/// [`String#capitalize!`]: https://docs.ruby-lang.org/en/4.0/String.html#method-i-capitalize-21
#[inline]
#[allow(clippy::module_name_repetitions)]
pub fn make_ascii_capitalize<T: AsMut<[u8]>>(slice: &mut T) {
    let slice = slice.as_mut();
    if let Some((head, tail)) = slice.split_first_mut() {
        head.make_ascii_uppercase();
        tail.make_ascii_lowercase();
    }
}

/// Returns a vector containing a copy of the given slice where each byte is
/// mapped to its ASCII capitalization equivalent.
///
/// ASCII letters 'a' to 'z' are mapped to 'A' to 'Z' in the first byte;
/// subsequent bytes with ASCII letters 'A' to 'Z' are mapped to 'a' to 'z';
/// non-ASCII letters are unchanged.
///
/// This function can be used to implement [`String#capitalize`] and
/// [`Symbol#capitalize`] for ASCII strings in Ruby.
///
/// To capitalize the value in-place, use [`make_ascii_capitalize`].
///
/// # Examples
///
/// ```
/// # use roe::ruby::to_ascii_capitalize;
/// assert_eq!(to_ascii_capitalize("ABCxyz"), &b"Abcxyz"[..]);
/// assert_eq!(to_ascii_capitalize("1234%&*"), &b"1234%&*"[..]);
/// assert_eq!(to_ascii_capitalize("ABC1234%&*"), &b"Abc1234%&*"[..]);
/// assert_eq!(to_ascii_capitalize("1234%&*abcXYZ"), &b"1234%&*abcxyz"[..]);
/// assert_eq!(to_ascii_capitalize("ABC, XYZ"), &b"Abc, xyz"[..]);
/// ```
///
/// [`String#capitalize`]: https://docs.ruby-lang.org/en/4.0/String.html#method-i-capitalize
/// [`Symbol#capitalize`]: https://docs.ruby-lang.org/en/4.0/Symbol.html#method-i-capitalize
#[inline]
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[allow(clippy::module_name_repetitions)]
pub fn to_ascii_capitalize<T: AsRef<[u8]>>(slice: T) -> Vec<u8> {
    let slice = slice.as_ref();
    let mut capitalized = slice.to_ascii_lowercase();
    if let Some(head) = capitalized.first_mut() {
        head.make_ascii_uppercase();
    }
    capitalized
}

#[cfg(test)]
mod tests {
    #[test]
    fn make_ascii_capitalize_empty() {
        let mut buf = *b"";
        super::make_ascii_capitalize(&mut buf);
        assert_eq!(buf, *b"");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn to_ascii_capitalize_empty() {
        assert_eq!(super::to_ascii_capitalize(""), b"");
    }
}
