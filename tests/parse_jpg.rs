use std::fs;

#[test]
fn parse_jpg() {
    let sample = fs::read("tests/samples/sample0.JPG").unwrap();

    // the JPEG header will automatically be removed
    let rule = quickexif::describe_rule!(tiff {
        0x010f {
            str + 0 / make
        }
        0x0110 {
            str + 0 / model
        }
        0x8769 {
            0x8827 : u16 / iso
            0x829a {
                r64 + 0 / exposure_time
            }
            0x829d {
                r64 + 0 / f_number
            }
            0x9004 {
                str + 0 / create_date
            }
            0x920a {
                r64 + 0 / focal_length
            }
            0xa002 / width
            0xa003 / height
        }
    });

    let parsed_info = quickexif::parser::Parser::parse(&sample, &rule).unwrap();

    let make = parsed_info.str("make").unwrap();
    let model = parsed_info.str("model").unwrap();
    let create_date = parsed_info.str("create_date").unwrap();
    let iso = parsed_info.u16("iso").unwrap();
    let focal_length = parsed_info.f64("focal_length").unwrap();
    let exposure_time = parsed_info.f64("exposure_time").unwrap();
    let f_number = parsed_info.f64("f_number").unwrap();
    let width = parsed_info.u32("width").unwrap();
    let height = parsed_info.u32("height").unwrap();

    let answer = "SONY ILCE-7C 2022:06:04 10:14:14 160 35 0.03333333333333333 5 6000 4000";
    assert_eq!(format!("{make} {model} {create_date} {iso} {focal_length} {exposure_time} {f_number} {width} {height}"), answer);
}
