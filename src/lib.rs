//! Easy pretty print your Rust struct into single element or list
//!
//! # Simple Example
//! ```
//! use descriptor::{object_describe_to_string, table_describe_to_string, Descriptor};
//!
//! #[derive(Descriptor)]
//! struct User {
//!     name: String,
//!     age: i32,
//!     address: Address,
//! }
//!
//! #[derive(Descriptor)]
//! struct Address {
//!     street: String,
//!     town: String,
//! }
//!
//! let user1 = User{
//!     name: "Adrien".to_string(),
//!     age: 32,
//!     address: Address{
//!         street: "Main street".to_string(),
//!         town: "NY".to_string()
//!     }
//! };
//! let user2 = User{
//!     name: "Corentin".to_string(),
//!     age: 40,
//!     address: Address{
//!         street: "10 rue de la paix".to_string(),
//!         town: "Paris".to_string()
//!     }
//! };
//! let description = object_describe_to_string(&user1).unwrap();
//!
//! assert_eq!(r#"
//! Name:    Adrien
//! Age:     32
//! Address:
//!   Street: Main street
//!   Town:   NY
//! "#,  description);
//!
//! let table = table_describe_to_string(&vec![user1, user2]).unwrap();
//!
//! assert_eq!(r#"
//! NAME     AGE ADDRESS.STREET    ADDRESS.TOWN
//! Adrien   32  Main street       NY
//! Corentin 40  10 rue de la paix Paris
//! "#, format!("\n{}", table));
//! ```
//!
//! # Macro attributes
//! ## Struct attributes
//!
//! #### `#[descriptor(flatten)]`
//! Flatten a struct into another.
//!
//! ```
//! use descriptor::{Descriptor, object_describe_to_string};
//!
//! #[derive(Descriptor)]
//! struct User {
//!     name: String,
//!     age: i32,
//!     #[descriptor(flatten)]
//!     address: Address
//! }
//!
//! #[derive(Descriptor)]
//! struct Address {
//!     street: String,
//!     town: String,
//! }
//!
//! let foo = User{
//!     name: "Adrien".to_string(),
//!     age: 32,
//!     address: Address{
//!         street: "Main street".to_string(),
//!         town: "NY".to_string()
//!     }
//! };
//! let description = object_describe_to_string(&foo).unwrap();
//!
//! assert_eq!(r#"
//! Name:    Adrien
//! Age:     32
//! Street:  Main street
//! Town:    NY
//! "#,  description);
//! ```
//!
//! ### `#[descriptor(into = AnotherStruct)]`
//! The `into` parameter convert the struct into another before describe.
//!
//! The `From` or `Into` Trait should be implemented and `AnotherStruct` should implement the `Describe` trait
//! ```
//! use descriptor::{Descriptor, object_describe_to_string};
//!
//! #[derive(Descriptor)]
//! pub struct ProgressDescribe {
//!     pub transfer: String,
//! }
//!
//! impl From<&Progress> for ProgressDescribe {
//!     fn from(progress: &Progress) -> Self {
//!         let bar_length = 20;
//!         let pad_l = ((progress.processed * bar_length) / progress.total.max(1)) as usize;
//!         let bar = format!(
//!             "[{:=>pad_l$}>{:>pad_r$}] {}/{}",
//!             "",
//!             "",
//!             progress.processed,
//!             progress.total,
//!             pad_l = pad_l,
//!             pad_r = bar_length as usize - pad_l
//!         );
//!
//!         Self {
//!             transfer: bar,
//!         }
//!     }
//! }
//!
//! #[derive(Descriptor)]
//! #[descriptor(into = ProgressDescribe)]
//! struct Progress {
//!     pub processed: u64,
//!     pub total: u64,
//! }
//!
//! let progress = Progress{
//!    processed: 20,
//!    total: 40,
//! };
//! let description = object_describe_to_string(&progress).unwrap();
//!
//! assert_eq!(r#"
//! Transfer: [==========>          ] 20/40
//! "#,  description);
//! ```
//!
//! ### `#[descriptor(extra_fields = ExtraStruct)]`
//!
//! Add fields from another struct into the description, useful for computed values without overriding real values.
//!
//! ```
//! use descriptor::{Descriptor, object_describe_to_string};
//!
//! #[derive(Descriptor)]
//! #[descriptor(extra_fields = AgeEntity)]
//! struct User {
//!     name: String,
//!     created_at: i32,
//! }
//!
//! #[derive(Descriptor)]
//! pub struct AgeEntity {
//!     pub age: String,
//! }
//!
//! impl From<&User> for AgeEntity {
//!     fn from(u: &User) -> Self {
//!         Self {
//!             age: format!("{} days ago", u.created_at),
//!         }
//!     }
//! }
//!
//! let progress = User{
//!    name: "Adrien".to_string(),
//!    created_at: 40,
//! };
//! let description = object_describe_to_string(&progress).unwrap();
//! assert_eq!(r#"
//! Name:       Adrien
//! Created At: 40
//! Age:        40 days ago
//! "#,  description);
//! ```
//!
//! ### `#[descriptor(default_headers = [""])]`
//!
//! Overrides default headers when using the table output
//! ```
//! use descriptor::{Descriptor, table_describe_to_string};
//! #[derive(Descriptor, Clone)]
//! #[descriptor(default_headers = ["brand", "seat"])]
//! struct Car {
//!     brand: String,
//!     seat: i16,
//!     model: String,
//! }
//! let cars = vec![
//!     Car{brand: "Audi".to_string(), seat:4, model: "A3".to_string()},
//!     Car{brand: "Mercedes".to_string(), seat: 2, model: "GLC".to_string()}
//! ];
//!
//! let description = table_describe_to_string(&cars).unwrap();
//! assert_eq!(r#"
//! BRAND    SEAT
//! Audi     4
//! Mercedes 2
//! "#,  format!("\n{}", description));
//! ```
//!
//! ## Field attributes
//!
//! ### `#[descriptor(map = ident)]`
//! Takes a transformation function as parameter, called before generating the field.
//! Return value of the function should implement the `Describe` Trait
//!
//! ```
//! use descriptor::{Descriptor, object_describe_to_string};
//!
//! fn age_to_string(val: &i32) -> String {
//!   format!("{} years", val)
//! }
//!
//! #[derive(Descriptor)]
//! struct User {
//!     name: String,
//!     #[descriptor(map = age_to_string)]
//!     age: i32,
//! }
//! let foo = User{
//!     name: "Adrien".to_string(),
//!     age: 32,
//! };
//! let description = object_describe_to_string(&foo).unwrap();
//! assert_eq!(r#"
//! Name: Adrien
//! Age:  32 years
//! "#,  description);
//! ```
//! `map` parameter can be used with `resolve_option` parameter.
//! If the field is an Option, it extract it before calling the transformation function
//!
//! ```
//! use descriptor::{Descriptor, object_describe_to_string};
//!
//! fn age_to_string(val: &i32) -> String {
//!   format!("{} years", val)
//! }
//!
//! #[derive(Descriptor)]
//! struct User {
//!     name: String,
//!     #[descriptor(map = age_to_string, resolve_option)]
//!     age: Option<i32>,
//! }
//! let foo = User{
//!     name: "Adrien".to_string(),
//!     age: Option::Some(32),
//! };
//! let description = object_describe_to_string(&foo).unwrap();
//! assert_eq!(r#"
//! Name: Adrien
//! Age:  32 years
//! "#,  description);
//! ```
//! ### `#[descriptor(into)]`
//!
//! Act like `into` parameter in struct level,
//!
//! `into` parameter can be used with `map` and `method` parameter in field level.
//! Sometimes, it's impossible to implement `Into` and `From` for some struct,
//! you can still use a public method from the struct
//! ```
//! use descriptor::{Descriptor, object_describe_to_string};
//! #[derive(Descriptor)]
//! struct Download {
//!     pub filename: String,
//!     #[descriptor(into = String, map = to_string, method)]
//!     pub progress: Progress,
//! }
//!
//! #[derive(Descriptor)]
//! struct Progress {
//!     pub processed: u64,
//!     pub total: u64,
//! }
//! impl Progress {
//!     fn to_string(&self) -> String {
//!         format!("{}/{}", self.processed, self.total)
//!     }
//! }
//! let download = Download{
//!     filename: "debian-11.iso".to_string(),
//!     progress: Progress{ processed: 2, total: 4},
//! };
//!
//! let description = object_describe_to_string(&download).unwrap();
//! assert_eq!(r#"
//! Filename: debian-11.iso
//! Progress: 2/4
//! "#,  description);
//! ```
//! ### `#[descriptor(output_table)]`
//!
//! Output a table-like output inside the description.
//! ```
//! use descriptor::{Descriptor, object_describe_to_string};
//! #[derive(Descriptor, Clone)]
//! struct Car {
//!     name: String,
//!     seat: i16,
//! }
//!
//! #[derive(Descriptor)]
//! struct User {
//!     name: String,
//!     cars: Vec<Car>,
//!     #[descriptor(output_table)]
//!     cars_list: Vec<Car>,
//! }
//!
//! let cars = vec![Car{name: "Audi".to_string(), seat:4}, Car{name: "Mercedes".to_string(), seat: 2}];
//!
//! let user = User{
//!     name: "Adrien".to_string(),
//!     cars: cars.clone(),
//!     cars_list: cars.clone(),
//! };
//! let description = object_describe_to_string(&user).unwrap();
//! assert_eq!(r#"
//! Name:      Adrien
//! Cars:
//! - Name: Audi
//!   Seat: 4
//! - Name: Mercedes
//!   Seat: 2
//! Cars List:
//!   NAME       SEAT
//!   Audi       4
//!   Mercedes   2
//! "#,  description);
//! ```
//! ### `#[descriptor(skip_header)]`
//!
//! Skip this field from default headers
//!
//! ```
//! use descriptor::{Descriptor, object_describe_to_string, table_describe_to_string, table_describe_with_header_to_string};
//! #[derive(Descriptor, Clone)]
//! struct Car {
//!     brand: String,
//!     #[descriptor(skip_header)]
//!     seat: i16,
//!     model: String,
//! }
//! let cars = vec![
//!     Car{brand: "Audi".to_string(), seat:4, model: "A3".to_string()},
//!     Car{brand: "Mercedes".to_string(), seat: 2, model: "GLC".to_string()}
//! ];
//!
//! let description = table_describe_to_string(&cars).unwrap();
//! assert_eq!(r#"
//! BRAND    MODEL
//! Audi     A3
//! Mercedes GLC
//! "#,  format!("\n{}", description));
//!
//! let description = table_describe_with_header_to_string(&cars,&vec!["seat".to_string()] ).unwrap();
//! assert_eq!(r#"
//! SEAT
//! 4
//! 2
//! "#,  format!("\n{}", description));
//! ```
//!
//! ## Enum parameters
//! ### `#[descriptor(rename_description = "Renamed")]`
//!
//! Rename the value for enums
//!
//! ```
//! use descriptor::{object_describe_to_string, Descriptor};
//! #[derive(Descriptor)]
//! struct User {
//!     name: String,
//!     role: Role,
//! }
//! #[derive(Descriptor)]
//! enum Role {
//!     Admin,
//!     #[descriptor(rename_description = "User role")]
//!     User,
//! }
//!
//! let description = object_describe_to_string(&User {
//!    name: "Adrien".to_string(),
//!    role: Role::User,
//! }).unwrap();
//! assert_eq!(r#"
//! Name: Adrien
//! Role: User role
//! "#, description);
//! ```
//!
//!
use std::collections::HashMap;
use std::io;

