use lazy_static::lazy_static;
use std::collections::HashMap;

macro_rules! github_releases {
    ($organization:literal, $project:literal, $version_prefix:literal) => {
        (
            $project,
            concat!(
                "https://github.com/",
                $organization,
                "/",
                $project,
                "/releases/tag/",
                $version_prefix,
                "{version}"
            ),
        )
    };
}

lazy_static! {
    /// URL's for projects who have version specific changelog URL's
    pub static ref TEMPLATES: HashMap<&'static str, &'static str> = [
        github_releases!("jwilm", "alacritty", "v"),
        github_releases!("hyperledger", "besu", ""),
        ("chromium", "https://chromium.googlesource.com/chromium/src/+log/{old_version}..{version}?pretty=fuller&n=10000"),
        github_releases!("FedeDP", "Clight", "v"),
        ("feh", "https://feh.finalrewind.org/archive/{version}/"),
        ("firefox", "https://www.mozilla.org/en-US/firefox/{version}/releasenotes/"),
        github_releases!("flatpak", "flatpak", ""),
        ("ghc", "https://downloads.haskell.org/~ghc/{version}/docs/html/users_guide/{version}-notes.html"),
        ("git", "https://raw.githubusercontent.com/git/git/master/Documentation/RelNotes/{version}.txt"),
        ("github-cli", "https://github.com/cli/cli/releases/tag/v{version}"),
        ("gimp", "https://www.gimp.org/release-notes/gimp-{version}.html"),
        ("go", "https://golang.org/doc/devel/release.html#go{version}"),
        github_releases!("jarun", "googler", ""),
        ("i3-gaps", "https://github.com/Airblader/i3/blob/gaps-next/RELEASE-NOTES-{version}"),
        ("linux", "https://cdn.kernel.org/pub/linux/kernel/v5.x/ChangeLog-{version}"),
        github_releases!("neovim", "neovim", "v"),
        ("racket", "https://download.racket-lang.org/v{version}.html"),
        github_releases!("BurntSushi","ripgrep", ""),
        github_releases!("r-darwish","topgrade", "v"),
        ("tmux", "https://raw.githubusercontent.com/tmux/tmux/{version}/CHANGES"),
        github_releases!("vim", "vim", "v"),
        github_releases!("Jguer", "yay", "v"),
        github_releases!("rg3", "youtube-dl", ""),
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
         ("vlc", "https://www.videolan.org/developers/vlc-branch/NEWS"),
         ("yourkit", "https://www.yourkit.com/changes/"),
         ("zoom", "https://support.zoom.us/hc/en-us/sections/201214205-Release-Notes")]
            .iter()
            .cloned()
            .collect();
}
