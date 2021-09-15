<!-- omit in TOC -->

# Simple Struct Descriptor

[![Crates.io](https://img.shields.io/crates/v/descriptor?style=flat-square)](https://crates.io/crates/descriptor)
[![Crates.io](https://img.shields.io/crates/d/descriptor?style=flat-square)](https://crates.io/crates/descriptor)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/XciD/descriptor/blob/master/LICENSE)

Easy pretty print your Rust struct into single element or table

## Example

```rust
use descriptor::{object_describe_to_string, table_describe_to_string, Descriptor};

#[derive(Descriptor)]
struct User {
    name: String,
    age: i32,
    address: Address,
}

#[derive(Descriptor)]
struct Address {
    street: String,
    town: String,
}

fn main() {
    let user1 = User {
        name: "Adrien".to_string(),
        age: 32,
        address: Address {
            street: "Main street".to_string(),
            town: "NY".to_string()
        }
    };
    let user2 = User {
        name: "Corentin".to_string(),
        age: 40,
        address: Address {
            street: "10 rue de la paix".to_string(),
            town: "Paris".to_string()
        }
    };
    let description = object_describe_to_string(&user1).unwrap();

    assert_eq!(r#"
     Name:    Adrien
     Age:     32
     Address:
       Street: Main street
       Town:   NY
     "#, description);

    let table = table_describe_to_string(&vec![user1, user2]).unwrap();

    assert_eq!(r#"
     NAME     AGE ADDRESS.STREET    ADDRESS.TOWN
     Adrien   32  Main street       NY
     Corentin 40  10 rue de la paix Paris
     "#, format!("\n{}", table));
}
```
