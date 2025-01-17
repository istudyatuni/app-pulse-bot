/// Modified version of [`teloxide::utils::markdown::escape`]
pub(crate) fn escape<S: Into<String>>(s: S) -> String {
    const CHARS: [char; 16] = [
        /*'_', '*',*/ '[', ']', '(', ')', '~', '`', '>', '#', '+', '-', '=', '|', '{', '}', '.', '!',
    ];

    let s = s.into();
    s.chars().fold(String::with_capacity(s.len()), |mut s, c| {
        if CHARS.contains(&c) {
            s.push('\\');
        }
        s.push(c);
        s
    })
}
