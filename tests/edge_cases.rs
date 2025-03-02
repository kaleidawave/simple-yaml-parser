use simple_yaml_parser::{parse, YAMLParseError};

#[test]
fn good_cases() {
    let good_cases: &[&str] = &[
        r#"a:
            b: 2"#,
        r#"a:
            b: [something, here]"#,
    ];

    for case in good_cases {
        let result = parse(case, |keys, value| eprintln!("{keys:?} -> {value:?}"));

        match result {
            Ok(_) => {
                eprintln!("✅ Valid script did not error")
            }
            Err(YAMLParseError { at, reason }) => {
                panic!("❌ Valid script error'ed {reason:?} @ {at}");
            }
        }
    }
}

#[test]
fn bad_cases() {
    // TODO
    let bad_cases: &[&str] = &[];

    for case in bad_cases {
        let result = parse(case, |keys, value| eprintln!("{keys:?} -> {value:?}"));

        match result {
            Ok(_) => {
                eprintln!("❌ Invalid script did not error")
            }
            Err(YAMLParseError { at, reason }) => {
                eprintln!("✅ Invalid script error'ed {reason:?} @ {at}");
            }
        }
    }
}
