use simple_yaml_parser::parse as parse_yaml;
// use simple_yaml_parser::value::parse_advanced as parse_yaml_expression;

static EXAMPLE: &str = r#"
person:
  name: John Doe
  description: |
    something here
    that spans multiple lines
  age: 30
  something:
    x: true
  address  :
    street: 123 Main St
    city: Example City
places:
  list: ["something", "here"]
  inner:
    x: string
  jobs:
    - x: string
      b:
        with: 5
"#;

fn main() {
    let arg = std::env::args().nth(1);

    if let Some("--interactive") = arg.as_deref() {
        run_interactive();
        return;
    }

    let source = if let Some("--content") = arg.as_deref() {
        std::env::args().nth(2).expect("no content")
    } else if let Some(path) = arg {
        std::fs::read_to_string(path).unwrap()
    } else {
        EXAMPLE.trim_start().to_owned()
    };

    let out = parse_yaml(&source, |keys, value| {
        println!("{keys:?}");
        println!(" -> {value:?}");
        // debug_keys(keys, context);
        // debug_value(value);
        // false
    });

    if let Err(error) = out {
        eprintln!("Error: {error:?}");
    }
}

// TODO better formatting needed
// fn debug_keys(keys: &[TOMLKey<'_>], context: &TOMLKeyContext) {
//     fn debug_keys_(keys: &[TOMLKey<'_>]) {
//         let mut first = false;
//         for key in keys {
//             if first {
//                 print!(".");
//             }
//             match key {
//                 TOMLKey::Slice(item) => print!("{item:?}"),
//                 TOMLKey::Index(item) => print!("{item:?}"),
//             }
//             first = true;
//         }
//     }

//     let (table_keys, specifier_keys, object_keys) = context.split_keys(keys);

//     if !table_keys.is_empty() {
//         print!("[");
//         debug_keys_(table_keys);
//         print!("] ");
//     }

//     if !specifier_keys.is_empty() {
//         debug_keys_(specifier_keys);
//     }

//     for keys in object_keys {
//         print!(" {{");
//         debug_keys_(keys);
//         print!("}}");
//     }

//     print!(" => ");
// }

// fn debug_value(value: RootTOMLValue<'_>) {
//     use std::borrow::Cow;

//     if let RootTOMLValue::String(value) = value {
//         if value.is_literal() {
//             let mut start = 0;
//             let value = value.raw();
//             let mut escaped = Cow::Borrowed("");
//             for (idx, matched) in value.match_indices(&['\t', '\r', '\n']) {
//                 escaped += Cow::Borrowed(&value[start..idx]);
//                 match matched {
//                     "\t" => {
//                         escaped += Cow::Borrowed("\\t");
//                     }
//                     "\n" => {
//                         escaped += Cow::Borrowed("\\n");
//                     }
//                     "\r" => {
//                         escaped += Cow::Borrowed("\\r");
//                     }
//                     chr => unreachable!("{chr}"),
//                 }
//                 start = idx + 1;
//             }
//             escaped += Cow::Borrowed(&value[start..]);
//             println!("String('{escaped}')");
//         } else {
//             println!("String({value:?})", value = value.value());
//         }
//     } else {
//         println!("{value:?}");
//     }
// }

fn run_interactive() {
    use std::io::{stdin, BufRead};
    let stdin = stdin();
    let mut buf = Vec::new();

    println!("start");

    for line in stdin.lock().lines().map_while(Result::ok) {
        if line == "close" {
            if !buf.is_empty() {
                eprintln!("no end to message {buf:?}");
            }
            break;
        }

        if line == "end" {
            let output = String::from_utf8_lossy(&buf);
            // println!("{output}");
            let out = parse_yaml(&output, |keys, value| {
                println!("{keys:?}");
                println!(" -> {value:?}");
                // debug_keys(keys, context);
                // debug_value(value);
                // false
            });
            if let Err(error) = out {
                println!("Error: {error:?}");
            }
            println!("end");
            buf.clear();
            continue;
        }

        buf.extend_from_slice(line.as_bytes());
        buf.push(b'\n');
    }
}
