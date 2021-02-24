use std::{borrow::Cow, fmt::Display, str::FromStr};

use serde::{Serialize, Serializer};

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Value<'a> {
    Boolean(Boolean<'a>),
    Integer(Integer),
    Color(Color),
    Other(Cow<'a, str>),
}

impl<'a> Value<'a> {
    pub fn from_str(s: &'a str) -> Self {
        if let Ok(bool) = Boolean::from_str(s) {
            return Self::Boolean(bool);
        }

        if let Ok(int) = Integer::from_str(s) {
            return Self::Integer(int);
        }

        // if let Ok(color) = Color::from_str(s) {
        //     return Self::Color(color);
        // }

        Self::Other(Cow::Borrowed(s))
    }

    pub fn from_string(s: String) -> Self {
        Self::Other(Cow::Owned(s))
    }
}

// todo display for value

impl Serialize for Value<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Value::Boolean(b) => b.serialize(serializer),
            Value::Integer(i) => i.serialize(serializer),
            Value::Color(c) => c.serialize(serializer),
            Value::Other(i) => i.serialize(serializer),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum Boolean<'a> {
    True(TrueVariant<'a>),
    False(FalseVariant<'a>),
}

impl<'a> Boolean<'a> {
    pub fn from_str(value: &'a str) -> Result<Self, ()> {
        if let Ok(v) = TrueVariant::from_str(value) {
            return Ok(Self::True(v));
        }

        if let Ok(v) = FalseVariant::from_str(value) {
            return Ok(Self::False(v));
        }

        Err(())
    }
}

// todo: Display for boolean

impl Serialize for Boolean<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            Boolean::True(_) => serializer.serialize_bool(true),
            Boolean::False(_) => serializer.serialize_bool(false),
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum TrueVariant<'a> {
    Explicit(&'a str),
    /// For variables defined without a `= <value>`. This can never be created
    /// from the [`FromStr`] trait, as an empty string is false without context.
    /// If directly serializing this struct (instead of using a higher level
    /// wrapper), then this variant is serialized as if it was `true`.
    Implicit,
}

impl<'a> TrueVariant<'a> {
    pub fn from_str(value: &'a str) -> Result<TrueVariant<'a>, ()> {
        if value.eq_ignore_ascii_case("yes")
            || value.eq_ignore_ascii_case("on")
            || value.eq_ignore_ascii_case("true")
            || value.eq_ignore_ascii_case("one")
        {
            Ok(Self::Explicit(value))
        } else {
            Err(())
        }
    }
}

impl Display for TrueVariant<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Self::Explicit(v) = self {
            write!(f, "{}", v)
        } else {
            Ok(())
        }
    }
}

impl Serialize for TrueVariant<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bool(true)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct FalseVariant<'a>(&'a str);

impl<'a> FalseVariant<'a> {
    pub fn from_str(value: &'a str) -> Result<FalseVariant<'a>, ()> {
        if value.eq_ignore_ascii_case("no")
            || value.eq_ignore_ascii_case("off")
            || value.eq_ignore_ascii_case("false")
            || value.eq_ignore_ascii_case("zero")
            || value == "\"\""
        {
            Ok(Self(value))
        } else {
            Err(())
        }
    }
}

impl Display for FalseVariant<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Serialize for FalseVariant<'_> {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_bool(false)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub struct Integer {
    value: i64,
    suffix: Option<IntegerSuffix>,
}

impl Display for Integer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.value)?;
        if let Some(suffix) = self.suffix {
            write!(f, "{}", suffix)
        } else {
            Ok(())
        }
    }
}

impl Serialize for Integer {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        if let Some(suffix) = self.suffix {
            serializer.serialize_i64(self.value << suffix.bitwise_offset())
        } else {
            serializer.serialize_i64(self.value)
        }
    }
}

impl FromStr for Integer {
    type Err = String;

