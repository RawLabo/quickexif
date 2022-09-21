use quickexif;
use quickexif::exif::{CondType::*, ExifTask::*, OffsetType::*};
use quickexif::value::Value::*;

#[test]
fn test_describe_rules_template() {
    let tpl1 = quickexif::describe_rule!(template {
        0x010f {
            str + 0 / make
            u32 + 1 / cfa
            u16 + 0 / cfa
            r64 + 0 / cfa
        }

        0x0112 : u16 / orientation3

        0x0096? / linear_table_offset(linear_table_len)
    });
    let tpl2 = quickexif::describe_rule!(template {
        0x7310 {
            u16 + 0 / black_level
        }
        0x7312 {
            u16 + 0 / white_balance_r
            u16 + 1 / white_balance_g
            u16 + 3 / white_balance_b
        }
        0x787f / legacy_white_level {
            u16 + 0 / white_level
        }
    });
    let task = quickexif::describe_rule!(tiff {
        load(tpl1)

        0xc61d / wl(white_level_len)
        if white_level_len == 1
        {
            0x0024 {
                load(tpl2)
            }
        }
        else
        {
            offset + 32 {
                load(tpl2)
            }
        }
    });

    let answer = Tiff(vec![
        Offset(
            Bytes(0),
            vec![
                Jump {
                    tag: 271,
                    is_optional: false,
                    tasks: vec![
                        OffsetItem {
                            offset: 0,
                            name: "make",
                            t: Str("".to_owned()),
                        },
                        OffsetItem {
                            offset: 1,
                            name: "cfa",
                            t: U32(0),
                        },
                        OffsetItem {
                            offset: 0,
                            name: "cfa",
                            t: U16(0),
                        },
                        OffsetItem {
                            offset: 0,
                            name: "cfa",
                            t: R64(0.0),
                        },
                    ],
                },
                TagItem {
                    tag: 274,
                    name: "orientation3",
                    len: None,
                    is_optional: false,
                    is_value_u16: true,
                },
                TagItem {
                    tag: 150,
                    name: "linear_table_offset",
                    len: Some("linear_table_len"),
                    is_optional: true,
                    is_value_u16: false,
                },
            ],
        ),
        TagItem {
            tag: 50717,
            name: "wl",
            len: Some("white_level_len"),
            is_optional: false,
            is_value_u16: false,
        },
        Condition {
            cond: (EQ, "white_level_len", 1),
            left: vec![Jump {
                tag: 36,
                is_optional: false,
                tasks: vec![Offset(
                    Bytes(0),
                    vec![
                        Jump {
                            tag: 29456,
                            is_optional: false,
                            tasks:vec![OffsetItem {
                                offset: 0,
                                name: "black_level",
                                t: U16(0),
                            }],
                        },
                        Jump {
                            tag: 29458,
                            is_optional: false,
                            tasks: vec![
                                OffsetItem {
                                    offset: 0,
                                    name: "white_balance_r",
                                    t: U16(0),
                                },
                                OffsetItem {
                                    offset: 1,
                                    name: "white_balance_g",
                                    t: U16(0),
                                },
                                OffsetItem {
                                    offset: 3,
                                    name: "white_balance_b",
                                    t: U16(0),
                                },
                            ],
                        },
                        TagItem {
                            tag: 30847,
                            name: "legacy_white_level",
                            len: None,
                            is_optional: false,
                            is_value_u16: false,
                        },
                        Jump {
                            tag: 30847,
                            is_optional: false,
                            tasks: vec![OffsetItem {
                                offset: 0,
                                name: "white_level",
                                t: U16(0),
                            }],
                        },
                    ],
                )],
            }],
            right: vec![Offset(
                Bytes(32),
                vec![Offset(
                    Bytes(0),
                    vec![
                        Jump {
                            tag: 29456,
                            is_optional: false,
                            tasks: vec![OffsetItem {
                                offset: 0,
                                name: "black_level",
                                t: U16(0),
                            }],
                        },
                        Jump {
                            tag: 29458,
                            is_optional: false,
                            tasks: vec![
                                OffsetItem {
                                    offset: 0,
                                    name: "white_balance_r",
                                    t: U16(0),
                                },
                                OffsetItem {
                                    offset: 1,
                                    name: "white_balance_g",
                                    t: U16(0),
                                },
                                OffsetItem {
                                    offset: 3,
                                    name: "white_balance_b",
                                    t: U16(0),
                                },
                            ],
                        },
                        TagItem {
                            tag: 30847,
                            name: "legacy_white_level",
                            len: None,
                            is_optional: false,
                            is_value_u16: false,
                        },
                        Jump {
                            tag: 30847,
                            is_optional: false,
                            tasks: vec![OffsetItem {
                                offset: 0,
                                name: "white_level",
                                t: U16(0),
                            }],
                        },
                    ],
                )],
            )],
        },
    ]);

    assert_eq!(task, answer);
}

