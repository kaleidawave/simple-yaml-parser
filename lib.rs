#[derive(Debug, Clone, PartialEq, Eq)]
pub enum YAMLKey<'a> {
    Slice(&'a str),
    // TODO Index(usize) with data on outside
    Index { index: usize, bracketed: bool },
}

#[derive(Debug, PartialEq, Eq)]
pub enum RootYAMLValue<'a> {
    String(YAMLString<'a>),
    Number(YAMLNumber<'a>),
    Comment(&'a str),
    Boolean(bool),
    EmptyArray,
    EmptyObject,
    Empty,
    Null,
}

impl<'a> RootYAMLValue<'a> {
    #[must_use]
    pub fn string_value(&self) -> Option<&'a str> {
        if let RootYAMLValue::String(value) = self {
            Some(value.0)
        } else {
            None
        }
    }
}

// TODO record quoted vs bare. etc
#[derive(PartialEq, Eq, Clone, Copy)]
pub struct YAMLString<'a>(&'a str);

impl<'a> std::fmt::Debug for YAMLString<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_fmt(format_args!("{:?}", self.0))
    }
}

#[derive(PartialEq, Eq, Clone, Copy)]
pub struct YAMLNumber<'a>(&'a str);

impl<'a> std::fmt::Debug for YAMLNumber<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> Result<(), std::fmt::Error> {
        f.write_fmt(format_args!("{}", self.0))
    }
}

#[derive(Debug)]
pub enum YAMLParseErrorReason {
    ExpectedColon,
    ExpectedEndOfValue,
    ExpectedBracket,
    ExpectedTrueFalseNull,
    ExpectedValue,
    ExpectedKey,
    ExpectedEndOfMultilineComment,
    ExpectedQuote,
    InvalidComment,
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

/// If you want to return early (not parse the whole input) use [`parse_with_options`]
///
/// # Errors
/// Returns an error if it tries to parse invalid YAML input
pub fn parse<'a>(
    on: &'a str,
    mut cb: impl for<'b> FnMut(&'b [YAMLKey<'a>], RootYAMLValue<'a>),
) -> Result<(), YAMLParseError> {
    parse_with_options::<()>(on, ParseOptions::default(), |k, v| {
        cb(k, v);
        None
    })
    .map(|_none| ())
}

/// For `|` and `>` based values
#[derive(Debug, PartialEq, Eq)]
pub struct MultiLineString<'a> {
    pub on: &'a str,
    /// replace new lines with spaces. Done using `>` rather than `|`
    pub collapse: bool,
    /// with `|+` etc
    pub preserve_leading_whitespace: bool,
}

#[derive(Debug, Clone, Copy)]
pub struct ParseOptions {
    pub tab_size: usize,
    pub general_boolean_values: bool,
}

impl Default for ParseOptions {
    fn default() -> Self {
        Self {
            tab_size: 2,
            general_boolean_values: false,
        }
    }
}

