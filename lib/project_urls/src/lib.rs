use lazy_static::lazy_static;
use std::collections::HashMap;

lazy_static! {
    /// URL's for projects who have version specific changelog URL's
    pub static ref TEMPLATES: HashMap<&'static str, &'static str> = [
        ("alacritty", "https://github.com/jwilm/alacritty/releases/tag/v{version}",),
        ("chromium", "https://chromium.googlesource.com/chromium/src/+log/{old_version}..{version}?pretty=fuller&n=10000"),
        ("feh", "https://feh.finalrewind.org/archive/{version}/"),
        ("firefox", "https://www.mozilla.org/en-US/firefox/{version}/releasenotes/",),
        ("flatpak", "https://github.com/flatpak/flatpak/releases/tag/{version}",),
        ("ghc", "https://downloads.haskell.org/~ghc/{version}/docs/html/users_guide/{version}-notes.html"),
        ("git", "https://raw.githubusercontent.com/git/git/master/Documentation/RelNotes/{version}.txt",),
        ("gimp", "https://www.gimp.org/release-notes/gimp-{version}.html"),
        ("go", "https://golang.org/doc/devel/release.html#go{version}"),
        ("googler", "https://github.com/jarun/googler/releases/tag/v{version}"),
        ("i3-gaps", "https://github.com/Airblader/i3/blob/gaps-next/RELEASE-NOTES-{version}"),
        ("linux", "https://cdn.kernel.org/pub/linux/kernel/v5.x/ChangeLog-{version}",),
        ("neovim", "https://github.com/neovim/neovim/releases/tag/v{version}",),
        ("racket", "https://download.racket-lang.org/v{version}.html"),
        ("tmux", "https://raw.githubusercontent.com/tmux/tmux/{version}/CHANGES"),
        ("vim", "https://github.com/vim/vim/releases/tag/v{version}"),
        ("yay", "https://github.com/Jguer/yay/releases/tag/v{version}"),
        ("youtube-dl", "https://github.com/rg3/youtube-dl/releases/tag/{version}",),
    ]
    .iter()
    .cloned()
    .collect();
    /// URL's for projects who have a single page with updated changelogs
    pub static ref MONO_PAGES: HashMap<&'static str, &'static str> =
        [("arduino", "https://www.arduino.cc/en/Main/ReleaseNotes"),
         ("google-chrome", "https://chromereleases.googleblog.com"),
         ("intellij-idea-community-edition", "https://www.jetbrains.com/idea/whatsnew/"),
         ("networkmanager", "https://gitlab.freedesktop.org/NetworkManager/NetworkManager/blob/master/NEWS"),
         ("virtualbox", "https://www.virtualbox.org/wiki/Changelog"),
         ("vlc", "https://www.videolan.org/developers/vlc-branch/NEWS")]
            .iter()
            .cloned()
            .collect();
}
