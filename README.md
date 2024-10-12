## Arbutus

Arbutus is a tree data structure library for Rust.

### Overview

Arbutus provides a high-level API for constructing and manipulating trees, along with support for indexing and querying. The library focuses on simplicity, flexibility, and performance.

### Features

*   **Tree Construction**: Build trees using the `TreeBuilder` API, which provides a composable way to construct tree structures.
*   **Indexing**: Utilize B-Tree indices for efficient querying and retrieval of node data.
*   **Iterators**: Traverse trees using iterators, which support depth-first or breadth-first traversal.

### Getting Started

To get started with Arbutus, add the following dependency to your `Cargo.toml` file:

```toml
[dependencies]
arbutus = "0.1.0"
```

Then, import the library in your Rust code using:

```rust
use arbutus::{Tree, Node};
```

### License

Arbutus is released under the MIT license. See the [LICENSE](LICENSE.txt) file for details.