    fn from_str<'a>(s: &'a str) -> Result<Self, Self::Err> {
        if let Ok(value) = s.parse() {
            return Ok(Self {
                value,
                suffix: None,
            });
        }

        // Assume we have a prefix at this point.

        if s.len() <= 1 {
            return Err(s.to_string());
        }

        let (number, suffix) = s.split_at(s.len() - 1);
        if let (Ok(value), Ok(suffix)) = (number.parse(), suffix.parse()) {
            Ok(Self {
                value,
                suffix: Some(suffix),
            })
        } else {
            Err(s.to_string())
        }
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
enum IntegerSuffix {
    Kilo,
    Mega,
    Giga,
}

impl IntegerSuffix {
    fn bitwise_offset(&self) -> usize {
        match self {
            Self::Kilo => 10,
            Self::Mega => 20,
            Self::Giga => 30,
        }
    }
}

impl Display for IntegerSuffix {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Kilo => write!(f, "k"),
            Self::Mega => write!(f, "m"),
            Self::Giga => write!(f, "g"),
        }
    }
}

impl Serialize for IntegerSuffix {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        serializer.serialize_str(match self {
            Self::Kilo => "k",
            Self::Mega => "m",
            Self::Giga => "g",
        })
    }
}

impl FromStr for IntegerSuffix {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "k" => Ok(Self::Kilo),
            "m" => Ok(Self::Mega),
            "g" => Ok(Self::Giga),
            _ => Err(()),
        }
    }
}

#[derive(Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug, Default)]
pub struct Color {
    foreground: Option<ColorValue>,
    background: Option<ColorValue>,
    attributes: Vec<ColorAttribute>,
}

impl Display for Color {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if let Some(fg) = self.foreground {
            fg.fmt(f)?;
        }

        write!(f, " ")?;

        if let Some(bg) = self.background {
            bg.fmt(f)?;
        }

        self.attributes
            .iter()
            .map(|attr| write!(f, " ").and_then(|_| attr.fmt(f)))
            .collect::<Result<_, _>>()
    }
}

impl Serialize for Color {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

pub enum FromColorErr {
    TooManyColorValues,
    InvalidColorOption,
}

impl FromStr for Color {
    type Err = FromColorErr;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        enum ColorItem {
            Value(ColorValue),
            Attr(ColorAttribute),
        }

        let items = s.split_whitespace().filter_map(|s| {
            if s.is_empty() {
                return None;
            }

            Some(
                ColorValue::from_str(s)
                    .map(ColorItem::Value)
                    .or_else(|_| ColorAttribute::from_str(s).map(ColorItem::Attr)),
            )
        });

        let mut new_self = Self::default();
        for item in items {
            match item {
                Ok(item) => match item {
                    ColorItem::Value(v) => {
                        if new_self.foreground.is_none() {
                            new_self.foreground = Some(v);
                        } else if new_self.background.is_none() {
                            new_self.background = Some(v);
                        } else {
                            return Err(FromColorErr::TooManyColorValues);
                        }
                    }
                    ColorItem::Attr(a) => new_self.attributes.push(a),
                },
                Err(_) => return Err(FromColorErr::InvalidColorOption),
            }
        }

        Ok(new_self)
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
enum ColorValue {
    Normal,
    Black,
    BrightBlack,
    Red,
    BrightRed,
    Green,
    BrightGreen,
    Yellow,
    BrightYellow,
    Blue,
    BrightBlue,
    Magenta,
    BrightMagenta,
    Cyan,
    BrightCyan,
    White,
    BrightWhite,
    Ansi(u8),
    Rgb(u8, u8, u8),
}

impl Display for ColorValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Normal => write!(f, "normal"),
            Self::Black => write!(f, "black"),
            Self::BrightBlack => write!(f, "brightblack"),
            Self::Red => write!(f, "red"),
            Self::BrightRed => write!(f, "brightred"),
            Self::Green => write!(f, "green"),
            Self::BrightGreen => write!(f, "brightgreen"),
            Self::Yellow => write!(f, "yellow"),
            Self::BrightYellow => write!(f, "brightyellow"),
            Self::Blue => write!(f, "blue"),
            Self::BrightBlue => write!(f, "brightblue"),
            Self::Magenta => write!(f, "magenta"),
            Self::BrightMagenta => write!(f, "brightmagenta"),
            Self::Cyan => write!(f, "cyan"),
            Self::BrightCyan => write!(f, "brightcyan"),
            Self::White => write!(f, "white"),
            Self::BrightWhite => write!(f, "brightwhite"),
            Self::Ansi(num) => num.fmt(f),
            Self::Rgb(r, g, b) => write!(f, "#{:02x}{:02x}{:02x}", r, g, b),
        }
    }
}