use chrono::{DateTime, Utc};
use convert_case::{Case, Casing};
#[doc(hidden)]
pub use descriptor_derive::{self, *};

#[derive(Clone, Default)]
pub struct Context {
    pub offset: usize,
    pub pad: usize,
    pub upper_pad: usize,
    pub is_array: bool,
    pub title_size: usize,
}

impl Context {
    pub fn pad(&self, upper_pad: usize) -> Self {
        Self {
            offset: self.offset,
            upper_pad: self.upper_pad.max(upper_pad),
            pad: 0,
            title_size: 0,
            is_array: false,
        }
    }

    pub fn indent(&self, pad: usize, title_size: usize) -> Self {
        Self {
            offset: self.offset + 2,
            pad: pad.max(self.upper_pad),
            upper_pad: 0,
            title_size,
            is_array: false,
        }
    }

    pub fn array(&self) -> Self {
        Self {
            offset: self.offset,
            pad: self.pad,
            title_size: 0,
            upper_pad: 0,
            is_array: true,
        }
    }

    pub fn indent_and_table(&self) -> Self {
        Self {
            offset: self.offset + 2,
            pad: 0,
            upper_pad: 0,
            title_size: 0,
            is_array: true,
        }
    }

    pub fn describe_table<T, W>(&self, data: &[T], writer: &mut W) -> io::Result<()>
    where
        T: Describe,
        W: io::Write,
    {
        writeln!(writer)?;
        Describer::describe_list_internal(data, &[], writer, self.indent_and_table())
    }