/// # Errors
/// Returns an error if it tries to parse invalid YAML input
#[allow(clippy::too_many_lines)]
pub fn parse_with_options<'a, T>(
    on: &'a str,
    options: ParseOptions,
    mut cb: impl for<'b> FnMut(&'b [YAMLKey<'a>], RootYAMLValue<'a>) -> Option<T>,
) -> Result<Option<T>, YAMLParseError> {
    #[derive(Debug)]
    enum State {
        Skip,
        Entry,
    }

    let mut key_chain = Vec::new();
    let mut state = State::Skip;
    // let mut list_idx: usize = 0;
    let mut indent = 0;

    let mut idx = 0;

    let bytes = on.as_bytes();

    macro_rules! skip_whitespace_find_comments {
        () => {
            while let Some(byte) = bytes.get(idx) {
                if let b' ' | b'\t' | b'\r' | b'\n' = byte {
                    idx += 1;
                }
                // else if let b'/' | b'#' = byte {
                //     let Some((comment, offset)) = parse_comment(&on[idx..]) else {
                //         return_err!(InvalidComment);
                //     };
                //     idx += offset;
                //     if options.yield_comments {
                //         emit!(RootYAMLValue::Comment(comment));
                //     }
                // }
                else {
                    break;
                }
            }
        };
    }

    while idx < on.len() {
        match state {
            State::Skip => {
                let byte = bytes[idx];
                if let b'\t' = byte {
                    indent += options.tab_size;
                    idx += 1;
                } else if let b' ' = byte {
                    indent += 1;
                    idx += 1;
                } else if let b'\n' | b'\r' = byte {
                    indent = 0;
                    idx += 1;
                } else if let b'-' = byte {
                    if on[idx..].starts_with("---") && indent == 0 {
                        idx += 3;
                        key_chain.clear();
                        // TODO emit.new_document?
                        // if let [YAMLKey::Index { index, .. }] = key_chain.as_mut() {
                        //     *index += 1;
                        // } else if on == 0 {
                        //     key_chain.push(YAMLKey::Index {
                        //         index: 0,
                        //         bracketed: false,
                        //     });
                        // } else {
                        //     todo!("error");
                        // }
                    } else {
                        let current_level = (indent / options.tab_size) + 2;
                        if current_level < key_chain.len() {
                            drop(key_chain.drain(current_level..));
                        }

                        // TODO ordering here is borked
                        if let Some(YAMLKey::Index { index, .. }) = key_chain.last_mut() {
                            *index += 1;
                        } else {
                            key_chain.push(YAMLKey::Index {
                                index: 0,
                                bracketed: false,
                            });
                        }
                        state = State::Entry;
                        idx += 1;
                        skip_whitespace_find_comments!();
                        // dbg!(&on[idx..], &state);
                        // TODO wip
                        indent += 2;
                    }
                } else if let b'#' = byte {
                    let rest = &on[idx..][1..];
                    let (comment, _) = &rest.split_once('\n').unwrap_or((rest, ""));
                    idx += 1 + comment.len();
                } else {
                    state = State::Entry;
                    let current_level = indent / options.tab_size;
                    let key_chain_level = key_chain
                        .iter()
                        .filter(|key| matches!(key, YAMLKey::Slice(_)))
                        .count();

                    // Indentation of key tracking
                    {
                        if current_level < key_chain_level {
                            let after = current_level;
                            drop(key_chain.drain(after..));
                            // match key_chain.last() {
                            //     Some(YAMLKey::Index { index, .. }) => {
                            //         list_idx = *index;
                            //     }
                            //     _ => {
                            //         list_idx = 0;
                            //     }
                            // }
                        }
                    }
                    // indent = 0;
                }
            }
            State::Entry => {
                let rest = &on[idx..];
                // TODO parse identifier (with quotes etc)
                let offset = rest.find([':', '\n']).unwrap_or(rest.len() - 1);

                if rest.as_bytes()[offset] == b':' {
                    let key_len = offset;

                    // If unquoted then can trim
                    let key = YAMLKey::Slice(&rest[..key_len].trim());
                    key_chain.push(key);
                    idx += key_len + 1;

                    let after: &str = &on[idx..];
                    let (header, rest) = after.split_once('\n').unwrap_or((after, ""));

                    // If not nested
                    if !header.trim().is_empty() {
                        // TODO comment heres?
                        let string_modifier = match header.trim() {
                            "|" => Some((true, false)),
                            ">" => Some((false, false)),
                            _ => None,
                        };

                        if let Some((_preserve_new_lines, _preserve_leading_whitespace)) =
                            string_modifier
                        {
                            idx += header.len() + 1;
                            let indent = indent + 2;
                            let offset =
                                value::parse_string(rest, indent, options.tab_size, false, false);
                            let string = &rest[..offset];
                            // dbg!((indent, preserve_new_lines, preserve_leading_whitespace), string);
                            // TODO pass options
                            let _ = cb(&key_chain, RootYAMLValue::String(YAMLString(string)));
                            idx += offset;
                        } else {
                            let (value_len, _) =
                                value::parse(after, indent, &mut key_chain, options, &mut cb)?;
                            idx += value_len;
                            key_chain.pop();
                            // dbg!(&on[idx..]);
                        }
                    }
                } else {
                    let (value_len, _) =
                        value::parse(rest, indent + 2, &mut key_chain, options, &mut cb)?;
                    idx += value_len;
                    // // in list
                    // if list && rest[..offset].is_empty() {
                    //     idx += offset;
                    // } else {
                    // }
                }

                state = State::Skip;
            }
        }
    }

    if let State::Skip = state {
        Ok(None)
    } else {
        println!("error");
        Ok(None)
    }
}

