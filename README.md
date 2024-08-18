Arkive
======

This is the lower-level library that powers Dirtabase. The family tree is ultimately going to look like this:

```
  Arkive   SBox360 (sandbox build tool)
     \      /
     Dirtabase
         |
      Layover
         |
    Layover Linux
```

Providing immutable directories (Arkive), immutable and reproducible software builds (Dirtabase), atomic/pure package management (Layover), and a cohesive operating system respectively.

Arkive exists to be:

 1. A high-performance datastructure for representing a directory tree in memory, and
 2. A bunch of features for that datastructure to make it useful.

Because it can associate any Rust type as per-file contents, it's extremely flexible, and allows you to do extremely fast directory manipulation on underpowered hardware. There are few limits to what you can do using `arkive::Ark` directly. The other side of the coin, though, is that compared to something that _wraps_ Arkive (like Dirtabase), there's some sharp edges and complexity exposed in Arkive that isn't exposed by wrapper crates.

Arkive isn't hard to use, per se, but it's an atypical representation of directory trees in multiple ways, and requires a bit of getting your head around sometimes. Basic use is very simple though!

```rust
use arkive::*;

// Gets you an Ark<std::path::PathBuf>, where each file is represented by a
// PathBuf pointing to where it lives on disk.
let scanned = Ark::scan("./src").expect("Failed to read directory structure");

// Expands an archive to disk, like `tar x`.
//
// In this case, since the content type is PathBuf, the underlying per-file
// write behavior is std::fs::copy() from the original file to the new destination.
scanned.write("./copy_of_src").expect("Failed to copy files");

// Or that whole example, concisely with ?:
Ark::scan("./src")?.write("./copy_of_src")?;
```

## License

Arkive is a GPL-3 codebase.
