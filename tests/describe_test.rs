use std::collections::HashMap;

use descriptor::{object_describe_to_string, Descriptor, table_describe, table_describe_to_string};

pub fn no_color(str: String) -> String {
    String::from_utf8(strip_ansi_escapes::strip(str).unwrap()).unwrap()
}

#[test]
fn test_flatten() {
    #[derive(Descriptor)]
    struct Foo {
        loooooooong: String,
        #[descriptor(flatten)]
        child: ChildFoo,
    }

    #[derive(Descriptor)]
    struct ChildFoo {
        twoooo: String,
        #[descriptor(flatten)]
        child: GrandKid,
    }

    #[derive(Descriptor)]
    struct GrandKid {
        one: String,
    }

    let description = object_describe_to_string(&Foo {
        loooooooong: "one".to_string(),
        child: ChildFoo {
            twoooo: "two".to_string(),
            child: GrandKid {
                one: "tee".to_string(),
            },
        },
    })
    .unwrap();
    assert_eq!(
        r#"
Loooooooong: one
Twoooo:      two
One:         tee
"#,
        description
    );
}

#[test]
fn test_func() {

    #[derive(Descriptor)]
    struct Bar {
        test: String,
    }

    #[derive(Descriptor)]
    struct Foo {
        #[descriptor(map = map_test)]
        time: u64,
        #[descriptor(map = map_test, resolve_option)]
        optional_some: Option<u64>,
        #[descriptor(map = map_test, resolve_option)]
        optional_none: Option<u64>,
    }

    fn map_test(val: &u64) -> Bar {
        Bar{test: format!("{} seconds", val)}
    }

    let foo = Foo {
        time: 10000,
        optional_some: Some(10),
        optional_none: None,
    };
    let description = object_describe_to_string(&foo)
    .unwrap();

    let result = table_describe_to_string(&vec![foo]).unwrap();
    println!("{}", result)
}

#[test]
fn test_vec() {
    #[derive(Descriptor)]
    struct Foo {
        list: Vec<ListBar>,
        string_list: Vec<String>,
    }

    #[derive(Descriptor)]
    struct ListBar {
        first_field: String,
        child: FooChild,
        second: Vec<FooChild>,
    }

    #[derive(Descriptor)]
    struct FooChild {
        anything: String,
    }

    let table = object_describe_to_string(&Foo {
        list: vec![
            ListBar {
                first_field: "first".to_string(),
                child: FooChild {
                    anything: "any".to_string(),
                },
                second: vec![FooChild {
                    anything: "foo".to_string(),
                }],
            },
            ListBar {
                first_field: "second".to_string(),
                child: FooChild {
                    anything: "any".to_string(),
                },
                second: vec![FooChild {
                    anything: "foo".to_string(),
                }],
            },
        ],
        string_list: vec!["test".to_string(), "test2".to_string()],
    })
    .unwrap();

    assert_eq!(
        r#"
List:
- First Field: first
  Child:
    Anything: any
  Second:
  - Anything: foo
- First Field: second
  Child:
    Anything: any
  Second:
  - Anything: foo
String List:
- test
- test2
"#,
        no_color(table)
    );
}

#[test]
fn test_into_struct_level() {
    #[derive(Descriptor)]
    #[descriptor(into = TestFromReceiver)]
    struct TestFrom {
        foo: String,
        bar: String,
    }

    #[derive(Descriptor)]
    struct TestFromReceiver {
        recv: String,
    }

    #[derive(Descriptor)]
    #[descriptor(into = TestIntoReceiver)]
    struct TestInto {
        foo_into: String,
        bar_into: String,
    }

    #[derive(Descriptor)]
    struct TestIntoReceiver {
        recv: String,
    }

    impl From<&TestFrom> for TestFromReceiver {
        fn from(e: &TestFrom) -> Self {
            Self {
                recv: format!("{}-{}", e.foo, e.bar),
            }
        }
    }

    impl Into<TestIntoReceiver> for &TestInto {
        fn into(self) -> TestIntoReceiver {
            TestIntoReceiver {
                recv: format!("{}-{}", self.foo_into, self.bar_into),
            }
        }
    }

    let from = object_describe_to_string(&TestFrom {
        foo: "foo".to_string(),
        bar: "bar".to_string(),
    })
    .unwrap();
    println!("{}", from);

    assert_eq!("\nRecv: foo-bar\n", from);

    let into = object_describe_to_string(&TestInto {
        foo_into: "foo".to_string(),
        bar_into: "bar".to_string(),
    })
    .unwrap();

    assert_eq!("\nRecv: foo-bar\n", into)
}