impl Serialize for ColorValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(&self.to_string())
    }
}

impl FromStr for ColorValue {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let mut s = s;
        let bright = if s.starts_with("bright") {
            s = &s[6..];
            true
        } else {
            false
        };

        match s {
            "normal" if !bright => return Ok(Self::Normal),
            "normal" if bright => return Err(()),
            "black" if !bright => return Ok(Self::Black),
            "black" if bright => return Ok(Self::BrightBlack),
            "red" if !bright => return Ok(Self::Red),
            "red" if bright => return Ok(Self::BrightRed),
            "green" if !bright => return Ok(Self::Green),
            "green" if bright => return Ok(Self::BrightGreen),
            "yellow" if !bright => return Ok(Self::Yellow),
            "yellow" if bright => return Ok(Self::BrightYellow),
            "blue" if !bright => return Ok(Self::Blue),
            "blue" if bright => return Ok(Self::BrightBlue),
            "magenta" if !bright => return Ok(Self::Magenta),
            "magenta" if bright => return Ok(Self::BrightMagenta),
            "cyan" if !bright => return Ok(Self::Cyan),
            "cyan" if bright => return Ok(Self::BrightCyan),
            "white" if !bright => return Ok(Self::White),
            "white" if bright => return Ok(Self::BrightWhite),
            _ => (),
        }

        if let Ok(v) = u8::from_str(s) {
            return Ok(Self::Ansi(v));
        }

        if s.starts_with("#") {
            let s = &s[1..];
            if s.len() == 6 {
                let rgb = (
                    u8::from_str_radix(&s[..2], 16),
                    u8::from_str_radix(&s[2..4], 16),
                    u8::from_str_radix(&s[4..], 16),
                );

                if let (Ok(r), Ok(g), Ok(b)) = rgb {
                    return Ok(Self::Rgb(r, g, b));
                }
            }
        }

        Err(())
    }
}

#[derive(Copy, Clone, Eq, PartialEq, Ord, PartialOrd, Hash, Debug)]
pub enum ColorAttribute {
    Bold,
    NoBold,
    Dim,
    NoDim,
    Ul,
    NoUl,
    Blink,
    NoBlink,
    Reverse,
    NoReverse,
    Italic,
    NoItalic,
    Strike,
    NoStrike,
}

impl Display for ColorAttribute {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Bold => write!(f, "bold"),
            Self::NoBold => write!(f, "nobold"),
            Self::Dim => write!(f, "dim"),
            Self::NoDim => write!(f, "nodim"),
            Self::Ul => write!(f, "ul"),
            Self::NoUl => write!(f, "noul"),
            Self::Blink => write!(f, "blink"),
            Self::NoBlink => write!(f, "noblink"),
            Self::Reverse => write!(f, "reverse"),
            Self::NoReverse => write!(f, "noreverse"),
            Self::Italic => write!(f, "italic"),
            Self::NoItalic => write!(f, "noitalic"),
            Self::Strike => write!(f, "strike"),
            Self::NoStrike => write!(f, "nostrike"),
        }
    }
}

impl Serialize for ColorAttribute {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        serializer.serialize_str(match self {
            Self::Bold => "bold",
            Self::NoBold => "nobold",
            Self::Dim => "dim",
            Self::NoDim => "nodim",
            Self::Ul => "ul",
            Self::NoUl => "noul",
            Self::Blink => "blink",
            Self::NoBlink => "noblink",
            Self::Reverse => "reverse",
            Self::NoReverse => "noreverse",
            Self::Italic => "italic",
            Self::NoItalic => "noitalic",
            Self::Strike => "strike",
            Self::NoStrike => "nostrike",
        })
    }
}

impl FromStr for ColorAttribute {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let inverted = s.starts_with("no");
        let mut parsed = s;

        if inverted {
            parsed = &parsed[2..];

            if parsed.starts_with("-") {
                parsed = &parsed[1..];
            }
        }

