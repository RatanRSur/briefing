### briefing
Your feature dossier.

`cargo run`


### Run it as a pacman hook to have easy access to release notes
`/etc/pacman.d/hooks/briefing.hook`
```
[Trigger]
Operation = Upgrade
Operation = Install
Operation = Remove
Type = Package
Target = *

[Action]
Description = What's new?
When = PostTransaction
Exec = </path/to/briefing/here>
```
