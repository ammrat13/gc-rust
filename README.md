# GC in Rust

This repository was inspired by a lab from [CS 240LX][1]. It creates a way to
create references to the heap that can be garbage collected. It's very much a
demo. The collection is not efficient at all, and it only works for a single
thread. The tests are also very brittle - they have to be run at a time and
without optimizations to avoid chunks as being considered reachable even though
they are not.

[1]: https://github.com/dddrrreee/cs240lx-24spr/tree/main/labs/6-malloc%2Bgc
