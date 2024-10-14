## Arbutus

Arbutus is a tree data structure library for Rust.

### Overview

Arbutus provides a high-level API for constructing and manipulating trees, along with support for indexing and querying. The library focuses on simplicity, flexibility, and performance.

### Features

*   **Tree Construction**: Build trees using the `TreeBuilder` API, which provides a composable way to construct tree structures.
*   **Indexing**: Utilize B-Tree indices for efficient querying and retrieval of node data.
*   **Iterators**: Traverse trees using iterators

### Getting Started

To get started with Arbutus, add the following dependency to your `Cargo.toml` file:

```toml
[dependencies]
arbutus = "0.1.0"
```

Example of building a tree

```rust
// Custom errors can be propagated through the builder closures
#[derive(Debug)]
enum MyError {
    Fail(String),
}

#[derive(Debug)]
enum TestData {
    Foo,
    Bar,
    String(String),
    Baz,
}

let tree = TreeBuilder::<TestData, MyError>::new()
    .root(TestData::Foo, |foo| {
        debug!("Foo builder closure");

        foo.child(TestData::Bar, |bar| {
            debug!("Bar builder closure");
            bar.child(TestData::Baz, |_| Ok(()))
        })?;

        foo.child(TestData::String("Hello".into()), |_| Ok(()))?;

        Ok(())
    })
    .unwrap()
    .done();
info!("{tree:#?}");
```

### License

Arbutus is released under the MIT license. See the [LICENSE](LICENSE.txt) file for details.
