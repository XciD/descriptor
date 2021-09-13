<!-- omit in TOC -->
# Simple Struct Descriptor

[![Crates.io](https://img.shields.io/crates/v/descriptor?style=flat-square)](https://crates.io/crates/descriptor)
[![Crates.io](https://img.shields.io/crates/d/descriptor?style=flat-square)](https://crates.io/crates/descriptor)
[![License](https://img.shields.io/badge/license-Apache%202.0-blue?style=flat-square)](https://github.com/XciD/descriptor/blob/master/LICENSE)

Easy pretty print your Rust struct into single element or list

## TLDR

```rust
use descriptor::{object_describe_to_string, Descriptor};

#[derive(Descriptor)]
struct Foo {
    first_field: String,
    int_field: i32,
}

fn main() {
    println!("{}", object_describe_to_string(Foo{first_field: "foo", int_field: 1}).unwrap());
}
```

Will print or simple description:
```
First Field: foo
Int Field:   1
```
Or for table:
```
FIRST_FIELD INT_FIELD
foo         1
```

The `#[derive(Descriptor)]` will auto implement the `Describe` Trait for the Struct

## Struct Attributes:

`#[descriptor(into = AnotherStruct)]`

Will use another struct to describe the struct.
The `From` or `Into` Trait should be implemented and `AnotherStruct` should implement the `Describe` trait

`#[descriptor(headers = &["HEADER_1"])]`

Will override the default generated header list

`#[descriptor(map = some_func)]`

A function hook that act on the entire row with the struct as context.
The function need to return a `String`
```
fn some_func(contextual_struct: &A, cell: String) -> String {
    match contextual_struct.is_selected {
        true => cell.green(),
        false => cell.reset(),
    }
    .to_string()
}
```

## Field Attributes

`#[descriptor(skip)]`

Will skip this field on any context

`#[descriptor(skip_header)]`

Will skip this field on the auto-generated default headers list

`#[descriptor(skip_description)]`

Will skip this field on the description context

`#[descriptor(output_table)]`

On a description context, will display a list into a table, example:

```

#[derive(Descriptor)]
struct A {
    #[descriptor(output_table)]
    history: Vec<AnotherStruct>,
}

History:
  STATE   VALUE
  test    value
```

`#[descriptor(resolve_option)]`

Should we resolve option, usefull with the `map` attribute

`#[descriptor(into = AnotherStruct)]`

Same as `into` in the Struct Level but for a specific Field

`#[descriptor(map=to_string)]`

Use a function in order to transform the value, function will receive the field by reference.
Assume that the return function is a `scalar type`.

`#[descriptor(map=to_string, method)]`

Use a method instead of a public function.

`#[descriptor(map=to_another_struct, method, into=AnotherStruct)]`

Use a method instead of `From` to go on another struct.

`#[descriptor(rename_header="HEADER_NAME")]`

Rename a header field

`#[descriptor(rename_description="Another Description")]`

Rename a description value for Enum.
TODO(aca): Implement this for description field ?

`#[descriptor(flatten)]`

Flatten a struct into another one

`#[descriptor(additional_struct = ComputedStruct)]`

Add another other field into the struct. `ComputedStruct` must impl the `From` Trait