#[test]
fn test_into_field_level() {
    #[derive(Descriptor)]
    struct Foo {
        #[descriptor(into = Bar)]
        foo: FooInto,
        #[descriptor(into = Bar, map = FooInto::test)]
        bar: FooInto,
    }

    struct FooInto {
        foo: String,
        bar: String,
    }

    #[derive(Descriptor)]
    struct Bar {
        foo: String,
    }

    impl From<&FooInto> for Bar {
        fn from(f: &FooInto) -> Self {
            Self {
                foo: format!("{}-{}", f.foo, f.bar),
            }
        }
    }

    impl FooInto {
        fn test(&self) -> Bar {
            Bar {
                foo: format!("{}-{}", self.bar, self.foo),
            }
        }
    }

    let string = object_describe_to_string(&Foo {
        foo: FooInto {
            foo: "a".to_string(),
            bar: "b".to_string(),
        },
        bar: FooInto {
            foo: "a".to_string(),
            bar: "b".to_string(),
        },
    })
    .unwrap();
    println!("{}", string);
    assert_eq!("\nFoo:\n  Foo: a-b\nBar:\n  Foo: b-a\n", string)
}

#[test]
fn test_table_description() {
    #[derive(Descriptor)]
    struct Foo {
        history_no_table: Vec<InnerFoo>,
        #[descriptor(output_table)]
        history: Vec<InnerFoo>,
    }

    #[derive(Descriptor, Clone)]
    struct InnerFoo {
        state: String,
        value: String,
    }

    let list = vec![
        InnerFoo {
            state: "test".to_string(),
            value: "t".to_string(),
        },
        InnerFoo {
            state: "t".to_string(),
            value: "test".to_string(),
        },
    ];
    let object = &Foo {
        history_no_table: list.clone(),
        history: list.clone(),
    };
    let description = object_describe_to_string(object).unwrap();
    assert_eq!(
        r#"
History No Table:
- State: test
  Value: t
- State: t
  Value: test
History:
  STATE   VALUE
  test    t
  t       test
"#,
        no_color(description)
    );
}

#[test]
fn test_describe_enum() {
    #[derive(Descriptor)]
    struct Foo {
        enum_field: Bar,
        enum_renamed: Bar,
    }
    #[derive(Descriptor)]
    enum Bar {
        SomeEnum,
        #[descriptor(rename_description = "Rename AnnotationValue")]
        AnotherRenamed,
    }

    let description = object_describe_to_string(&Foo {
        enum_field: Bar::SomeEnum,
        enum_renamed: Bar::AnotherRenamed,
    })
    .unwrap();
    assert_eq!(
        r#"
Enum Field:   SomeEnum
Enum Renamed: Rename AnnotationValue
"#,
        no_color(description)
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

    let description = object_describe_to_string(&Foo {
        first_field: "test".to_string(),
        number: 200,
    })
    .unwrap();
    assert_eq!(
        r#"
First Field: test
Number:      200
Anything:    test-20
"#,
        no_color(description)
    );
}

#[test]
fn test_map() {
    #[derive(Descriptor)]
    struct B {
        map: HashMap<String, String>,
    }

    let mut map = HashMap::new();
    map.insert("test".to_string(), "f".to_string());
    map.insert("f".to_string(), "f".to_string());

    let description = object_describe_to_string(&B { map }).unwrap();
    assert_eq!(
        r#"
Map:
  f:    f
  test: f
"#,
        no_color(description)
    );
}
