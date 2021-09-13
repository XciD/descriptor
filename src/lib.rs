use std::collections::HashMap;
use std::io;

use chrono::{DateTime, Utc};
use convert_case::{Case, Casing};
use crossterm::style::{StyledContent, Stylize};

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
        Printer::describe_list_internal(data, &[], writer, self.indent_and_table())
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

pub fn get_keys(field_name: &str) -> (&str, &str) {
    match field_name.split_once(".") {
        None => (field_name, ""),
        Some((field_name, child)) => (field_name, child),
    }
}

pub trait Describe {
    // Method that take a field name and should return a String value of the field.
    // This method will extract keys with dot in order to call the to_field method for children
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

    // Describe will write the current description of the struct
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
            ctx.write_value(writer, "~".bold().to_string())?
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
            ctx.write_value(writer, "~".bold().to_string())
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
            None => "~".bold().to_string(),
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
            None => ctx.write_value(writer, "~".bold().to_string()),
            Some(v) => v.describe(writer, ctx),
        }
    }
}

#[macro_export]
macro_rules! print_macro {
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

print_macro!(String);
print_macro!(StyledContent<String>);
print_macro!(i32);
print_macro!(i64);
print_macro!(u32);
print_macro!(u64);
print_macro!(u16);
print_macro!(i16);
print_macro!(usize);
print_macro!(bool);

pub struct Printer;

impl Printer {
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
                col_widths[idx] = col_widths[idx].max(Self::get_string_size(cell))
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
                cell.as_str().bold(),
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
                        width = col_widths[idx] - Self::get_string_size(&cell)
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

    fn get_string_size(str: &str) -> usize {
        String::from_utf8(strip_ansi_escapes::strip(str).unwrap())
            .unwrap_or_else(|_| str.to_string())
            .len()
    }
}

pub fn object_describe_to_string<T: Describe>(object: &T) -> io::Result<String> {
    let mut vec = Vec::with_capacity(128);
    Printer::describe_object(object, &mut vec, Context::default())?;
    let string = String::from_utf8(vec).unwrap();
    Ok(string)
}

pub fn object_describe<W: io::Write, T: Describe>(object: &T, writer: &mut W) -> io::Result<()> {
    Printer::describe_object(object, writer, Context::default())
}

pub fn list_describe_to_string<T: Describe>(data: &[T]) -> io::Result<String> {
    let mut vec = Vec::with_capacity(128);
    Printer::describe_list(data, &mut vec, Context::default())?;
    let string = String::from_utf8(vec).unwrap();
    Ok(string)
}

pub fn list_describe_with_header_to_string<T: Describe>(
    data: &[T],
    headers: &[String],
) -> io::Result<String> {
    let mut vec = Vec::with_capacity(128);
    Printer::describe_list_with_header(data, headers, &mut vec, Context::default())?;
    let string = String::from_utf8(vec).unwrap();
    Ok(string)
}

pub fn list_describe<W: io::Write, T: Describe>(
    data: &[T],
    headers: &[String],
    writer: &mut W,
) -> io::Result<()> {
    Printer::describe_list_with_header(data, headers, writer, Context::default())
}