fn get_indent_size(on: &str, tab_size: usize) -> usize {
    let mut idx = 0;
    for chr in on.chars() {
        if let ' ' = chr {
            idx += 1;
        } else if let '\t' = chr {
            idx += tab_size;
        } else {
            break;
        }
    }
    idx
}

fn parse_key(on: &str) -> (usize, &str) {
    if let Some(after) = on.strip_prefix('"') {
        let (key, _) = after.split_once('"').unwrap();
        (2 + key.len(), key)
    } else {
        let (key, _) = on.split_once(':').unwrap();
        (key.len(), key)
    }
}

pub mod value {
    // WHITESPACE
    use super::{
        RootYAMLValue, YAMLKey, YAMLNumber, YAMLParseError, YAMLParseErrorReason, YAMLString,
    };

    pub(crate) fn parse_string(
        on: &str,
        indent: usize,
        tab_size: usize,
        handle_comments: bool,
        in_object: bool,
    ) -> usize {
        let lines = on.lines();
        // let first = lines.next().expect("no lines");
        let mut last = 0;
        for line in lines {
            if handle_comments && let Some((end, _comment)) = line.split_once('#') {
                return (line.as_ptr() as usize - on.as_ptr() as usize) + end.len();
            }
            // TODO escaping?
            if in_object && let Some((end, _)) = line.split_once([',', ']', '}']) {
                return (line.as_ptr() as usize - on.as_ptr() as usize) + end.len();
            }
            // important
            if line.trim().is_empty() {
                continue;
            }
            let line_indent = super::get_indent_size(line, tab_size);
            // TODO might need improvements
            if last != 0 && line_indent < indent {
                break;
            }
            last = (line.as_ptr() as usize - on.as_ptr() as usize) + line.len();
        }
        last
    }

