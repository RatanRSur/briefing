use ansi_term::Style;
use std::collections::HashMap;
use strfmt::strfmt;

pub fn package_output(margin_width: usize, package_name: &str, urls: &Vec<String>) -> String {
    let mut buf = String::new();
    buf.push_str(&bold(left_pad_to_width(margin_width, package_name)));
    for (i, url) in urls.iter().enumerate() {
        if i != 0 {
            buf.push_str(&left_pad_to_width(margin_width, ""));
        }
        buf.push(' ');
        buf.push_str(url);
        buf.push('\n');
    }
    buf
}

pub fn format_url(template: &str, old_version: &str, new_version: &str) -> String {
    let format_args: HashMap<String, &str> = [
        ("version".to_string(), new_version),
        ("old_version".to_string(), old_version),
    ]
    .iter()
    .cloned()
    .collect();

    strfmt(template, &format_args).unwrap()
}

pub fn section_header(margin_width: usize, header: &str) -> String {
    let mut buf = String::new();
    buf.push_str(&left_pad_to_width(margin_width, ""));
    buf.push(' ');
    buf.push_str(header);
    bold(buf)
}

fn left_pad_to_width(width: usize, str: &str) -> String {
    let mut buf = String::new();
    (0..(width - str.len())).for_each(|_| buf.push(' '));
    buf.push_str(str);
    buf
}

fn bold(str: String) -> String {
    Style::new().bold().paint(str).to_string()
}
