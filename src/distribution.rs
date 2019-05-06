use std::process::Command;

use Distribution::*;

pub enum Distribution {
    Arch,
}

pub fn current() -> Distribution {
    let uname_a = String::from_utf8(
        Command::new("/usr/bin/uname")
            .arg("-a")
            .output()
            .map(|output| output.stdout)
            .expect("Something went wrong determining the distribution (uname)"),
    )
    .expect("Something went wrong reading the output of uname");
    if uname_a.contains("ARCH") {
        Arch
    } else {
        eprintln!("It looks like you're running an as yet unsupported distribution.");
        eprintln!("It turns out adding your distribution is easy!");
        eprintln!("https://github.com/RatanRSur/briefing/blob/master/src/package.rs");
        std::process::exit(1);
    }
}
