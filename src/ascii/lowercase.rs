#[cfg(feature = "alloc")]
use alloc::vec::Vec;

/// Converts the given slice to its ASCII lower case equivalent in-place.
///
/// ASCII letters 'A' to 'Z' are mapped to 'a' to 'z', but non-ASCII letters are
/// unchanged.
///
/// This function can be used to implement [`String#downcase!`] for ASCII
/// strings in Ruby.
///
/// To return a new lowercased value without modifying the existing one, use
/// [`to_ascii_lowercase`].
///
/// See also [`<[u8]>::make_ascii_lowercase`][slice-primitive].
///
/// # Examples
///
/// ```
/// # use roe::make_ascii_lowercase;
/// let mut buf = *b"ABCxyz";
/// make_ascii_lowercase(&mut buf);
/// assert_eq!(buf, *b"abcxyz");
///
/// let mut buf = *b"1234%&*";
/// make_ascii_lowercase(&mut buf);
/// assert_eq!(buf, *b"1234%&*");
/// ```
///
/// [`String#downcase!`]: https://ruby-doc.org/core-2.6.3/String.html#method-i-downcase-21
/// [slice-primitive]: https://doc.rust-lang.org/std/primitive.slice.html#method.make_ascii_lowercase
#[inline]
#[allow(clippy::module_name_repetitions)]
pub fn make_ascii_lowercase<T: AsMut<[u8]>>(slice: &mut T) {
    let slice = slice.as_mut();
    slice.make_ascii_lowercase();
}

/// Returns a vector containing a copy of the given slice where each byte is
/// mapped to its ASCII lower case equivalent.
///
/// ASCII letters 'A' to 'Z' are mapped to 'a' to 'z', but non-ASCII letters are
/// unchanged.
///
/// This function can be used to implement [`String#downcase`] and
/// [`Symbol#downcase`] for ASCII strings in Ruby.
///
/// To lowercase the value in-place, use [`make_ascii_lowercase`].
///
/// See also [`<[u8]>::to_ascii_lowercase`][slice-primitive].
///
/// # Examples
///
/// ```
/// # use roe::to_ascii_lowercase;
/// assert_eq!(to_ascii_lowercase("ABCxyz"), &b"abcxyz"[..]);
/// assert_eq!(to_ascii_lowercase("1234%&*"), &b"1234%&*"[..]);
/// ```
///
/// [`String#downcase`]: https://ruby-doc.org/core-2.6.3/String.html#method-i-downcase
/// [`Symbol#downcase`]: https://ruby-doc.org/core-2.6.3/Symbol.html#method-i-downcase
/// [slice-primitive]: https://doc.rust-lang.org/std/primitive.slice.html#method.to_ascii_lowercase
#[inline]
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
#[allow(clippy::module_name_repetitions)]
pub fn to_ascii_lowercase<T: AsRef<[u8]>>(slice: T) -> Vec<u8> {
    let slice = slice.as_ref();
    slice.to_ascii_lowercase()
}

#[cfg(test)]
mod tests {
    #[test]
    fn make_ascii_lowercase_empty() {
        let mut buf = *b"";
        super::make_ascii_lowercase(&mut buf);
        assert_eq!(buf, *b"");
    }

    #[test]
    #[cfg(feature = "alloc")]
    fn to_ascii_lowercase_empty() {
        assert_eq!(super::to_ascii_lowercase(""), b"");
    }
}