    pub fn write_title<W>(&self, writer: &mut W, field: &str, first_field: bool) -> io::Result<()>
    where
        W: io::Write,
    {
        writeln!(writer)?;
        let offset = if first_field && self.is_array {
            write!(writer, "{:<offset$}- ", "", offset = self.offset - 2)?;
            0
        } else {
            self.offset
        };

        write!(
            writer,
            "{:<offset$}{}",
            "",
            format!("{}:", field),
            offset = offset
        )
    }

    pub fn write_value<W>(&self, writer: &mut W, field: String) -> io::Result<()>
    where
        W: io::Write,
    {
        if self.is_array {
            writeln!(writer)?;
            write!(
                writer,
                "{:<offset$}- {}",
                "",
                field,
                offset = self.offset - 2
            )
        } else {
            write!(
                writer,
                "{:>pad$}{}",
                "",
                field,
                pad = self.pad - self.title_size
            )
        }
    }
}

#[doc(hidden)]
pub fn get_keys(field_name: &str) -> (&str, &str) {
    match field_name.split_once(".") {
        None => (field_name, ""),
        Some((field_name, child)) => (field_name, child),
    }
}

pub trait Describe {
    // Method that take a field name and should return a String value of the field.
    // This method extract keys with dot in order to call the to_field method for children
    fn to_field(&self, field_name: &str) -> String;

