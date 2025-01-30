use simple_yaml_parser::parse as parse_yaml;

fn main() {
    let example = r#"
person:
  name: John Doe
  description: |
    something here
    that spans multiple lines
  age: 30
  something:
    x: true
  address:
    street: 123 Main St
    city: Example City
places:
  list: ["something", "here"]
  inner:
    x: string
"#
    .trim_start();

    let source = if let Some(path) = std::env::args().nth(1) {
        std::fs::read_to_string(path).unwrap()
    } else {
        example.to_owned()
    };

    // let second_arg = std::env::args().nth(2);
    // let mode = second_arg
    //     .and_then(|arg| {
    //         matches!(arg.as_str(), "--verbose" | "--text").then_some(arg[2..].to_owned())
    //     })
    //     .unwrap_or_default();

    parse_yaml(&source, |keys, value| {
        eprintln!("{keys:?} -> {value:?}");
    })
    .unwrap();
}
