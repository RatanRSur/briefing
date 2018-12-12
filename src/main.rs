use std::io::{self, BufReader};
use std::io::prelude::*;
use std::fs::File;

fn is_upgrade(s: &str) -> bool {
    s.contains("upgraded")
}

fn main() -> io::Result<()> {
    let f = BufReader::new(File::open("/var/log/pacman.log")?);

    f.lines()
        .filter_map(|result_str| result_str.ok().filter(|s| is_upgrade(&s)))
        .for_each(|s| println!("{}", s));

    Ok(())
}