    // Return the default_headers for the structs
    fn default_headers() -> Vec<String> {
        Self::headers()
    }

    // Return the list of all headers for the struct
    fn headers() -> Vec<String> {
        vec![]
    }

    // Return another name for an header
    fn header_name(_: &str) -> Option<String> {
        None
    }

    fn struct_pad() -> usize {
        0
    }

    // Describe write the current description of the struct
    // The current version is used for scalar types
    fn describe<W>(&self, writer: &mut W, ctx: Context) -> io::Result<()>
    where
        W: io::Write,
    {
        ctx.write_value(writer, self.to_field(""))
    }
}

impl Describe for DateTime<Utc> {
    fn to_field(&self, _: &str) -> String {
        self.format("%d-%m-%y %H:%M:%S").to_string()
    }
}

impl<V: Describe> Describe for HashMap<String, V> {
    fn to_field(&self, _: &str) -> String {
        "todo".to_string()
    }

    fn describe<W: io::Write>(&self, writer: &mut W, ctx: Context) -> io::Result<()> {
        if !self.is_empty() {
            let pad = &self.keys().map(|k| k.len()).max().unwrap_or_default() + 1;
            let mut keys = self.keys().collect::<Vec<_>>();
            keys.sort();
            for k in keys {
                ctx.write_title(writer, k, false)?;
                self[k].describe(writer, ctx.indent(pad, k.len()))?;
            }
        } else {
            ctx.write_value(writer, "~".to_string())?
        }
        Ok(())
    }
}

impl<T: Describe> Describe for Vec<T> {
    fn to_field(&self, field: &str) -> String {
        self.iter()
            .map(|x| x.to_field(field))
            .collect::<Vec<_>>()
            .join(",")
    }

    fn describe<W: io::Write>(&self, writer: &mut W, ctx: Context) -> io::Result<()> {
        if self.is_empty() {
            ctx.write_value(writer, "~".to_string())
        } else {
            for inner in self {
                inner.describe(writer, ctx.array())?;
            }
            Ok(())
        }
    }
}

impl<T: Describe> Describe for Option<T> {
    fn to_field(&self, field_name: &str) -> String {
        match self {
            None => "~".to_string(),
            Some(v) => v.to_field(field_name),
        }
    }

    fn headers() -> Vec<String> {
        T::headers()
    }

    fn header_name(header: &str) -> Option<String> {
        T::header_name(header)
    }

    fn describe<W: io::Write>(&self, writer: &mut W, ctx: Context) -> io::Result<()> {
        match self {
            None => ctx.write_value(writer, "~".to_string()),
            Some(v) => v.describe(writer, ctx),
        }
    }
}

pub struct Describer;

impl Describer {
    pub fn describe_object<W: io::Write, T>(
        data: &T,
        writer: &mut W,
        ctx: Context,
    ) -> io::Result<()>
    where
        T: Describe,
    {
        data.describe(writer, ctx)?;
        writeln!(writer)
    }

    pub fn describe_list<W: io::Write, T>(
        data: &[T],
        writer: &mut W,
        ctx: Context,
    ) -> io::Result<()>
    where
        T: Describe,
    {
        Self::describe_list_with_header(data, &[], writer, ctx)
    }

    pub fn describe_list_with_header<W: io::Write, T>(
        data: &[T],
        headers: &[String],
        writer: &mut W,
        ctx: Context,
    ) -> io::Result<()>
    where
        T: Describe,
    {
        Self::describe_list_internal(data, headers, writer, ctx)?;
        writeln!(writer)
    }

