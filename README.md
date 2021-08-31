# Blog OS

### Relevant Branches
- `main` branch is Basically the newest version of the blog
- `uefi-support` is my current state of porting everything to the newest version
of the bootloader crate, which enables UEFI.

Currently this only produces a grey screen.
As soon as Text output works i fill try to get the missing stuff for allocation
etc back running again. I already fixed all type errors resulting from
bootloader deprecating `memory_map`, but not sure yet if stuff works.

### Notice

The Licenses of the original project are preserved in the licenses subfolder.

Any changes this project makes are licensed under the MIT license which you
can take a look at via the LICENSE.md file.
