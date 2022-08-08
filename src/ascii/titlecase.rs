#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Converts the given slice to its ASCII title case equivalent in-place.
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
    doc = "To return a new titlecased value without modifying the existing one, use [`to_ascii_titlecase`]."
)]
///
/// # Examples
///
/// ```
/// # use roe::make_ascii_titlecase;
/// let mut buf = *b"ABCxyz";
/// make_ascii_titlecase(&mut buf);
/// assert_eq!(buf, *b"Abcxyz");
///
/// let mut buf = *b"1234%&*";
/// make_ascii_titlecase(&mut buf);
/// assert_eq!(buf, *b"1234%&*");
///
/// let mut buf = *b"ABC1234%&*";
/// make_ascii_titlecase(&mut buf);
/// assert_eq!(buf, *b"Abc1234%&*");
///
/// let mut buf = *b"1234%&*abcXYZ";
/// make_ascii_titlecase(&mut buf);
/// assert_eq!(buf, *b"1234%&*abcxyz");
///
/// let mut buf = *b"ABC, XYZ";
/// make_ascii_titlecase(&mut buf);
/// assert_eq!(buf, *b"Abc, xyz");
/// ```
///
/// [`String#capitalize!`]: https://ruby-doc.org/core-3.1.2/String.html#method-i-capitalize-21
#[inline]
#[allow(clippy::module_name_repetitions)]
pub fn make_ascii_titlecase<T: AsMut<[u8]>>(slice: &mut T) {
    let slice = slice.as_mut();
    if let Some((head, tail)) = slice.split_first_mut() {
        head.make_ascii_uppercase();
        tail.make_ascii_lowercase();
    }
}

/// Returns a vector containing a copy of the given slice where each byte is
/// mapped to its ASCII title case equivalent.
///
/// ASCII letters 'a' to 'z' are mapped to 'A' to 'Z' in the first byte;
/// subsequent bytes with ASCII letters 'A' to 'Z' are mapped to 'a' to 'z';
/// non-ASCII letters are unchanged.
///
/// This function can be used to implement [`String#capitalize`] and
/// [`Symbol#capitalize`] for ASCII strings in Ruby.
///
/// To titlecase the value in-place, use [`make_ascii_titlecase`].
///
/// # Examples
///
/// ```
/// # use roe::to_ascii_titlecase;
/// assert_eq!(to_ascii_titlecase("ABCxyz"), &b"Abcxyz"[..]);
/// assert_eq!(to_ascii_titlecase("1234%&*"), &b"1234%&*"[..]);
/// assert_eq!(to_ascii_titlecase("ABC1234%&*"), &b"Abc1234%&*"[..]);
/// assert_eq!(to_ascii_titlecase("1234%&*abcXYZ"), &b"1234%&*abcxyz"[..]);
/// assert_eq!(to_ascii_titlecase("ABC, XYZ"), &b"Abc, xyz"[..]);
/// ```
///
/// [`String#capitalize`]: https://ruby-doc.org/core-3.1.2/String.html#method-i-capitalize
/// [`Symbol#capitalize`]: https://ruby-doc.org/core-3.1.2/Symbol.html#method-i-capitalize
#[inline]
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[allow(clippy::module_name_repetitions)]
pub fn to_ascii_titlecase<T: AsRef<[u8]>>(slice: T) -> Vec<u8> {
    let slice = slice.as_ref();
    let mut titlecase = slice.to_ascii_lowercase();
    if let Some(head) = titlecase.first_mut() {
        head.make_ascii_uppercase();
    }
    titlecase
}

#[cfg(test)]
mod tests {
    #[test]
    fn make_ascii_titlecase_empty() {
        let mut buf = *b"";
        super::make_ascii_titlecase(&mut buf);
        assert_eq!(buf, *b"");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn to_ascii_titlecase_empty() {
        assert_eq!(super::to_ascii_titlecase(""), b"");
    }
}