        match parsed {
            "bold" if !inverted => Ok(Self::Bold),
            "bold" if inverted => Ok(Self::NoBold),
            "dim" if !inverted => Ok(Self::Dim),
            "dim" if inverted => Ok(Self::NoDim),
            "ul" if !inverted => Ok(Self::Ul),
            "ul" if inverted => Ok(Self::NoUl),
            "blink" if !inverted => Ok(Self::Blink),
            "blink" if inverted => Ok(Self::NoBlink),
            "reverse" if !inverted => Ok(Self::Reverse),
            "reverse" if inverted => Ok(Self::NoReverse),
            "italic" if !inverted => Ok(Self::Italic),
            "italic" if inverted => Ok(Self::NoItalic),
            "strike" if !inverted => Ok(Self::Strike),
            "strike" if inverted => Ok(Self::NoStrike),
            _ => Err(()),
        }
    }
}

#[cfg(test)]
mod integer {
    use super::*;

    #[test]
    fn from_str_no_suffix() {
        assert_eq!(
            Integer::from_str("1").unwrap(),
            Integer {
                value: 1,
                suffix: None
            }
        );

        assert_eq!(
            Integer::from_str("-1").unwrap(),
            Integer {
                value: -1,
                suffix: None
            }
        );
    }

    #[test]
    fn from_str_with_suffix() {
        assert_eq!(
            Integer::from_str("1k").unwrap(),
            Integer {
                value: 1,
                suffix: Some(IntegerSuffix::Kilo),
            }
        );

        assert_eq!(
            Integer::from_str("1m").unwrap(),
            Integer {
                value: 1,
                suffix: Some(IntegerSuffix::Mega),
            }
        );

        assert_eq!(
            Integer::from_str("1g").unwrap(),
            Integer {
                value: 1,
                suffix: Some(IntegerSuffix::Giga),
            }
        );
    }

    #[test]
    fn invalid_from_str() {
        assert!(Integer::from_str("").is_err());
        assert!(Integer::from_str("-").is_err());
        assert!(Integer::from_str("k").is_err());
        assert!(Integer::from_str("m").is_err());
        assert!(Integer::from_str("g").is_err());
        assert!(Integer::from_str("123123123123123123123123").is_err());
        assert!(Integer::from_str("gg").is_err());
    }
}

#[cfg(test)]
mod color_value {
    use super::ColorValue;
    use std::str::FromStr;

    #[test]
    fn non_bright() {
        assert_eq!(ColorValue::from_str("normal"), Ok(ColorValue::Normal));
        assert_eq!(ColorValue::from_str("black"), Ok(ColorValue::Black));
        assert_eq!(ColorValue::from_str("red"), Ok(ColorValue::Red));
        assert_eq!(ColorValue::from_str("green"), Ok(ColorValue::Green));
        assert_eq!(ColorValue::from_str("yellow"), Ok(ColorValue::Yellow));
        assert_eq!(ColorValue::from_str("blue"), Ok(ColorValue::Blue));
        assert_eq!(ColorValue::from_str("magenta"), Ok(ColorValue::Magenta));
        assert_eq!(ColorValue::from_str("cyan"), Ok(ColorValue::Cyan));
        assert_eq!(ColorValue::from_str("white"), Ok(ColorValue::White));
    }

    #[test]
    fn bright() {
        assert_eq!(
            ColorValue::from_str("brightblack"),
            Ok(ColorValue::BrightBlack)
        );
        assert_eq!(ColorValue::from_str("brightred"), Ok(ColorValue::BrightRed));
        assert_eq!(
            ColorValue::from_str("brightgreen"),
            Ok(ColorValue::BrightGreen)
        );
        assert_eq!(
            ColorValue::from_str("brightyellow"),
            Ok(ColorValue::BrightYellow)
        );
        assert_eq!(
            ColorValue::from_str("brightblue"),
            Ok(ColorValue::BrightBlue)
        );
        assert_eq!(
            ColorValue::from_str("brightmagenta"),
            Ok(ColorValue::BrightMagenta)
        );
        assert_eq!(
            ColorValue::from_str("brightcyan"),
            Ok(ColorValue::BrightCyan)
        );
        assert_eq!(
            ColorValue::from_str("brightwhite"),
            Ok(ColorValue::BrightWhite)
        );
    }

