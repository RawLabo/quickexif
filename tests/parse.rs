use std::fs;

#[test]
fn parse_arw() {
    let sample = "tests/samples/sample0.ARW";

    quickexif::parse_exif(sample);
}


#[test]
fn parse_jpg() {
    let sample = "tests/samples/sample0.JPG";

    quickexif::parse_exif(sample);
}
