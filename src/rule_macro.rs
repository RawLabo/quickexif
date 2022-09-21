#[macro_export(local_inner_macros)]
macro_rules! describe_rule {
    // entries

    // return a parsing task for tiff block
    [tiff {$($body:tt)*}] => {
        $crate::exif::ExifTask::Tiff(describe_rule![@acc() $($body)*])
    };
    // generate a partial parsing task for later use
    [template {$($body:tt)*}] => {
        $crate::exif::ExifTask::Offset($crate::exif::OffsetType::Bytes(0), describe_rule![@acc() $($body)*])
    };

    // blocks

    // parse a inner tiff block
    [@acc($($x:tt)*) tiff {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::Tiff(describe_rule![@acc() $($body)*]),) $($tails)*]
    };
    // jump to the next ifd session
    [@acc($($x:tt)*) next {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::JumpNext(describe_rule![@acc() $($body)*]),) $($tails)*]
    };
    // load previous defined template
    [@acc($($x:tt)*) load($name:ident) $($tails:tt)*] => {
        describe_rule![@acc($($x)* $name.clone(),) $($tails)*]
    };

    // parse an if-else block to check the existence of a previously parsed item
    [@acc($($x:tt)*) if $a:ident ? {$($body1:tt)*} else {$($body2:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::Condition {
            cond: ($crate::exif::CondType::EXIST, describe_rule!(@stringify $a), 0),
            left: describe_rule![@acc() $($body1)*],
            right: describe_rule![@acc() $($body2)*]
        },) $($tails)*]
    };
    [@acc($($x:tt)*) if $a:ident ? {$($body1:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)*) if $a ? {$($body1)*} else {} $($tails)*]
    };

    // parse an if-else block to check the equality of a previously parsed item and a value
    [@acc($($x:tt)*) if $a:ident == $b:literal {$($body1:tt)*} else {$($body2:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::Condition {
            cond: ($crate::exif::CondType::EQ, describe_rule!(@stringify $a), $b),
            left: describe_rule![@acc() $($body1)*],
            right: describe_rule![@acc() $($body2)*]
        },) $($tails)*]
    };
    [@acc($($x:tt)*) if $a:ident == $b:literal {$($body1:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)*) if $a == $b {$($body1)*} else {} $($tails)*]
    };

    // parse an if-else block to check if the previously parsed item is less than a value
    [@acc($($x:tt)*) if $a:ident < $b:literal {$($body1:tt)*} else {$($body2:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::Condition {
            cond: ($crate::exif::CondType::LT, describe_rule!(@stringify $a), $b),
            left: describe_rule![@acc() $($body1)*],
            right: describe_rule![@acc() $($body2)*]
        },) $($tails)*]
    };
    [@acc($($x:tt)*) if $a:ident < $b:literal {$($body1:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)*) if $a < $b {$($body1)*} else {} $($tails)*]
    };

    // parse an if-else block to check if the previously parsed item is greater than a value
    [@acc($($x:tt)*) if $a:ident > $b:literal {$($body1:tt)*} else {$($body2:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::Condition {
            cond: ($crate::exif::CondType::GT, describe_rule!(@stringify $a), $b),
            left: describe_rule![@acc() $($body1)*],
            right: describe_rule![@acc() $($body2)*]
        },) $($tails)*]
    };
    [@acc($($x:tt)*) if $a:ident > $b:literal {$($body1:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)*) if $a > $b {$($body1)*} else {} $($tails)*]
    };

    // scan the raw bytes until the marker is met
    [@acc($($x:tt)*) scan [$($marker:tt)*] / $name:ident {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::Scan {
            marker: &[$($marker)*],
            name: Some(describe_rule!(@stringify $name)),
            tasks: describe_rule![@acc() $($body)*]
        },) $($tails)*]
    };
    [@acc($($x:tt)*) scan [$($marker:tt)*] {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::Scan {
            target: &[$($marker)*],
            name: None,
            tasks: describe_rule![@acc() $($body)*]
        },) $($tails)*]
    };

    // take the current 4 bytes as address and jump to the address
    [@acc($($x:tt)*) offset address {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::Offset($crate::exif::OffsetType::Address, describe_rule![@acc() $($body)*]),) $($tails)*]
    };
    // take the previous item as offset and jump to the new offset
    [@acc($($x:tt)*) offset + $offset:ident {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::Offset($crate::exif::OffsetType::PrevField(describe_rule!(@stringify $offset)), describe_rule![@acc() $($body)*]),) $($tails)*]
    };
    // jump to the new offset
    [@acc($($x:tt)*) offset + $offset:literal {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::Offset($crate::exif::OffsetType::Bytes($offset), describe_rule![@acc() $($body)*]),) $($tails)*]
    };
    [@acc($($x:tt)*) offset - $offset:literal {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::Offset($crate::exif::OffsetType::Bytes(-$offset), describe_rule![@acc() $($body)*]),) $($tails)*]
    };

    // parse a special encrypted block for sony arw file
    [@acc($($x:tt)*) sony_decrypt / $offset_tag:tt / $len_tag:tt / $key_tag:tt {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::SonyDecrypt {
            offset_tag: $offset_tag,
            len_tag: $len_tag,
            key_tag: $key_tag,
            tasks: describe_rule![@acc() $($body)*]
        },) $($tails)*]
    };

    // these rules must be the latest of blocks
    // jump to the address referenced by the tag value and store the address to name
    [@acc($($x:tt)*) $tag:tt / $name:tt {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)*) $tag / $name $tag {$($body)*} $($tails)*]
    };
    // jump to the address referenced by the tag value if the tag exists
    [@acc($($x:tt)*) $tag:tt ? {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::Jump {
            tag: $tag,
            is_optional: true,
            tasks: describe_rule![@acc() $($body)*]
        },) $($tails)*]
    };
    // jump to the address referenced by the tag value
    [@acc($($x:tt)*) $tag:tt {$($body:tt)*} $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::Jump {
            tag: $tag,
            is_optional: false,
            tasks: describe_rule![@acc() $($body)*]
        },) $($tails)*]
    };

    // data types
    // collect value and length by tag
    [@acc($($x:tt)*) $tag:tt / $name:tt($len:tt) $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::TagItem {
            tag: $tag,
            name: describe_rule!(@stringify $name),
            len: Some(describe_rule!(@stringify $len)),
            is_optional: false,
            is_value_u16: false
        },) $($tails)*]
    };
    // collect value and length by tag if the tag exists
    [@acc($($x:tt)*) $tag:tt ? / $name:tt($len:tt) $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::TagItem {
            tag: $tag,
            name: describe_rule!(@stringify $name),
            len: Some(describe_rule!(@stringify $len)),
            is_optional: true,
            is_value_u16: false
        },) $($tails)*]
    };
    // collect value to name by tag if the tag exists
    [@acc($($x:tt)*) $tag:tt ? / $name:tt $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::TagItem {
            tag: $tag,
            name: describe_rule!(@stringify $name),
            len: None,
            is_optional: true,
            is_value_u16: false
        },) $($tails)*]
    };
    // collect value to name by tag
    [@acc($($x:tt)*) $tag:tt / $name:tt $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::TagItem {
            tag: $tag,
            name: describe_rule!(@stringify $name),
            len: None,
            is_optional: false,
            is_value_u16: false
        },) $($tails)*]
    };
    // collect the u16 value to name by tag
    [@acc($($x:tt)*) $tag:tt : u16 / $name:tt $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::TagItem {
            tag: $tag,
            name: describe_rule!(@stringify $name),
            len: None,
            is_optional: false,
            is_value_u16: true
        },) $($tails)*]
    };

    // collect the u16 value to name according to the offset
    [@acc($($x:tt)*) u16 + $offset:tt / $name:tt $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::OffsetItem {
            offset: $offset,
            name: describe_rule!(@stringify $name),
            t: $crate::value::Value::U16(0)
        },) $($tails)*]
    };
    // collect the u32 value to name according to the offset
    [@acc($($x:tt)*) u32 + $offset:tt / $name:tt $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::OffsetItem {
            offset: $offset,
            name: describe_rule!(@stringify $name),
            t: $crate::value::Value::U32(0)
        },) $($tails)*]
    };
    // collect the r64 value to name according to the offset
    [@acc($($x:tt)*) r64 + $offset:tt / $name:tt $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::OffsetItem {
            offset: $offset,
            name: describe_rule!(@stringify $name),
            t: $crate::value::Value::R64(0.)
        },) $($tails)*]
    };
    // collect the string to name according to the offset
    [@acc($($x:tt)*) str + $offset:tt / $name:tt $($tails:tt)*] => {
        describe_rule![@acc($($x)* $crate::exif::ExifTask::OffsetItem {
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