    /// # Errors
    /// Returns an error if it tries to parse invalid YAML input
    #[allow(clippy::too_many_lines)]
    pub fn parse<'a, 'l, T>(
        on: &'a str,
        // mut indent: usize,
        indent: usize,
        key_chain: &'l mut Vec<YAMLKey<'a>>,
        options: super::ParseOptions,
        cb: &'l mut impl for<'c> FnMut(&'c [YAMLKey<'a>], RootYAMLValue<'a>) -> Option<T>,
    ) -> Result<(usize, Option<T>), YAMLParseError> {
        fn find_non_escaped_quoted(on: &str) -> Option<usize> {
            on.match_indices('"').find_map(|(idx, _)| {
                let rev = on[..idx].bytes();
                let last = rev.rev().take_while(|chr: &u8| *chr == b'\\').count();
                (last % 2 == 0).then_some(idx)
            })
        }

        fn parse_comment(on: &str) -> Option<(&str, usize)> {
            if let Some(rest) = on.strip_prefix('#') {
                let offset = rest.find('\n').unwrap_or(rest.len());
                Some((&rest[..offset], offset + 1))
            } else if let Some(rest) = on.strip_prefix("//") {
                let offset = rest.find('\n').unwrap_or(rest.len());
                Some((&rest[..offset], offset + 2))
            } else if let Some(rest) = on.strip_prefix("/*") {
                let offset = rest.find("*/")?;
                Some((&rest[..offset], offset + 4))
            } else {
                None
            }
        }

        fn parsed_slice_as_value<'a>(
            parsed: &'a str,
            general_boolean_values: bool,
        ) -> RootYAMLValue<'a> {
            assert!(!parsed.is_empty());
            match parsed.trim() {
                "true" => RootYAMLValue::Boolean(true),
                "false" => RootYAMLValue::Boolean(false),
                "y" | "Y" | "yes" | "Yes" | "YES" | "True" | "TRUE" | "on" | "On" | "ON"
                    if general_boolean_values =>
                {
                    RootYAMLValue::Boolean(true)
                }
                "n" | "N" | "no" | "No" | "NO" | "False" | "FALSE" | "off" | "Off" | "OFF"
                    if general_boolean_values =>
                {
                    RootYAMLValue::Boolean(false)
                }
                "null" => RootYAMLValue::Null,
                // TODO if partial_syntax, else error...?
                // "" => RootYAMLValue::Empty,
                value => {
                    // TODO better number check
                    let trimmed = value.trim();
                    if trimmed.chars().all(|chr| matches!(chr, '0'..='9' | '.')) {
                        RootYAMLValue::Number(YAMLNumber(trimmed))
                    } else {
                        RootYAMLValue::String(YAMLString(value))
                    }
                }
            }
        }

        let (mut idx, mut in_object): (usize, bool) = Default::default();
        let bytes = on.as_bytes();
        let parent_len = key_chain.len();

        // if tls.is_some() {
        //     key_chain.push(YAMLKey::Index(0));
        // }

        macro_rules! emit {
            ($item:expr) => {
                // let res = visitor.callback(&key_chain, $item, idx);
                let res = cb(&key_chain, $item);
                if res.is_some() {
                    return Ok((idx, res));
                }
            };
        }

        macro_rules! return_err {
            ($reason:ident) => {
                return Err(YAMLParseError {
                    at: idx,
                    reason: YAMLParseErrorReason::$reason,
                });
            };
        }

        macro_rules! skip_whitespace_find_comments {
            () => {
                while let Some(byte) = bytes.get(idx) {
                    if let b' ' | b'\t' | b'\r' | b'\n' = byte {
                        idx += 1;
                    }
                    // else if let b'/' | b'#' = byte {
                    //     let Some((comment, offset)) = parse_comment(&on[idx..]) else {
                    //         return_err!(InvalidComment);
                    //     };
                    //     idx += offset;
                    //     if options.yield_comments {
                    //         emit!(RootYAMLValue::Comment(comment));
                    //     }
                    // }
                    else {
                        break;
                    }
                }
            };
        }

        skip_whitespace_find_comments!();

        while idx < bytes.len() {
            if in_object {
                skip_whitespace_find_comments!();
                let (offset, key) = super::parse_key(&on[idx..]);
                key_chain.push(YAMLKey::Slice(key));
                idx += offset;
                skip_whitespace_find_comments!();
                if on.as_bytes().get(idx).copied().unwrap_or_default() != b':' {
                    // TODO partial could find next ':'?
                    return_err!(ExpectedColon);
                }
                idx += 1;
            }

            skip_whitespace_find_comments!();

            match bytes.get(idx).copied() {
                Some(b'{') => {
                    idx += 1;
                    // little hack
                    skip_whitespace_find_comments!();
                    if let Some(b'}') = bytes.get(idx) {
                        idx += 1;
                        emit!(RootYAMLValue::EmptyObject);
                    } else {
                        in_object = true;
                        // visitor.start(true, &mut idx);
                        continue;
                    }
                }
                Some(b'[') => {
                    idx += 1;
                    key_chain.push(YAMLKey::Index {
                        index: 0,
                        bracketed: true,
                    });
                    in_object = false;
                    // visitor.start(false, &mut idx);
                    continue;
                }
                Some(b']') => {
                    idx += 1;
                    match key_chain.pop() {
                        Some(YAMLKey::Index {
                            index: 0,
                            bracketed: true,
                        }) => {
                            emit!(RootYAMLValue::EmptyArray);
                        }
                        // Some(YAMLKey::Index(_)) if options.allow_trailing_commas => {}
                        _ => {
                            return_err!(ExpectedEndOfValue);
                        }
                    }
                    in_object = matches!(key_chain.last(), Some(YAMLKey::Slice(_)));
                }
                Some(b'"') => {
                    let rest = &on[idx..][1..];
                    let Some(offset) = find_non_escaped_quoted(rest) else {
                        return_err!(ExpectedEndOfValue);
                    };
                    idx += offset + 2;
                    emit!(RootYAMLValue::String(YAMLString(&rest[..offset])));
                }
                // Some(b @ (b',' | b'}')) if options.partial_syntax => {
                //     emit!(RootYAMLValue::Empty);
                //     idx += 1;
                //     if in_object {
                //         let _ = key_chain.pop();
                //     }
                //     if b == b',' {
                //         continue;
                //     }
                // }
                _ => {
                    let rest = &on[idx..];
                    // TODO wip
                    let indent = indent + 2;
                    let handle_commas = key_chain.len() > parent_len;
                    let offset = parse_string(rest, indent, options.tab_size, true, handle_commas);
                    let value =
                        parsed_slice_as_value(&rest[..offset], options.general_boolean_values);
                    emit!(value);
                    idx += offset;
                    // return_err!(ExpectedValue);
                }
            }

            while let Some(byte) = bytes.get(idx) {
                if key_chain.len() <= parent_len {
                    return Ok((idx, None));
                }

                // if tls.is_some_and(|c: char| on[idx..].starts_with(c)) {
                //     idx += 1;
                //     if let [YAMLKey::Index(ref mut idx)] = key_chain.as_mut_slice() {
                //         *idx += 1;
                //         break;
                //     }
                // } else
                if let b' ' | b'\t' | b'\r' | b'\n' = byte {
                    idx += 1;
                } else if let b'/' | b'#' = byte {
                    let Some((_comment, offset)) = parse_comment(&on[idx..]) else {
                        return_err!(InvalidComment);
                    };
                    idx += offset;
                    // if options.yield_comments {
                    //     emit!(RootYAMLValue::Comment(comment));
                    // }
                } else {
                    let new_byte = if *byte == b',' {
                        idx += 1;
                        if let Some(YAMLKey::Index {
                            index,
                            bracketed: true,
                        }) = key_chain.last_mut()
                        {
                            *index += 1;
                        } else {
                            key_chain.pop();
                        }
                        // if !options.allow_trailing_commas {
                        //     break;
                        // }
                        skip_whitespace_find_comments!();
                        let Some(b @ (b'}' | b']')) = bytes.get(idx) else {
                            break;
                        };
                        *b
                    } else {
                        // visitor.end(in_object, &key_chain[..key_chain.len() - 1]);
                        *byte
                    };
                    match new_byte {
                        b'}' if *byte == b',' || in_object => {}
                        b']' if matches!(key_chain.last(), Some(YAMLKey::Index { .. })) => {}
                        _ => {
                            return_err!(ExpectedEndOfValue);
                        }
                    }
                    idx += 1;
                    // Can fail for trailing commas?
                    key_chain.pop();
                    in_object = matches!(key_chain.last(), Some(YAMLKey::Slice(_)));
                }
            }
        }

        // let tl = tls.is_some_and(|_| matches!(key_chain.as_slice(), &[YAMLKey::Index(_)]));

        // debug_assert!(key_chain.len() >= parent_len, "{key_chain:?}");
        if key_chain.len() > parent_len {
            return_err!(ExpectedBracket);
        }

        Ok((on.len(), None))
    }
}
