#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YAMLKey<'a> {
    Slice(&'a str),
    Index { index: usize, bracketed: bool },
}

#[derive(Debug, PartialEq, Eq)]
pub enum RootYAMLValue<'a> {
    String(&'a str),
    MultilineString(MultilineString<'a>),
    Number(&'a str),
    Boolean(bool),
    Null,
}

#[derive(Debug)]
pub enum YAMLParseErrorReason {
    ExpectedColon,
    ExpectedEndOfValue,
    ExpectedBracket,
    ExpectedTrueFalseNull,
    ExpectedValue,
    ExpectedKey,
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

/// If you want to return early (not parse the whole input) use [`parse_advanced`]
///
/// # Errors
/// Returns an error if it tries to parse invalid YAML input
pub fn parse<'a>(
    on: &'a str,
    mut cb: impl for<'b> FnMut(&'b [YAMLKey<'a>], RootYAMLValue<'a>),
) -> Result<(), YAMLParseError> {
    parse_advanced::<()>(
        on,
        |k, v| {
            cb(k, v);
            None
        },
        &ParseOptions::default(),
    )
    .map(|_none| ())
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
pub fn parse_advanced<'a, T>(
    on: &'a str,
    mut cb: impl for<'b> FnMut(&'b [YAMLKey<'a>], RootYAMLValue<'a>) -> Option<T>,
    options: &ParseOptions,
) -> Result<Option<T>, YAMLParseError> {
    enum State {
        Skip,
        /// TODO quoted strings
        Identifier,
        ListItem,
        MultilineStringValue {
            collapse: bool,
            preserve_leading_whitespace: bool,
            indent: usize,
        },
    }

    let mut chars = on.char_indices();

    let mut key_chain = Vec::new();
    let mut state = State::Identifier;
    let mut list_idx: usize = 0;
    let mut indent = 0;
    let mut start = 0;

    while let Some((idx, chr)) = chars.next() {
        match state {
            State::Skip => {
                if let '-' = chr {
                    state = State::ListItem;
                    start = idx + chr.len_utf8();
                } else if let '\t' = chr {
                    indent += options.indent_size;
                } else if let ' ' = chr {
                    indent += 1;
                } else if !chr.is_whitespace() {
                    state = State::Identifier;
                    start = idx;
                }
            }
            State::Identifier => {
                if let ':' = chr {
                    let current_level = indent / options.indent_size;
                    let key_chain_level = key_chain
                        .iter()
                        .filter(|key| matches!(key, YAMLKey::Slice(_)))
                        .count();

                    let key = YAMLKey::Slice(on[start..idx].trim());

                    // Indentation of key tracking
                    {
                        if current_level < key_chain_level {
                            let after = current_level;
                            drop(key_chain.drain(after..));
                            match key_chain.last() {
                                Some(YAMLKey::Index { index, .. }) => {
                                    list_idx = *index;
                                }
                                _ => {
                                    list_idx = 0;
                                }
                            }
                        }
                    }

                    key_chain.push(key);
                    start = idx + ':'.len_utf8();

                    {
                        let rest_of_line = on[start..].lines().next().unwrap_or_default();
                        let modifier = match rest_of_line.trim() {
                            "|" => Some((true, false)),
                            ">" => Some((false, false)),
                            _ => None,
                        };
                        if let Some((collapse, preserve_leading_whitespace)) = modifier {
                            state = State::MultilineStringValue {
                                collapse,
                                preserve_leading_whitespace,
                                indent,
                            };
                            start = idx + rest_of_line.len() + '\n'.len_utf8();
                        } else {
                            if !rest_of_line.is_empty() {
                                let _ = value::parse_advanced_chars(
                                    on,
                                    &mut chars,
                                    &mut key_chain,
                                    &mut cb,
                                );
                                let _popped = key_chain.pop();
                                // dbg!(popped);
                                // TODO is this correct
                                indent = 0;
                            }
                            state = State::Skip;
                        }
                    }
                }
                // TODO whitespace warning etc...?
            }
            State::ListItem => {
                if let ':' = chr {
                    let current_level = indent / options.indent_size;
                    if current_level < key_chain.len() {
                        drop(key_chain.drain((current_level + 1)..));
                    }
                    key_chain.push(YAMLKey::Index {
                        index: list_idx,
                        bracketed: false,
                    });
                    key_chain.push(YAMLKey::Slice(on[start..idx].trim()));
                    start = idx + ':'.len_utf8();
                    list_idx += 1;

                    // TODO abstract
                    {
                        let rest_of_line = on[start..].lines().next().unwrap_or_default();
                        let modifier = match rest_of_line {
                            "|" => Some((true, false)),
                            ">" => Some((false, false)),
                            _ => None,
                        };
                        if let Some((collapse, preserve_leading_whitespace)) = modifier {
                            state = State::MultilineStringValue {
                                collapse,
                                preserve_leading_whitespace,
                                indent,
                            };
                            start = idx + rest_of_line.len();
                        } else {
                            let _ = value::parse_advanced_chars(
                                on,
                                &mut chars,
                                &mut key_chain,
                                &mut cb,
                            );
                            let _popped = key_chain.pop();
                            indent = 0;
                            // dbg!(popped);
                            // TODO is this correct
                            state = State::Skip;
                        }
                    }
                }
                if let '\n' = chr {
                    key_chain.push(YAMLKey::Index {
                        index: list_idx,
                        bracketed: false,
                    });
                    let value = on[start..idx].trim();
                    let value = match value {
                        "true" => RootYAMLValue::Boolean(true),
                        "false" => RootYAMLValue::Boolean(false),
                        value => RootYAMLValue::String(value),
                    };
                    let res = cb(&key_chain, value);
                    if res.is_some() {
                        return Ok(res);
                    }
                    key_chain.pop();
                    list_idx += 1;
                    state = State::Skip;
                    indent = 0;
                }
            }
            // This is not a regular value
            State::MultilineStringValue {
                collapse,
                preserve_leading_whitespace,
                indent: current_indent,
            } => {
                if let '\n' = chr {
                    let upcoming_line = &on[(idx + '\n'.len_utf8())..];
                    let mut upcoming_indent = 0;
                    let mut is_empty = false;
                    for chr in upcoming_line.chars() {
                        if let '\n' | '\r' = chr {
                            is_empty = true;
                            break;
                        }

                        if let '\t' = chr {
                            upcoming_indent += options.indent_size;
                        } else if let ' ' = chr {
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
                        let res = cb(&key_chain, RootYAMLValue::MultilineString(multiline_string));
                        if res.is_some() {
                            return Ok(res);
                        }
                        key_chain.pop();
                        state = State::Skip;
                        indent = 0;
                    }
                }
            }
        }
    }

    // TODO left over stuff can should error here

    Ok(None)
}

pub mod value {
    use super::{RootYAMLValue, YAMLKey, YAMLParseError, YAMLParseErrorReason};

    #[derive(Debug)]
    enum State {
        InKey {
            escaped: bool,
            start: usize,
        },
        Colon,
        InObject,
        Comment {
            start: usize,
            multiline: bool,
            last_was_asterisk: bool,
            hash: bool,
        },
        ExpectingValue,
        // Smilar to string but without quotes
        LiteralValue {
            start: usize,
        },
        StringValue {
            start: usize,
            escaped: bool,
        },
        EndOfValue,
    }

    /// # Errors
    /// Returns an error if it tries to parse invalid YAML input
    pub fn parse_advanced<'a, T>(
        on: &'a str,
        mut cb: impl for<'b> FnMut(&'b [YAMLKey<'a>], RootYAMLValue<'a>) -> Option<T>,
    ) -> Result<(usize, Option<T>), YAMLParseError> {
        let mut chars = on.char_indices();
        let mut key_chain = Vec::new();
        parse_advanced_chars(on, &mut chars, &mut key_chain, &mut cb)
    }

    #[allow(clippy::too_many_lines)]
    pub(crate) fn parse_advanced_chars<'a, T>(
        on: &'a str,
        chars: &mut std::str::CharIndices<'a>,
        key_chain: &mut Vec<YAMLKey<'a>>,
        cb: &mut impl for<'b> FnMut(&'b [YAMLKey<'a>], RootYAMLValue<'a>) -> Option<T>,
    ) -> Result<(usize, Option<T>), YAMLParseError> {
        // Temp fix
        struct Options {
            pub allow_comments: bool,
        }

        let options = Options {
            allow_comments: false,
        };

        let mut state = State::ExpectingValue;

        let current_len = key_chain.len();

        for (idx, chr) in chars {
            match state {
                // TODO parse key function
                State::InKey {
                    ref mut start,
                    ref mut escaped,
                } => {
                    let is_colon = ':' == chr;
                    if is_colon || chr.is_whitespace() {
                        let key = &on[*start..idx];
                        key_chain.push(YAMLKey::Slice(key));
                        state = if is_colon {
                            State::ExpectingValue
                        } else {
                            State::Colon
                        };
                    } else {
                        *escaped = chr == '\\';
                    }
                    // if !*escaped && chr == '"' {
                    //     key_chain.push(YAMLKey::Slice(&on[start..idx]));
                    //     state = State::Colon;
                    // } else {
                    //     *escaped = chr == '\\';
                    // }
                }
                State::Colon => {
                    if chr == ':' {
                        state = State::ExpectingValue;
                    } else if !chr.is_whitespace() {
                        return Err(YAMLParseError {
                            at: idx,
                            reason: YAMLParseErrorReason::ExpectedColon,
                        });
                    }
                }
                State::InObject => {
                    if chr == '}' {
                        state = State::EndOfValue;
                        // TODO check result here
                        let _popped = key_chain.pop();
                    } else if let (true, c @ ('/' | '#')) = (options.allow_comments, chr) {
                        state = State::Comment {
                            last_was_asterisk: false,
                            start: idx,
                            multiline: false,
                            hash: c == '#',
                        };
                    } else if chr.is_alphabetic() {
                        state = State::InKey {
                            escaped: false,
                            start: idx,
                        }
                    } else if !chr.is_whitespace() {
                        return Err(YAMLParseError {
                            at: idx,
                            reason: YAMLParseErrorReason::ExpectedKey,
                        });
                    }
                }
                State::ExpectingValue => {
                    state = match chr {
                        '[' => {
                            key_chain.push(YAMLKey::Index {
                                index: 0,
                                bracketed: true,
                            });
                            State::ExpectingValue
                        }
                        ']' => {
                            // TODO check result here
                            key_chain.pop();
                            State::EndOfValue
                        }
                        '{' => State::InObject,
                        '}' => {
                            // TODO check result here
                            let _popped = key_chain.pop();
                            State::EndOfValue
                        }
                        '"' => State::StringValue {
                            start: idx + chr.len_utf8(),
                            escaped: false,
                        },
                        c @ ('/' | '#') if options.allow_comments => State::Comment {
                            last_was_asterisk: false,
                            start: idx,
                            multiline: false,
                            hash: c == '#',
                        },
                        chr if chr.is_whitespace() => state,
                        _ => State::LiteralValue { start: idx },
                    }
                }
                State::StringValue {
                    start,
                    ref mut escaped,
                } => {
                    if !*escaped && chr == '"' {
                        let res = cb(key_chain, RootYAMLValue::String(&on[start..idx]));
                        if res.is_some() {
                            return Ok((idx + chr.len_utf8(), res));
                        }
                        if key_chain.len() == current_len {
                            return Ok((idx + chr.len_utf8(), None));
                        }
                        state = State::EndOfValue;
                    } else if *escaped {
                        *escaped = false;
                    } else {
                        *escaped = chr == '\\';
                    }
                }
                State::LiteralValue { start } => {
                    // TODO '}' count
                    if let '\n' | ']' | ',' | '}' = chr {
                        let parsed = &on[start..idx];
                        let value: RootYAMLValue = match parsed.trim() {
                            "true" => RootYAMLValue::Boolean(true),
                            "false" => RootYAMLValue::Boolean(false),
                            "null" => RootYAMLValue::Null,
                            value => {
                                // TODO number check
                                RootYAMLValue::String(value)
                            }
                        };
                        let res = cb(key_chain, value);
                        if res.is_some() {
                            return Ok((idx + chr.len_utf8(), res));
                        }
                        state = State::EndOfValue;
                        end_of_value(idx, chr, &mut state, key_chain, options.allow_comments)?;
                        if key_chain.len() == current_len {
                            return Ok((idx + chr.len_utf8(), None));
                        }
                    }
                }
                State::EndOfValue => {
                    end_of_value(idx, chr, &mut state, key_chain, options.allow_comments)?;

                    if key_chain.len() == current_len {
                        return Ok((idx + chr.len_utf8(), None));
                    }
                }
                // TODO I don't think this exists
                State::Comment {
                    ref mut last_was_asterisk,
                    ref mut multiline,
                    hash,
                    start,
                } => {
                    if chr == '\n' && !*multiline {
                        if let Some(YAMLKey::Index { .. }) = key_chain.last() {
                            state = State::ExpectingValue;
                        } else {
                            state = State::InObject;
                        }
                    } else if chr == '*' && start + 1 == idx && !hash {
                        *multiline = true;
                    } else if *multiline {
                        if *last_was_asterisk && chr == '/' {
                            if let Some(YAMLKey::Index { .. }) = key_chain.last() {
                                state = State::ExpectingValue;
                            } else {
                                state = State::InObject;
                            }
                        } else {
                            *last_was_asterisk = chr == '*';
                        }
                    }
                }
            }
        }

        match state {
            State::InKey { .. } | State::StringValue { .. } => {
                todo!()
                // return Err(YAMLParseError {
                //     at: on.len(),
                //     reason: YAMLParseErrorReason::ExpectedQuote,
                // })
            }
            State::Colon => {
                return Err(YAMLParseError {
                    at: on.len(),
                    reason: YAMLParseErrorReason::ExpectedColon,
                });
            }
            State::Comment { multiline, .. } => {
                if multiline {
                    todo!();
                    // return Err(YAMLParseError {
                    //     at: on.len(),
                    //     reason: YAMLParseErrorReason::ExpectedEndOfMultilineComment,
                    // });
                }
            }
            State::EndOfValue | State::ExpectingValue => {
                if !key_chain.is_empty() {
                    // dbg!(&key_chain);
                    return Err(YAMLParseError {
                        at: on.len(),
                        reason: YAMLParseErrorReason::ExpectedBracket,
                    });
                }
            }
            State::InObject => {
                // dbg!("in object");
                return Err(YAMLParseError {
                    at: on.len(),
                    reason: YAMLParseErrorReason::ExpectedBracket,
                });
            }
            State::LiteralValue { start } => {
                let _result = cb(key_chain, RootYAMLValue::String(&on[start..]));
            }
        }

        Ok((on.len(), None))
    }

    // TODO always pops from key_chain **unless** we are in an array.
    // TODO there are complications using this in an iterator when we yielding numbers
    fn end_of_value(
        idx: usize,
        chr: char,
        state: &mut State,
        key_chain: &mut Vec<YAMLKey<'_>>,
        allow_comments: bool,
    ) -> Result<(), YAMLParseError> {
        if let ',' = chr {
            if let Some(YAMLKey::Index {
                index,
                bracketed: _,
            }) = key_chain.last_mut()
            {
                *index += 1;
                *state = State::ExpectingValue;
                return Ok(());
            }

            // TODO check here
            let _popped = key_chain.pop();
            *state = State::InObject;
        } else if let ('}', Some(YAMLKey::Slice(..))) = (chr, key_chain.last()) {
            // TODO errors here if index
            let _popped = key_chain.pop();
        } else if let (
            ']',
            Some(YAMLKey::Index {
                bracketed: true, ..
            }),
        ) = (chr, key_chain.last())
        {
            // TODO errors here if slice etc
            key_chain.pop();
        } else if let (true, c @ ('/' | '#')) = (allow_comments, chr) {
            *state = State::Comment {
                last_was_asterisk: false,
                start: idx,
                multiline: false,
                hash: c == '#',
            };
        } else if !chr.is_whitespace() {
            // dbg!(chr, key_chain);
            return Err(YAMLParseError {
                at: idx,
                reason: YAMLParseErrorReason::ExpectedEndOfValue,
            });
        }

        Ok(())
    }
}
