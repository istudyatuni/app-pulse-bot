use camino::Utf8PathBuf as PathBuf;
use fluent::FluentResource;
use fluent_syntax::ast::{Entry, Expression, InlineExpression, Message, Pattern, PatternElement};
use unic_langid::LanguageIdentifier;

#[derive(Debug, Clone)]
pub(crate) struct LangInfo {
    pub(crate) messages: Vec<MessageInfo>,
    pub(crate) source: String,
}

#[derive(Debug, Clone)]
pub(crate) struct MessageInfo {
    pub(crate) id: String,
    pub(crate) attrs: Vec<String>,
}

pub(crate) fn extract_messages(
    locales: Vec<(PathBuf, String)>,
) -> Result<Vec<(LanguageIdentifier, LangInfo)>, String> {
    let mut res = vec![];
    for (path, source) in locales {
        let resource = FluentResource::try_new(source.clone()).map_err(|(_, e)| {
            e.iter()
                .map(|e| e.to_string())
                .collect::<Vec<_>>()
                .join("\n")
        })?;

        let mut messages = vec![];

        for e in resource.entries() {
            match e {
                Entry::Message(Message {
                    id,
                    value: Some(Pattern { elements }),
                    ..
                }) => {
                    let mut attrs = vec![];
                    for e in elements {
                        match e {
                            PatternElement::Placeable {
                                expression:
                                    Expression::Inline(InlineExpression::VariableReference {
                                        id, ..
                                    }),
                            } => {
                                attrs.push(id.name.to_owned());
                            }
                            _ => (),
                        }
                    }
                    messages.push(MessageInfo {
                        id: id.name.to_owned(),
                        attrs,
                    });
                }
                _ => (),
            }
        }

        let lang = match extract_lang(path.clone()) {
            Ok(i) => i,
            Err(e) => return Err(e),
        };
        res.push((
            lang,
            LangInfo {
                messages: messages.clone(),
                source,
            },
        ));
    }

    Ok(res)
}

fn extract_lang(path: PathBuf) -> Result<LanguageIdentifier, String> {
    if !matches!(path.extension(), Some("ftl")) {
        return Err(format!(
            "something went wrong, expected .ftl file, but got \"{path}\""
        ));
    }

    path.file_name()
        .expect("path without file_name")
        .strip_suffix(".ftl")
        .unwrap()
        .parse::<LanguageIdentifier>()
        .map_err(|e| e.to_string())
}
