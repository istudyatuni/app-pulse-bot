pub const CALLBACK_VERSION: u8 = 1;

const SEP: &str = ":";

pub(crate) trait PayloadData {
    type Error;

    fn to_payload(&self) -> String;
    fn try_from_payload(payload: &str) -> Result<Self, Self::Error>
    where
        Self: Sized;
}

/// Describes how to parse and construct layout of callback payload
///
/// ```
/// # use bot_handlers::{PayloadLayout, CALLBACK_VERSION};
/// let layout = PayloadLayout::new(3, Some(1));
///
/// let fields = vec!["test", "test:1", "text"];
///
/// let s = layout.make_payload(fields.clone()).unwrap();
/// assert_eq!(s, format!("{CALLBACK_VERSION}:test:test:1:text"));
///
/// assert_eq!(layout.parse_payload(&s).unwrap(), fields);
/// ```
// todo: fix test above
#[derive(Debug)]
pub struct PayloadLayout {
    /// Number of elements in payload
    size: usize,
    /// Element which can contain colon (used as separator), indexed from 0.
    /// It's not possible to parse payload with multiple such parts
    raw_element: Option<usize>,
}

impl PayloadLayout {
    pub const fn new(size: usize, raw_element: Option<usize>) -> Self {
        Self { size, raw_element }
    }
    pub fn make_payload(&self, elements: Vec<&str>) -> Result<String, PayloadParseError> {
        if elements.len() != self.size {
            return Err(PayloadParseError::InvalidSize);
        }

        Ok([CALLBACK_VERSION.to_string().as_str()]
            .into_iter()
            .chain(elements.iter().copied())
            .collect::<Vec<_>>()
            .join(SEP))
    }
    pub fn parse_payload(&self, payload: &str) -> Result<Vec<String>, PayloadParseError> {
        let payload = payload
            .split(SEP)
            .skip(1)
            .map(ToOwned::to_owned)
            .collect::<Vec<String>>();

        let size = payload.len();
        if self.size > size {
            return Err(PayloadParseError::InvalidSize);
        }

        let res = match self.raw_element {
            Some(i) if self.size < size => {
                // [0]:[1]:[2]:[3]:[4], raw_element == 1 -> [0], [1]:[2], [3], [4]
                // size = 5
                // real size = 5 - 1
                // take(raw_element)
                // real size = real size - (raw_element - 1)
                // take(real size - 1).join(SEP)
                // take remainder
                let mut res = Vec::with_capacity(size);
                res.extend_from_slice(&payload[0..i]);
                // size - 1 - (i - 1);
                let raw_end = size - i;
                res.push(payload[i..raw_end].join(SEP));
                res.extend_from_slice(&payload[raw_end..]);
                res
            }
            Some(_) | None => {
                if self.size < size {
                    return Err(PayloadParseError::InvalidSize);
                }
                payload
            }
        };
        Ok(res)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, thiserror::Error)]
pub enum PayloadParseError {
    #[error("invalid payload size")]
    InvalidSize,
}

#[cfg(test)]
mod tests {
    use super::*;

    use PayloadParseError::*;

    #[test]
    fn test_parse_payload_layout() {
        let size = 5;
        let layout = PayloadLayout::new(size, Some(2));
        let layout_simple = PayloadLayout::new(size, None);

        // version is added in loop
        let table: &[(_, Result<&[_], PayloadParseError>)] = &[
            ("", Err(InvalidSize)),
            ("0", Err(InvalidSize)),
            ("0:1", Err(InvalidSize)),
            ("0:1:2", Err(InvalidSize)),
            ("0:1:2:3", Err(InvalidSize)),
            ("0:1:2:3:4", Ok(&["0", "1", "2", "3", "4"])),
            ("0:1:2:3:4:5", Ok(&["0", "1", "2:3", "4", "5"])),
            ("0:1:2:3:4:5:6", Ok(&["0", "1", "2:3:4", "5", "6"])),
        ];
        for (input, expected) in table {
            eprintln!("running for {input}");

            let input = format!("{CALLBACK_VERSION}:{input}");
            let expected = expected
                .as_ref()
                .map(|v| v.iter().map(|s| s.to_string()).collect::<Vec<_>>())
                .map_err(|e| *e);

            let res = layout.parse_payload(&input);
            assert_eq!(res, expected);

            let res_simple = layout_simple.parse_payload(&input);
            // wtf, why Result moved in is_ok_and?
            if res_simple.clone().is_ok_and(|v| v.len() == size) {
                assert_eq!(res_simple, expected);
            } else {
                assert_eq!(res_simple, Err(InvalidSize));
            }
        }
    }
}
