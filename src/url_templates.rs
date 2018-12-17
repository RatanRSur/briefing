use std::collections::HashMap;
use strfmt::strfmt;

use crate::Upgrade;

pub static RELEASE_NOTES_TEMPLATES: &'static [(&str, &str)] = &[
    (
        "alacritty",
        "https://github.com/jwilm/alacritty/releases/tag/v{version}",
    ),
    ("feh", "https://feh.finalrewind.org/archive/{version}/"),
    (
        "firefox",
        "https://www.mozilla.org/en-US/firefox/{version}/releasenotes/",
    ),
    (
        "flatpak",
        "https://github.com/flatpak/flatpak/releases/tag/{version}",
    ),
    (
        "git",
        "https://raw.githubusercontent.com/git/git/master/Documentation/RelNotes/{version}.txt",
    ),
    (
        "linux",
        "https://cdn.kernel.org/pub/linux/kernel/v4.x/ChangeLog-{version}",
    ),
    (
        "neovim",
        "https://github.com/neovim/neovim/releases/tag/v{version}",
    ),
    ("vim", "https://github.com/vim/vim/releases/tag/v{version}"),
    (
        "youtube-dl",
        "https://github.com/rg3/youtube-dl/releases/tag/{version}",
    ),
];

pub fn format_url(template: &String, version: &String) -> String {
    let format_args: HashMap<String, &String> = [(String::from("version"), version)]
        .iter()
        .cloned()
        .collect();

    strfmt(template, &format_args).unwrap()
}
