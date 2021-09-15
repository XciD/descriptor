use descriptor::{table_describe_to_string, table_describe_with_header_to_string, Descriptor};

pub fn no_color_and_line_return(str: String) -> String {
    format!(
        "\n{}",
        String::from_utf8(strip_ansi_escapes::strip(str).unwrap()).unwrap()
    )
    .to_string()
}

#[test]
fn test_table_descriptor() {
    #[derive(Descriptor, Clone)]
    struct InnerA {
        state: String,
        value: String,
    }

    let table = vec![
        InnerA {
            state: "f".to_string(),
            value: "bar".to_string(),
        },
        InnerA {
            state: "foo".to_string(),
            value: "b".to_string(),
        },
    ];
    let table = table_describe_to_string(&table).unwrap();

    assert_eq!(
        r#"
STATE VALUE
f     bar
foo   b
"#,
        no_color_and_line_return(table)
    )
}

#[test]
fn test_table_skip() {
    #[derive(Descriptor, Clone)]
    struct TableHide {
        table: String,
        long_column: String,
        small_one: String,
        #[descriptor(skip_header)]
        hidden_one: String,
    }

    let list = vec![
        TableHide {
            table: "table".to_string(),
            long_column: "long".to_string(),
            small_one: "s".to_string(),
            hidden_one: "hidden".to_string(),
        },
        TableHide {
            table: "row2".to_string(),
            long_column: "row".to_string(),
            small_one: "s".to_string(),
            hidden_one: "hidden".to_string(),
        },
    ];

    let table = table_describe_to_string(&list).unwrap();

    assert_eq!(
        r#"
TABLE LONG_COLUMN SMALL_ONE
table long        s
row2  row         s
"#,
        no_color_and_line_return(table)
    );

    let table =
        table_describe_with_header_to_string(&list, &vec!["hidden_one".to_string()]).unwrap();

    assert_eq!(
        r#"
HIDDEN_ONE
hidden
hidden
"#,
        no_color_and_line_return(table)
    );
}

#[test]
fn test_table_headers() {
    #[derive(Descriptor, Clone)]
    #[descriptor(default_headers = ["table", "long_column"])]
    struct Table {
        table: String,
        long_column: String,
        small_one: String,
    }

    let table = vec![
        Table {
            table: "table".to_string(),
            long_column: "long".to_string(),
            small_one: "s".to_string(),
        },
        Table {
            table: "row2".to_string(),
            long_column: "row".to_string(),
            small_one: "s".to_string(),
        },
    ];

    let table = table_describe_to_string(&table).unwrap();
    assert_eq!(
        r#"
TABLE LONG_COLUMN
table long
row2  row
"#,
        no_color_and_line_return(table)
    );
}

#[test]
fn test_table_inner() {
    #[derive(Descriptor)]
    struct Foo {
        string: String,
    }
    #[derive(Descriptor)]
    struct Bar {
        inner_foo: Foo,
        parent: String,
    }
    let foo = Bar {
        inner_foo: Foo {
            string: "c".to_string(),
        },
        parent: "parent".to_string(),
    };

    let table = table_describe_to_string(&vec![foo]).unwrap();
    assert_eq!(
        r#"
INNER_FOO.STRING PARENT
c                parent
"#,
        no_color_and_line_return(table)
    );
}

#[test]
fn test_into_field_level() {
    #[derive(Descriptor)]
    struct Foo {
        #[descriptor(into = AnotherFoo)]
        foo: Bar,
    }

    struct Bar {
        foo: String,
        bar: String,
    }

    #[derive(Descriptor)]
    struct AnotherFoo {
        lorem: String,
    }

    impl From<&Bar> for AnotherFoo {
        fn from(f: &Bar) -> Self {
            Self {
                lorem: format!("{}-{}", f.foo, f.bar),
            }
        }
    }
    let table = table_describe_to_string(&vec![Foo {
        foo: Bar {
            foo: "a".to_string(),
            bar: "b".to_string(),
        },
    }])
    .unwrap();

    assert_eq!(
        r#"
FOO.LOREM
a-b
"#,
        no_color_and_line_return(table)
    );
}

#[test]
fn test_extra_fields() {
    #[derive(Descriptor)]
    #[descriptor(extra_fields = ExtraFieldsStruct)]
    struct Foo {
        first_field: String,
        number: u32,
    }

    #[derive(Descriptor)]
    struct ExtraFieldsStruct {
        anything: String,
    }

    impl From<&Foo> for ExtraFieldsStruct {
        fn from(b: &Foo) -> Self {
            Self {
                anything: format!("{}-{}", b.first_field, b.number / 10),
            }
        }
    }

    let table = table_describe_to_string(&vec![Foo {
        first_field: "test".to_string(),
        number: 200,
    }])
    .unwrap();
    assert_eq!(
        r#"
FIRST_FIELD NUMBER ANYTHING
test        200    test-20
"#,
        no_color_and_line_return(table)
    );
}

#[test]
fn test_map_all() {
    #[derive(Descriptor)]
    #[descriptor(map = map_all)]
    struct Foo {
        first_field: String,
        number: u32,
    }

    fn map_all(b: &Foo, field: String) -> String {
        if b.number > 2 {
            "-".to_string()
        } else {
            field
        }
    }

    let table = table_describe_to_string(&vec![
        Foo {
            first_field: "test".to_string(),
            number: 200,
        },
        Foo {
            first_field: "test".to_string(),
            number: 1,
        },
    ])
    .unwrap();
    assert_eq!(
        r#"
FIRST_FIELD NUMBER
-           -
test        1
"#,
        no_color_and_line_return(table)
    );
}