    fn describe_list_internal<W: io::Write, T>(
        data: &[T],
        headers: &[String],
        writer: &mut W,
        ctx: Context,
    ) -> io::Result<()>
    where
        T: Describe,
    {
        // Compute headers to display
        let default_headers: Vec<String> =
            T::default_headers().iter().map(|x| x.to_string()).collect();
        let headers = if headers.is_empty() {
            default_headers.as_slice()
        } else {
            headers
        };

        // Compute rows
        let rows = data
            .iter()
            .map(|row| {
                headers
                    .iter()
                    .map(|x| {
                        let val = x.as_str();
                        row.to_field(val)
                    })
                    .collect::<Vec<_>>()
            })
            .collect::<Vec<_>>();

        let header_names = headers
            .iter()
            .map(|header| match T::header_name(header) {
                None => header.to_string().to_case(Case::UpperSnake),
                Some(header) => header,
            })
            .collect::<Vec<_>>();

        // Compute columns width
        let mut col_widths = header_names
            .iter()
            .map(|header| header.len())
            .collect::<Vec<_>>();
        for row in rows.iter() {
            for (idx, cell) in row.iter().enumerate() {
                col_widths[idx] = col_widths[idx].max(Self::compute_string_size(cell))
            }
        }

        let header_len = header_names.len();
        // Print header
        for (idx, cell) in header_names.into_iter().enumerate() {
            if idx > 0 {
                write!(writer, " ")?;
            }

            let space = if idx + 1 != header_len {
                format!("{:width$}", "", width = col_widths[idx] - cell.len())
            } else {
                format!("")
            };

            write!(
                writer,
                "{:<offset$}{}{}",
                "",
                cell.as_str(),
                space,
                offset = ctx.offset
            )?;
        }

        // Print rows
        if rows.is_empty() {
            writeln!(writer, "Empty list")?;
        }
        for row in rows {
            writeln!(writer)?;
            for (idx, cell) in row.into_iter().enumerate() {
                if idx > 0 {
                    writer.write_fmt(format_args!(" "))?;
                }
                let space = if idx + 1 != header_len {
                    format!(
                        "{:width$}",
                        "",
                        width = col_widths[idx] - Self::compute_string_size(&cell)
                    )
                } else {
                    format!("")
                };
                writer.write_fmt(format_args!(
                    "{:<offset$}{}{}",
                    "",
                    cell,
                    space,
                    offset = ctx.offset
                ))?;
            }
        }

        Ok(())
    }

    fn compute_string_size(str: &str) -> usize {
        String::from_utf8(strip_ansi_escapes::strip(str).unwrap())
            .unwrap_or_else(|_| str.to_string())
            .len()
    }
}

pub fn object_describe_to_string<T: Describe>(object: &T) -> io::Result<String> {
    let mut vec = Vec::with_capacity(128);
    Describer::describe_object(object, &mut vec, Context::default())?;
    let string = String::from_utf8(vec).unwrap();
    Ok(string)
}

pub fn object_describe<W: io::Write, T: Describe>(object: &T, writer: &mut W) -> io::Result<()> {
    Describer::describe_object(object, writer, Context::default())
}

pub fn table_describe_to_string<T: Describe>(data: &[T]) -> io::Result<String> {
    let mut vec = Vec::with_capacity(128);
    Describer::describe_list(data, &mut vec, Context::default())?;
    let string = String::from_utf8(vec).unwrap();
    Ok(string)
}

pub fn table_describe_with_header_to_string<T: Describe>(
    data: &[T],
    headers: &[String],
) -> io::Result<String> {
    let mut vec = Vec::with_capacity(128);
    Describer::describe_list_with_header(data, headers, &mut vec, Context::default())?;
    let string = String::from_utf8(vec).unwrap();
    Ok(string)
}

pub fn table_describe<W: io::Write, T: Describe>(
    data: &[T],
    headers: &[String],
    writer: &mut W,
) -> io::Result<()> {
    Describer::describe_list_with_header(data, headers, writer, Context::default())
}

#[doc(hidden)]
macro_rules! describe_macro_to_string {
    (
        $t: ty
    ) => {
        impl Describe for $t {
            fn to_field(&self, _: &str) -> String {
                self.to_string()
            }
        }
    };
}

describe_macro_to_string!(String);
describe_macro_to_string!(i32);
describe_macro_to_string!(i64);
describe_macro_to_string!(u32);
describe_macro_to_string!(u64);
describe_macro_to_string!(u16);
describe_macro_to_string!(i16);
describe_macro_to_string!(usize);
describe_macro_to_string!(bool);
