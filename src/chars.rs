use std::fmt::{self, Debug, Display};

use crate::chars::case_fold::CASE_FOLDING_SIMPLE;
use crate::MatcherConfig;

//autogenerated by generate-ucd
#[allow(warnings)]
#[rustfmt::skip]
mod case_fold;
mod normalize;

pub trait Char: Copy + Eq + Ord + fmt::Display {
    const ASCII: bool;
    fn char_class(self, config: &MatcherConfig) -> CharClass;
    fn char_class_and_normalize(self, config: &MatcherConfig) -> (Self, CharClass);
    fn normalize(self, config: &MatcherConfig) -> Self;
}

/// repr tansparent wrapper around u8 with better formatting and PartialEq<char> implementation
#[repr(transparent)]
#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) struct AsciiChar(u8);

impl AsciiChar {
    pub fn cast(bytes: &[u8]) -> &[AsciiChar] {
        unsafe { &*(bytes as *const [u8] as *const [AsciiChar]) }
    }
}

impl fmt::Display for AsciiChar {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&(self.0 as char), f)
    }
}

impl PartialEq<AsciiChar> for char {
    fn eq(&self, other: &AsciiChar) -> bool {
        other.0 as char == *self
    }
}

impl Char for AsciiChar {
    const ASCII: bool = true;
    #[inline]
    fn char_class(self, config: &MatcherConfig) -> CharClass {
        let c = self.0;
        // using manual if conditions instead optimizes better
        if c >= b'a' && c <= b'z' {
            CharClass::Lower
        } else if c >= b'A' && c <= b'Z' {
            CharClass::Upper
        } else if c >= b'0' && c <= b'9' {
            CharClass::Number
        } else if c.is_ascii_whitespace() {
            CharClass::Whitespace
        } else if config.delimiter_chars.contains(&c) {
            CharClass::Delimiter
        } else {
            CharClass::NonWord
        }
    }

    #[inline(always)]
    fn char_class_and_normalize(mut self, config: &MatcherConfig) -> (Self, CharClass) {
        let char_class = self.char_class(config);
        if config.ignore_case && char_class == CharClass::Upper {
            self.0 += 32
        }
        (self, char_class)
    }

    #[inline(always)]
    fn normalize(mut self, config: &MatcherConfig) -> Self {
        if config.ignore_case && self.0 >= b'A' && self.0 <= b'Z' {
            self.0 += 32
        }
        self
    }
}
fn char_class_non_ascii(c: char) -> CharClass {
    if c.is_lowercase() {
        CharClass::Lower
    } else if c.is_uppercase() {
        CharClass::Upper
    } else if c.is_numeric() {
        CharClass::Number
    } else if c.is_alphabetic() {
        CharClass::Letter
    } else if c.is_whitespace() {
        CharClass::Whitespace
    } else {
        CharClass::NonWord
    }
}
impl Char for char {
    const ASCII: bool = false;
    #[inline(always)]
    fn char_class(self, config: &MatcherConfig) -> CharClass {
        if self.is_ascii() {
            return AsciiChar(self as u8).char_class(config);
        }
        char_class_non_ascii(self)
    }

    #[inline(always)]
    fn char_class_and_normalize(mut self, config: &MatcherConfig) -> (Self, CharClass) {
        if self.is_ascii() {
            let (c, class) = AsciiChar(self as u8).char_class_and_normalize(config);
            return (c.0 as char, class);
        }
        let char_class = char_class_non_ascii(self);
        if char_class == CharClass::Upper && config.ignore_case {
            self = CASE_FOLDING_SIMPLE
                .binary_search_by_key(&self, |(upper, _)| *upper)
                .map_or(self, |idx| CASE_FOLDING_SIMPLE[idx].1)
        }
        if config.normalize {
            self = normalize::normalize(self);
        }
        (self, char_class)
    }

    #[inline(always)]
    fn normalize(mut self, config: &MatcherConfig) -> Self {
        if config.normalize {
            self = normalize::normalize(self);
        }
        if config.ignore_case {
            self = to_lower_case(self)
        }
        self
    }
}

pub use normalize::normalize;

#[inline(always)]
pub fn to_lower_case(c: char) -> char {
    if c >= 'A' && c <= 'Z' {
        char::from_u32(c as u32 + 32).unwrap()
    } else if !c.is_ascii() {
        CASE_FOLDING_SIMPLE
            .binary_search_by_key(&c, |(upper, _)| *upper)
            .map_or(c, |idx| CASE_FOLDING_SIMPLE[idx].1)
    } else {
        c
    }
}

#[derive(Debug, Eq, PartialEq, PartialOrd, Ord, Copy, Clone, Hash)]
#[non_exhaustive]
pub enum CharClass {
    Whitespace,
    NonWord,
    Delimiter,
    Lower,
    Upper,
    Letter,
    Number,
}
