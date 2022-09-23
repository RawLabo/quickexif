//! Rule definition and macro for rule generation
//! 
use super::value::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum OffsetType {
    Bytes(isize),
    Address,
    PrevField(&'static str),
}

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum CondType {
    LT,
    EQ,
    GT,
    EXIST,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ParsingRule {
    Tiff(Vec<ParsingRule>),

    Condition {
        cond: (CondType, &'static str, u32),
        left: Vec<ParsingRule>,
        right: Vec<ParsingRule>,
    },
    Offset(OffsetType, Vec<ParsingRule>),
    Jump {
        tag: u16,
        is_optional: bool,
        rules: Vec<ParsingRule>,
    },
    JumpNext(Vec<ParsingRule>),
    Scan {
        marker: &'static [u8],
        name: Option<&'static str>,
        rules: Vec<ParsingRule>,
    },
    SonyDecrypt {
        offset_tag: u16,
        len_tag: u16,
        key_tag: u16,
        rules: Vec<ParsingRule>,
    },
    TagItem {
        tag: u16,
        name: &'static str,
        len: Option<&'static str>,
        is_optional: bool,
        is_value_u16: bool,
    },
    OffsetItem {
        offset: usize,
        name: &'static str,
        t: Value,
    },
}

/// A rule generator with simple syntax
/// 
/// * `tiff { ... }` returns a parsing rule for tiff block.
/// * `template { ... }` generates a partial parsing rule for later use.
/// 
/// * `next { ... }` is to jump to the next ifd session.
/// * `load($ident)` is to load previous defined template.
/// * `if $ident ? { ... }` is to parse an if block to check the existence of a previously parsed item.
/// * `if $ident ? { ... } else { ... }` is to parse an if-else block to check the existence of a previously parsed item.
/// * `if $ident = $ident { ... }` is to parse an if block to check the equality of a previously parsed item and a value.
/// * `if $ident = $ident { ... } else { ... }` is to parse an if-else block to check the equality of a previously parsed item and a value.
/// * `if $ident < $literal { ... }` is to parse an if block to check if the previously parsed item is less than a value.
/// * `if $ident < $literal { ... } else { ... }` is to parse an if-else block to check if the previously parsed item is less than a value.
/// * `if $ident > $literal { ... }` is to parse an if block to check if the previously parsed item is greater than a value.
/// * `if $ident > $literal { ... } else { ... }` is to parse an if-else block to check if the previously parsed item is greater than a value.
/// * `scan [_u8; _] { ... }` is to scan the raw bytes until the marker is met.
/// * `scan [_u8; _] / $ident { ... }` is to scan the raw bytes until the marker is met and store the address to the identity.
/// * `offset address { ... }` is to take the current 4 bytes as address and jump to the address.
/// * `offset + $ident { ... }` is to take the previous item as offset and jump to the new offset.
/// * `offset + $literal { ... }` is to jump to the new offset.
/// * `offset - $literal { ... }` is to jump to the new offset.
/// * `sony_decrypt / $offset_tag:tt / $len_tag:tt / $key_tag:tt { ... }` is to parse a special encrypted block for sony arw file.
/// * `$tag:tt / $name:tt { ... }` is to jump to the address referenced by the tag value and store the address to name.
/// * `$tag:tt ? { ... }` is to jump to the address referenced by the tag value if the tag exists.
/// * `$tag:tt { ... }` is to jump to the address referenced by the tag value.
/// * `$tag:tt / $name:tt($len:tt)` is to collect value and length by tag.
/// * `$tag:tt ? / $name:tt($len:tt)` is to collect value and length by tag if the tag exists.
/// * `$tag:tt ? / $name:tt` is to collect value as name by tag if the tag exists.
/// * `$tag:tt / $name:tt` is to collect value as name by tag.
/// * `$tag:tt : u16 / $name:tt` is to collect the u16 value as name by tag.
/// * `u16 + $offset:tt / $name:tt` is to collect the u16 value as name according to the offset.
/// * `u32 + $offset:tt / $name:tt` is to collect the u32 value as name according to the offset.
/// * `r64 + $offset:tt / $name:tt` is to collect the r64 value as name according to the offset.
/// * `str + $offset:tt / $name:tt` is to collect the string value as name according to the offset.
///
/// ### Example
/// ```no_run
/// // a rule for JPEG parsing
/// let rule = quickexif::describe_rule!(tiff {
///     0x010f {
///         str + 0 / make
///     }
///     0x0110 {
///         str + 0 / model
///     }
///     0x8769 {
///         0x8827 : u16 / iso
///         0x829a {
///             r64 + 0 / exposure_time
///         }
///         0x829d {
///             r64 + 0 / f_number
///         }
///         0x9004 {
///             str + 0 / create_date
///         }
///         0x920a {
///             r64 + 0 / focal_length
///         }
///         0xa002 / width
///         0xa003 / height
///     }
/// });
/// ```
#[macro_export(local_inner_macros)]
macro_rules! describe_rule {
    // entries

    // returns a parsing rule for tiff block
    [tiff {$($body:tt)*}] => {
        $crate::rule::ParsingRule::Tiff(describe_rule![@acc() $($body)*])
    };
    // generates a partial parsing rule for later use
    [template {$($body:tt)*}] => {
        $crate::rule::ParsingRule::Offset($crate::rule::OffsetType::Bytes(0), describe_rule![@acc() $($body)*])
    };

    // blocks

    // To parse a inner tiff block
    [@acc($($x:tt)*) tiff {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::Tiff(describe_rule![@acc() $($body)*]),) $($tails)*]
    };
    // To jump to the next ifd session
    [@acc($($x:tt)*) next {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::JumpNext(describe_rule![@acc() $($body)*]),) $($tails)*]
    };
    // To load previous defined template
    [@acc($($x:tt)*) load($name:ident) $($tails:tt)*] => {
        describe_rule![@acc($($x)* $name.clone(),) $($tails)*]
    };

    // To parse an if-else block to check the existence of a previously parsed item
    [@acc($($x:tt)*) if $a:ident ? {$($body1:tt)*} else {$($body2:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::Condition {
            cond: ($crate::rule::CondType::EXIST, describe_rule!(@stringify $a), 0),
            left: describe_rule![@acc() $($body1)*],
            right: describe_rule![@acc() $($body2)*]
        },) $($tails)*]
    };
    [@acc($($x:tt)*) if $a:ident ? {$($body1:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)*) if $a ? {$($body1)*} else {} $($tails)*]
    };

    // To parse an if-else block to check the equality of a previously parsed item and a value
    [@acc($($x:tt)*) if $a:ident == $b:literal {$($body1:tt)*} else {$($body2:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::Condition {
            cond: ($crate::rule::CondType::EQ, describe_rule!(@stringify $a), $b),
            left: describe_rule![@acc() $($body1)*],
            right: describe_rule![@acc() $($body2)*]
        },) $($tails)*]
    };
    [@acc($($x:tt)*) if $a:ident == $b:literal {$($body1:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)*) if $a == $b {$($body1)*} else {} $($tails)*]
    };

    // To parse an if-else block to check if the previously parsed item is less than a value
    [@acc($($x:tt)*) if $a:ident < $b:literal {$($body1:tt)*} else {$($body2:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::Condition {
            cond: ($crate::rule::CondType::LT, describe_rule!(@stringify $a), $b),
            left: describe_rule![@acc() $($body1)*],
            right: describe_rule![@acc() $($body2)*]
        },) $($tails)*]
    };
    [@acc($($x:tt)*) if $a:ident < $b:literal {$($body1:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)*) if $a < $b {$($body1)*} else {} $($tails)*]
    };

    // To parse an if-else block to check if the previously parsed item is greater than a value
    [@acc($($x:tt)*) if $a:ident > $b:literal {$($body1:tt)*} else {$($body2:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::Condition {
            cond: ($crate::rule::CondType::GT, describe_rule!(@stringify $a), $b),
            left: describe_rule![@acc() $($body1)*],
            right: describe_rule![@acc() $($body2)*]
        },) $($tails)*]
    };
    [@acc($($x:tt)*) if $a:ident > $b:literal {$($body1:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)*) if $a > $b {$($body1)*} else {} $($tails)*]
    };

    // To scan the raw bytes until the marker is met
    [@acc($($x:tt)*) scan [$($marker:tt)*] / $name:ident {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::Scan {
            marker: &[$($marker)*],
            name: Some(describe_rule!(@stringify $name)),
            rules: describe_rule![@acc() $($body)*]
        },) $($tails)*]
    };
    [@acc($($x:tt)*) scan [$($marker:tt)*] {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::Scan {
            target: &[$($marker)*],
            name: None,
            rules: describe_rule![@acc() $($body)*]
        },) $($tails)*]
    };

    // To take the current 4 bytes as address and jump to the address
    [@acc($($x:tt)*) offset address {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::Offset($crate::rule::OffsetType::Address, describe_rule![@acc() $($body)*]),) $($tails)*]
    };
    // To take the previous item as offset and jump to the new offset
    [@acc($($x:tt)*) offset + $offset:ident {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::Offset($crate::rule::OffsetType::PrevField(describe_rule!(@stringify $offset)), describe_rule![@acc() $($body)*]),) $($tails)*]
    };
    // To jump to the new offset
    [@acc($($x:tt)*) offset + $offset:literal {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::Offset($crate::rule::OffsetType::Bytes($offset), describe_rule![@acc() $($body)*]),) $($tails)*]
    };
    [@acc($($x:tt)*) offset - $offset:literal {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::Offset($crate::rule::OffsetType::Bytes(-$offset), describe_rule![@acc() $($body)*]),) $($tails)*]
    };

    // To parse a special encrypted block for sony arw file
    [@acc($($x:tt)*) sony_decrypt / $offset_tag:tt / $len_tag:tt / $key_tag:tt {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::SonyDecrypt {
            offset_tag: $offset_tag,
            len_tag: $len_tag,
            key_tag: $key_tag,
            rules: describe_rule![@acc() $($body)*]
        },) $($tails)*]
    };

    // these rules must be the latest of blocks
    // To jump to the address referenced by the tag value and store the address to name
    [@acc($($x:tt)*) $tag:tt / $name:tt {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)*) $tag / $name $tag {$($body)*} $($tails)*]
    };
    // To jump to the address referenced by the tag value if the tag exists
    [@acc($($x:tt)*) $tag:tt ? {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::Jump {
            tag: $tag,
            is_optional: true,
            rules: describe_rule![@acc() $($body)*]
        },) $($tails)*]
    };
    // To jump to the address referenced by the tag value
    [@acc($($x:tt)*) $tag:tt {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::Jump {
            tag: $tag,
            is_optional: false,
            rules: describe_rule![@acc() $($body)*]
        },) $($tails)*]
    };

    // data types
    // To collect value and length by tag
    [@acc($($x:tt)*) $tag:tt / $name:tt($len:tt) $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::TagItem {
            tag: $tag,
            name: describe_rule!(@stringify $name),
            len: Some(describe_rule!(@stringify $len)),
            is_optional: false,
            is_value_u16: false
        },) $($tails)*]
    };
    // To collect value and length by tag if the tag exists
    [@acc($($x:tt)*) $tag:tt ? / $name:tt($len:tt) $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::TagItem {
            tag: $tag,
            name: describe_rule!(@stringify $name),
            len: Some(describe_rule!(@stringify $len)),
            is_optional: true,
            is_value_u16: false
        },) $($tails)*]
    };
    // To collect value as name by tag if the tag exists
    [@acc($($x:tt)*) $tag:tt ? / $name:tt $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::TagItem {
            tag: $tag,
            name: describe_rule!(@stringify $name),
            len: None,
            is_optional: true,
            is_value_u16: false
        },) $($tails)*]
    };
    // To collect value as name by tag
    [@acc($($x:tt)*) $tag:tt / $name:tt $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::TagItem {
            tag: $tag,
            name: describe_rule!(@stringify $name),
            len: None,
            is_optional: false,
            is_value_u16: false
        },) $($tails)*]
    };
    // To collect the u16 value as name by tag
    [@acc($($x:tt)*) $tag:tt : u16 / $name:tt $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::TagItem {
            tag: $tag,
            name: describe_rule!(@stringify $name),
            len: None,
            is_optional: false,
            is_value_u16: true
        },) $($tails)*]
    };

    // To collect the u16 value as name according to the offset
    [@acc($($x:tt)*) u16 + $offset:tt / $name:tt $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::OffsetItem {
            offset: $offset,
            name: describe_rule!(@stringify $name),
            t: $crate::value::Value::U16(0)
        },) $($tails)*]
    };
    // To collect the u32 value as name according to the offset
    [@acc($($x:tt)*) u32 + $offset:tt / $name:tt $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::OffsetItem {
            offset: $offset,
            name: describe_rule!(@stringify $name),
            t: $crate::value::Value::U32(0)
        },) $($tails)*]
    };
    // To collect the r64 value as name according to the offset
    [@acc($($x:tt)*) r64 + $offset:tt / $name:tt $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::OffsetItem {
            offset: $offset,
            name: describe_rule!(@stringify $name),
            t: $crate::value::Value::R64(0.)
        },) $($tails)*]
    };
    // To collect the string as name according to the offset
    [@acc($($x:tt)*) str + $offset:tt / $name:tt $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::rule::ParsingRule::OffsetItem {
            offset: $offset,
            name: describe_rule!(@stringify $name),
            t: $crate::value::Value::Str("".to_owned())
        },) $($tails)*]
    };

    // reduction
    [@acc($($x:tt)*)] => {
        std::vec![$($x)*]
    };

    // util
    [@stringify $name:tt] => { std::stringify!($name) };
}