    #[test]
    fn ansi() {
        assert_eq!(ColorValue::from_str("255"), Ok(ColorValue::Ansi(255)));
        assert_eq!(ColorValue::from_str("0"), Ok(ColorValue::Ansi(0)));
    }

    #[test]
    fn hex() {
        assert_eq!(
            ColorValue::from_str("#ff0010"),
            Ok(ColorValue::Rgb(255, 0, 16))
        );
        assert_eq!(
            ColorValue::from_str("#ffffff"),
            Ok(ColorValue::Rgb(255, 255, 255))
        );
        assert_eq!(
            ColorValue::from_str("#000000"),
            Ok(ColorValue::Rgb(0, 0, 0))
        );
    }

    #[test]
    fn invalid() {
        assert!(ColorValue::from_str("brightnormal").is_err());
        assert!(ColorValue::from_str("").is_err());
        assert!(ColorValue::from_str("bright").is_err());
        assert!(ColorValue::from_str("256").is_err());
        assert!(ColorValue::from_str("#").is_err());
        assert!(ColorValue::from_str("#fff").is_err());
        assert!(ColorValue::from_str("#gggggg").is_err());
    }
}

#[cfg(test)]
mod color_attribute {
    use super::ColorAttribute;
    use std::str::FromStr;

    #[test]
    fn non_inverted() {
        assert_eq!(ColorAttribute::from_str("bold"), Ok(ColorAttribute::Bold));
        assert_eq!(ColorAttribute::from_str("dim"), Ok(ColorAttribute::Dim));
        assert_eq!(ColorAttribute::from_str("ul"), Ok(ColorAttribute::Ul));
        assert_eq!(ColorAttribute::from_str("blink"), Ok(ColorAttribute::Blink));
        assert_eq!(
            ColorAttribute::from_str("reverse"),
            Ok(ColorAttribute::Reverse)
        );
        assert_eq!(
            ColorAttribute::from_str("italic"),
            Ok(ColorAttribute::Italic)
        );
        assert_eq!(
            ColorAttribute::from_str("strike"),
            Ok(ColorAttribute::Strike)
        );
    }

    #[test]
    fn inverted_no_dash() {
        assert_eq!(
            ColorAttribute::from_str("nobold"),
            Ok(ColorAttribute::NoBold)
        );
        assert_eq!(ColorAttribute::from_str("nodim"), Ok(ColorAttribute::NoDim));
        assert_eq!(ColorAttribute::from_str("noul"), Ok(ColorAttribute::NoUl));
        assert_eq!(
            ColorAttribute::from_str("noblink"),
            Ok(ColorAttribute::NoBlink)
        );
        assert_eq!(
            ColorAttribute::from_str("noreverse"),
            Ok(ColorAttribute::NoReverse)
        );
        assert_eq!(
            ColorAttribute::from_str("noitalic"),
            Ok(ColorAttribute::NoItalic)
        );
        assert_eq!(
            ColorAttribute::from_str("nostrike"),
            Ok(ColorAttribute::NoStrike)
        );
    }

    #[test]
    fn inverted_dashed() {
        assert_eq!(
            ColorAttribute::from_str("no-bold"),
            Ok(ColorAttribute::NoBold)
        );
        assert_eq!(
            ColorAttribute::from_str("no-dim"),
            Ok(ColorAttribute::NoDim)
        );
        assert_eq!(ColorAttribute::from_str("no-ul"), Ok(ColorAttribute::NoUl));
        assert_eq!(
            ColorAttribute::from_str("no-blink"),
            Ok(ColorAttribute::NoBlink)
        );
        assert_eq!(
            ColorAttribute::from_str("no-reverse"),
            Ok(ColorAttribute::NoReverse)
        );
        assert_eq!(
            ColorAttribute::from_str("no-italic"),
            Ok(ColorAttribute::NoItalic)
        );
        assert_eq!(
            ColorAttribute::from_str("no-strike"),
            Ok(ColorAttribute::NoStrike)
        );
    }

    #[test]
    fn invalid() {
        assert!(ColorAttribute::from_str("a").is_err());
        assert!(ColorAttribute::from_str("no bold").is_err());
        assert!(ColorAttribute::from_str("").is_err());
        assert!(ColorAttribute::from_str("no").is_err());
        assert!(ColorAttribute::from_str("no-").is_err());
    }
}
