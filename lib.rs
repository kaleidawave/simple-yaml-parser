#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YAMLKey<'a> {
    Slice(&'a str),
    Index(usize),
}

#[derive(Debug, PartialEq, Eq)]
pub enum RootYAMLValue<'a> {
    String(&'a str),
    MultilineString(MultilineString<'a>),
    Number(&'a str),
    True,
    False,
    // Null,
}

#[derive(Debug)]
pub enum YAMLParseErrorReason {
    ExpectedColon,
    ExpectedEndOfValue,
    ExpectedBracket,
    ExpectedTrueFalseNull,
    ExpectedValue,
}

#[derive(Debug)]
pub struct YAMLParseError {
    pub at: usize,
    pub reason: YAMLParseErrorReason,
}

impl std::error::Error for YAMLParseError {}

impl std::fmt::Display for YAMLParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_fmt(format_args!(
            "YAMLParseError: {:?} at {:?}",
            self.reason, self.at
        ))
    }
}

/// If you want to return early (not parse the whole input) use [`parse_with_exit_signal`]
///
/// # Errors
/// Returns an error if it tries to parse invalid YAML input
pub fn parse<'a>(
    on: &'a str,
    mut cb: impl for<'b> FnMut(&'b [YAMLKey<'a>], RootYAMLValue<'a>),
) -> Result<(), YAMLParseError> {
    parse_with_exit_signal(
        on,
        |k, v| {
            cb(k, v);
            false
        },
        &ParseOptions::default(),
    )
}

/// For `|` and `>` based values
#[derive(Debug, PartialEq, Eq)]
pub struct MultilineString<'a> {
    on: &'a str,
    /// replace new lines with spaces. Done using `>`
    collapse: bool,
    /// with `|+` etc
    preserve_leading_whitespace: bool,
}

pub struct ParseOptions {
    pub indent_size: usize,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self { indent_size: 2 }
    }
}

/// # Errors
/// Returns an error if it tries to parse invalid YAML input
#[allow(clippy::too_many_lines)]
pub fn parse_with_exit_signal<'a>(
    on: &'a str,
    mut cb: impl for<'b> FnMut(&'b [YAMLKey<'a>], RootYAMLValue<'a>) -> bool,
    options: &ParseOptions,
) -> Result<(), YAMLParseError> {
    enum State {
        Value,
        Identifier,
        ListItem,
        Multiline {
            collapse: bool,
            preserve_leading_whitespace: bool,
            indent: usize,
        },
        Skip,
    }

    let chars = on.char_indices();

    let mut key_chain = Vec::new();
    let mut state = State::Identifier;
    let mut list_idx: usize = 0;
    let mut indent = 0;
    let mut start = 0;

    for (idx, chr) in chars {
        match state {
            State::Value => {
                let rest_of_line = on[start..idx].trim();
                if let (true, '-') = (rest_of_line.is_empty(), chr) {
                    state = State::ListItem;
                    start = idx + '-'.len_utf8();
                } else if let '\n' = chr {
                    if rest_of_line.is_empty() {
                        // ready for identifier
                        state = State::Skip;
                    } else {
                        let modifier = match rest_of_line {
                            "|" => Some((true, false)),
                            ">" => Some((false, false)),
                            _ => None,
                        };
                        if let Some((collapse, preserve_leading_whitespace)) = modifier {
                            state = State::Multiline {
                                collapse,
                                preserve_leading_whitespace,
                                indent,
                            };
                            start = idx;
                        } else {
                            let value = on[start..idx].trim();
                            let value = match value {
                                "true" => RootYAMLValue::True,
                                "false" => RootYAMLValue::False,
                                value => RootYAMLValue::String(value),
                            };
                            cb(&key_chain, value);
                            key_chain.pop();
                            state = State::Skip;
                        }
                    }
                    indent = 0;
                }
            }
            State::Multiline {
                collapse,
                preserve_leading_whitespace,
                indent: current_indent,
            } => {
                if let '\n' = chr {
                    let upcoming_line = &on[(idx + '\n'.len_utf8())..];
                    let mut upcoming_indent = 0;
                    let mut is_empty = false;
                    for chr in upcoming_line.chars() {
                        if let '\n' = chr {
                            is_empty = true;
                            break;
                        }
                        if let '\t' | ' ' = chr {
                            upcoming_indent += 1;
                        } else {
                            break;
                        }
                    }
                    if !is_empty && upcoming_indent <= current_indent {
                        let multiline_string = MultilineString {
                            on: &on[start..idx],
                            collapse,
                            preserve_leading_whitespace,
                        };
                        cb(&key_chain, RootYAMLValue::MultilineString(multiline_string));
                        key_chain.pop();
                        state = State::Skip;
                        indent = 0;
                    }
                }
            }
            State::Identifier => {
                if let ':' = chr {
                    let key = YAMLKey::Slice(on[start..idx].trim());
                    let current_level = indent / options.indent_size;
                    let keys = key_chain
                        .iter()
                        .filter(|key| matches!(key, YAMLKey::Slice(_)))
                        .count();
                    if current_level < keys {
                        drop(key_chain.drain(current_level..));
                        match key_chain.last() {
                            Some(YAMLKey::Index(idx)) => {
                                list_idx = *idx;
                            }
                            _ => {
                                list_idx = 0;
                            }
                        }
                    }
                    key_chain.push(key);
                    state = State::Value;
                    start = idx + ':'.len_utf8();
                }
                // TODO whitespace warning etc...?
            }
            State::ListItem => {
                if let ':' = chr {
                    let current_level = indent / options.indent_size;
                    if current_level < key_chain.len() {
                        drop(key_chain.drain((current_level + 1)..));
                    }
                    key_chain.push(YAMLKey::Index(list_idx));
                    key_chain.push(YAMLKey::Slice(on[start..idx].trim()));
                    state = State::Value;
                    start = idx + ':'.len_utf8();
                    list_idx += 1;
                }
                if let '\n' = chr {
                    key_chain.push(YAMLKey::Index(list_idx));
                    let value = on[start..idx].trim();
                    let value = match value {
                        "true" => RootYAMLValue::True,
                        "false" => RootYAMLValue::False,
                        value => RootYAMLValue::String(value),
                    };
                    cb(&key_chain, value);
                    key_chain.pop();
                    list_idx += 1;
                    state = State::Skip;
                    indent = 0;
                }
            }
            State::Skip => {
                if let '-' = chr {
                    state = State::ListItem;
                    start = idx + '-'.len_utf8();
                } else if let '\t' = chr {
                    indent += options.indent_size;
                } else if let ' ' = chr {
                    indent += 1;
                } else if !chr.is_whitespace() {
                    state = State::Identifier;
                    start = idx;
                }
            }
        }
    }

    // TODO left over stuff here

    Ok(())
}
