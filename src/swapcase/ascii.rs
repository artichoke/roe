#[cfg(feature = "alloc")]
use alloc::vec::Vec;

pub(crate) const fn swapcase_byte(byte: u8) -> u8 {
    if byte.is_ascii_lowercase() {
        byte.to_ascii_uppercase()
    } else {
        byte.to_ascii_lowercase()
    }
}

/// Swap the case of ASCII letters in-place, preserving all other bytes.
///
/// ```
/// let mut bytes = *b"Hello, Ruby!";
/// roe::make_ascii_swapcase(&mut bytes);
/// assert_eq!(bytes, *b"hELLO, rUBY!");
/// ```
pub fn make_ascii_swapcase<T: AsMut<[u8]>>(slice: &mut T) {
    for byte in slice.as_mut() {
        *byte = swapcase_byte(*byte);
    }
}

/// Return a copy with ASCII letters case-swapped and all other bytes preserved.
///
/// ```
/// assert_eq!(roe::to_ascii_swapcase(b"Hello, Ruby!"), b"hELLO, rUBY!");
/// ```
#[cfg(feature = "alloc")]
#[cfg_attr(docsrs, doc(cfg(feature = "alloc")))]
pub fn to_ascii_swapcase<T: AsRef<[u8]>>(slice: T) -> Vec<u8> {
    slice.as_ref().iter().copied().map(swapcase_byte).collect()
}