#[test]
fn test_describe_rules() {
    let task = quickexif::describe_rule!(tiff {
        0x010f {
            str + 0 / make
            u32 + 1 / cfa
            u16 + 0 / cfa
            r64 + 0 / cfa
        }

        0x0112 : u16 / orientation3

        0x0096? / linear_table_offset(linear_table_len)

        0x003d? {
            offset + 10 {
                u16 + 0 / black_level
            }
        }

        0xc634 {
            sony_decrypt / 0x7200 / 0x7201 / 0x7221 {
                0x7310 {
                    u16 + 0 / black_level
                }
                0x7312 {
                    u16 + 0 / white_balance_r
                    u16 + 1 / white_balance_g
                    u16 + 3 / white_balance_b
                }
                0x787f / legacy_white_level {
                    u16 + 0 / white_level
                }
            }
        }

        offset - 4 {
            0x0535 / bps
        }

        0x0124 / some_address
        offset + some_address {
            0x0611 / bps
        }

        0x014a {
            offset + 4 {
                offset address {
                    0x0100 / width
                    0x0103 : u16 / compression
                }
            }
        }

        offset + 8 {
            scan [0x49, 0x49, 0x2a, 0x00] / tiff_offset {
                tiff {
                    0xf000 {
                        0xf001 / width
                        0xf00a {
                            u32 + 0 / black_level
                        }
                        0xf00d {
                            u32 + 0 / white_balance_g
                        }
                    }
                }
            }
        }

        0x1253 / bar
        if bar < 1
        {
            0x4254 : u16 / bar_level
        }
        else
        {
            0x4254 {
                u16 + 0 / bar_level
            }
        }

        0x4253 / foo
        if foo > 1
        {
            0x4254 : u16 / foo_level
        }
        else
        {
            0x4254 {
                u16 + 0 / foo_level
            }
        }

        0xc61d / wl(white_level_len)
        if white_level_len == 1
        {
            0xc61d : u16 / white_level
        }
        else
        {
            0xc61d {
                u16 + 0 / white_level
            }
        }

        0x0111? / strip
        if strip ?
        {
            0x0117 / strip_len
        }
        else
        {
            0x0144 / tile_offsets
            0x0142 / tile_width
            0x0143 / tile_len
        }

        next {
            0x0201 / thumbnail
            0x0202 / thumbnail_len
        }

        0x002e {
            offset + 12 {
                tiff {
                    0x8769 {
                        0x927c {
                            offset + 12 {
                                0x004b / cropped_width
                                0x004c / cropped_height
                            }
                        }
                    }
                }
            }
        }
    });

    let answer = Tiff(vec![
        Jump {
            tag: 271,
            is_optional: false,
            tasks: vec![
                OffsetItem {
                    offset: 0,
                    name: "make",
                    t: Str("".to_owned()),
                },
                OffsetItem {
                    offset: 1,
                    name: "cfa",
                    t: U32(0),
                },
                OffsetItem {
                    offset: 0,
                    name: "cfa",
                    t: U16(0),
                },
                OffsetItem {
                    offset: 0,
                    name: "cfa",
                    t: R64(0.0),
                },
            ],
        },
        TagItem {
            tag: 274,
            name: "orientation3",
            len: None,
            is_optional: false,
            is_value_u16: true,
        },
        TagItem {
            tag: 150,
            name: "linear_table_offset",
            len: Some("linear_table_len"),
            is_optional: true,
            is_value_u16: false,
        },
        Jump {
            tag: 61,
            is_optional: true,
            tasks: vec![Offset(
                Bytes(10),
                vec![OffsetItem {
                    offset: 0,
                    name: "black_level",
                    t: U16(0),
                }],
            )],
        },
        Jump {
            tag: 50740,
            is_optional: false,
            tasks: vec![SonyDecrypt {
                offset_tag: 29184,
                len_tag: 29185,
                key_tag: 29217,
                tasks: vec![
                    Jump {
                        tag: 29456,
                        is_optional: false,
                        tasks: vec![OffsetItem {
                            offset: 0,
                            name: "black_level",
                            t: U16(0),
                        }],
                    },
                    Jump {
                        tag: 29458,
                        is_optional: false,
                        tasks: vec![
                            OffsetItem {
                                offset: 0,
                                name: "white_balance_r",
                                t: U16(0),
                            },
                            OffsetItem {
                                offset: 1,
                                name: "white_balance_g",
                                t: U16(0),
                            },
                            OffsetItem {
                                offset: 3,
                                name: "white_balance_b",
                                t: U16(0),
                            },
                        ],
                    },
                    TagItem {
                        tag: 30847,
                        name: "legacy_white_level",
                        len: None,
                        is_optional: false,
                        is_value_u16: false,
                    },
                    Jump {
                        tag: 30847,
                        is_optional: false,
                        tasks: vec![OffsetItem {
                            offset: 0,
                            name: "white_level",
                            t: U16(0),
                        }],
                    },
                ],
            }],
        },
        Offset(
            Bytes(-4),
            vec![TagItem {
                tag: 1333,
                name: "bps",
                len: None,
                is_optional: false,
                is_value_u16: false,
            }],
        ),
        TagItem {
            tag: 292,
            name: "some_address",
            len: None,
            is_optional: false,
            is_value_u16: false,
        },
        Offset(
            PrevField("some_address"),
            vec![TagItem {
                tag: 1553,
                name: "bps",
                len: None,
                is_optional: false,
                is_value_u16: false,
            }],
        ),
        Jump {
            tag: 330,
            is_optional: false,
            tasks: vec![Offset(
                Bytes(4),
                vec![Offset(
                    Address,
                    vec![
                        TagItem {
                            tag: 256,
                            name: "width",
                            len: None,
                            is_optional: false,
                            is_value_u16: false,
                        },
                        TagItem {
                            tag: 259,
                            name: "compression",
                            len: None,
                            is_optional: false,
                            is_value_u16: true,
                        },
                    ],
                )],
            )],
        },
        Offset(
            Bytes(8),
            vec![Scan {
                marker: &[73, 73, 42, 0],
                name: Some("tiff_offset"),
                tasks: vec![Tiff(vec![Jump {
                    tag: 61440,
                    is_optional: false,
                    tasks: vec![
                        TagItem {
                            tag: 61441,
                            name: "width",
                            len: None,
                            is_optional: false,
                            is_value_u16: false,
                        },
                        Jump {
                            tag: 61450,
                            is_optional: false,
                            tasks: vec![OffsetItem {
                                offset: 0,
                                name: "black_level",
                                t: U32(0),
                            }],
                        },
                        Jump {
                            tag: 61453,
                            is_optional: false,
                            tasks: vec![OffsetItem {
                                offset: 0,
                                name: "white_balance_g",
                                t: U32(0),
                            }],
                        },
                    ],
                }])],
            }],
        ),
        TagItem {
            tag: 4691,
            name: "bar",
            len: None,
            is_optional: false,
            is_value_u16: false,
        },
        Condition {
            cond: (LT, "bar", 1),
            left: vec![TagItem {
                tag: 16980,
                name: "bar_level",
                len: None,
                is_optional: false,
                is_value_u16: true,
            }],
            right: vec![Jump {
                tag: 16980,
                is_optional: false,
                tasks: vec![OffsetItem {
                    offset: 0,
                    name: "bar_level",
                    t: U16(0),
                }],
            }],
        },
        TagItem {
            tag: 16979,
            name: "foo",
            len: None,
            is_optional: false,
            is_value_u16: false,
        },
        Condition {
            cond: (GT, "foo", 1),
            left: vec![TagItem {
                tag: 16980,
                name: "foo_level",
                len: None,
                is_optional: false,
                is_value_u16: true,
            }],
            right: vec![Jump {
                tag: 16980,
                is_optional: false,
                tasks: vec![OffsetItem {
                    offset: 0,
                    name: "foo_level",
                    t: U16(0),
                }],
            }],
        },
        TagItem {
            tag: 50717,
            name: "wl",
            len: Some("white_level_len"),
            is_optional: false,
            is_value_u16: false,
        },
        Condition {
            cond: (EQ, "white_level_len", 1),
            left: vec![TagItem {
                tag: 50717,
                name: "white_level",
                len: None,
                is_optional: false,
                is_value_u16: true,
            }],
            right: vec![Jump {
                tag: 50717,
                is_optional: false,
                tasks: vec![OffsetItem {
                    offset: 0,
                    name: "white_level",
                    t: U16(0),
                }],
            }],
        },
        TagItem {
            tag: 273,
            name: "strip",
            len: None,
            is_optional: true,
            is_value_u16: false,
        },
        Condition {
            cond: (EXIST, "strip", 0),
            left: vec![TagItem {
                tag: 279,
                name: "strip_len",
                len: None,
                is_optional: false,
                is_value_u16: false,
            }],
            right: vec![
                TagItem {
                    tag: 324,
                    name: "tile_offsets",
                    len: None,
                    is_optional: false,
                    is_value_u16: false,
                },
                TagItem {
                    tag: 322,
                    name: "tile_width",
                    len: None,
                    is_optional: false,
                    is_value_u16: false,
                },
                TagItem {
                    tag: 323,
                    name: "tile_len",
                    len: None,
                    is_optional: false,
                    is_value_u16: false,
                },
            ],
        },
        JumpNext(vec![
            TagItem {
                tag: 513,
                name: "thumbnail",
                len: None,
                is_optional: false,
                is_value_u16: false,
            },
            TagItem {
                tag: 514,
                name: "thumbnail_len",
                len: None,
                is_optional: false,
                is_value_u16: false,
            },
        ]),
        Jump {
            tag: 46,
            is_optional: false,
            tasks: vec![Offset(
                Bytes(12),
                vec![Tiff(vec![Jump {
                    tag: 34665,
                    is_optional: false,
                    tasks: vec![Jump {
                        tag: 37500,
                        is_optional: false,
                        tasks: vec![Offset(
                            Bytes(12),
                            vec![
                                TagItem {
                                    tag: 75,
                                    name: "cropped_width",
                                    len: None,
                                    is_optional: false,
                                    is_value_u16: false,
                                },
                                TagItem {
                                    tag: 76,
                                    name: "cropped_height",
                                    len: None,
                                    is_optional: false,
                                    is_value_u16: false,
                                },
                            ],
                        )],
                    }],
                }])],
            )],
        },
    ]);

    assert_eq!(task, answer);
}
