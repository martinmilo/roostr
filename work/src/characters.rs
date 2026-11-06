//! Characters that do not show as themselves: what `validate` reports, how text output writes
//! them, and what the project map leaves out.

use std::borrow::Cow;

/// U+1F3F4, the waving black flag: with the tag characters of a region, a region's flag.
const BLACK_FLAG: char = '\u{1F3F4}';

/// The tag characters after U+1F3F4 that make the flags of England, Scotland and Wales, the
/// three Unicode recommends for general interchange, each ended by U+E007F (cancel tag).
const FLAG_TAGS: [&str; 3] = [
    "\u{E0067}\u{E0062}\u{E0065}\u{E006E}\u{E0067}\u{E007F}",
    "\u{E0067}\u{E0062}\u{E0073}\u{E0063}\u{E0074}\u{E007F}",
    "\u{E0067}\u{E0062}\u{E0077}\u{E006C}\u{E0073}\u{E007F}",
];

/// Each character of `text`, with whether `validate` reports it: those that are invisible or
/// change how text shows, and that nobody writes on purpose.
fn each(text: &str, mut visit: impl FnMut(char, bool)) {
    let mut rest = text;
    let mut previous = None;
    while let Some(ch) = rest.chars().next() {
        rest = &rest[ch.len_utf8()..];
        if ch == BLACK_FLAG {
            if let Some(tags) = FLAG_TAGS.iter().find(|tags| rest.starts_with(*tags)) {
                visit(ch, false);
                tags.chars().for_each(|tag| visit(tag, false));
                rest = &rest[tags.len()..];
                previous = tags.chars().last();
                continue;
            }
        }
        visit(ch, is_reported(ch, previous));
        previous = Some(ch);
    }
}

fn is_reported(ch: char, previous: Option<char>) -> bool {
    match ch {
        '\t' | '\n' => false,
        '\u{202A}'..='\u{202E}'
        | '\u{2066}'..='\u{2069}'
        | '\u{200B}'
        | '\u{2060}'..='\u{2065}'
        | '\u{206A}'..='\u{206F}'
        | '\u{FEFF}'
        | '\u{180E}'
        | '\u{FFF0}'..='\u{FFFB}'
        | '\u{2028}'
        | '\u{2029}'
        // Hangul fillers and Khmer vowels that show as nothing.
        | '\u{115F}'
        | '\u{1160}'
        | '\u{3164}'
        | '\u{FFA0}'
        | '\u{17B4}'
        | '\u{17B5}'
        // The tag characters, and after them the code points kept unassigned and ignorable.
        | '\u{E0000}'..='\u{E00FF}'
        | '\u{E01F0}'..='\u{E0FFF}' => true,
        // One after a character writes an emoji or a CJK variant; a second says nothing.
        ch if is_variation_selector(ch) => previous.is_some_and(is_variation_selector),
        ch => ch.is_control(),
    }
}

/// The characters `validate` reports in `text`, each once, in the order they first appear.
pub(crate) fn reported(text: &str) -> Vec<char> {
    let mut found = Vec::new();
    each(text, |ch, report| {
        if report && !found.contains(&ch) {
            found.push(ch);
        }
    });
    found
}

/// What a finding says of the characters found: `holds U+202E, a character that is invisible
/// or changes how text shows`.
fn holds(found: &[char]) -> Option<(String, bool)> {
    let names: Vec<String> = found
        .iter()
        .map(|&ch| format!("U+{:04X}", u32::from(ch)))
        .collect();
    match names.split_last()? {
        (only, []) => Some((
            format!("holds {only}, a character that is invisible or changes how text shows"),
            false,
        )),
        (last, rest) => Some((
            format!(
                "holds {} and {last}, characters that are invisible or change how text shows",
                rest.join(", ")
            ),
            true,
        )),
    }
}

/// The finding about the characters `reported` found in a string.
pub(crate) fn finding(found: &[char]) -> String {
    holds(found).map_or_else(String::new, |(holds, several)| {
        format!("{holds}: remove {}", if several { "them" } else { "it" })
    })
}

/// The finding about the characters `changed` found in a record's file name.
pub(crate) fn file_name_finding(found: &[char]) -> String {
    holds(found).map_or_else(String::new, |(holds, _)| {
        format!("the file's name {holds}: rename the file")
    })
}

/// `text` as text output writes it: each character `validate` reports as `<U+XXXX>`.
pub(crate) fn written(text: &str) -> Cow<'_, str> {
    write_out(text, false)
}

/// A file name, a path, a key or an id as text output writes it: as `written`, and a tab and
/// a line break written out too, so that it stays on its line.
pub(crate) fn written_name(text: &str) -> Cow<'_, str> {
    write_out(text, true)
}

/// A file's name, or an id, which is one, as text output writes it: each character `validate`
/// reports in a file's name written out, which is each one the map changes.
pub(crate) fn written_file_name(text: &str) -> Cow<'_, str> {
    if !text.chars().any(map_changes) {
        return Cow::Borrowed(text);
    }
    let mut result = String::with_capacity(text.len() + 16);
    for ch in text.chars() {
        if map_changes(ch) {
            result.push_str(&format!("<U+{:04X}>", u32::from(ch)));
        } else {
            result.push(ch);
        }
    }
    Cow::Owned(result)
}

fn write_out(text: &str, name: bool) -> Cow<'_, str> {
    let mut any = false;
    each(text, |ch, report| {
        any |= report || (name && matches!(ch, '\t' | '\n' | '\r'));
    });
    if !any {
        return Cow::Borrowed(text);
    }
    let mut result = String::with_capacity(text.len() + 16);
    each(text, |ch, report| {
        if report || (name && matches!(ch, '\t' | '\n' | '\r')) {
            result.push_str(&format!("<U+{:04X}>", u32::from(ch)));
        } else {
            result.push(ch);
        }
    });
    Cow::Owned(result)
}

/// `text` as the project map holds it: a tab and every line break one space, so that an item
/// stays on its line, and what does not show left out.
pub(crate) fn shown(text: &str) -> String {
    let mut result = String::with_capacity(text.len());
    let mut chars = text.chars().peekable();
    while let Some(ch) = chars.next() {
        match ch {
            '\r' => {
                chars.next_if_eq(&'\n');
                result.push(' ');
            }
            '\t' | '\n' | '\u{2028}' | '\u{2029}' => result.push(' '),
            ch if is_removed(ch) => {}
            ch => result.push(ch),
        }
    }
    result
}

/// The characters of `text` that the map removes or changes, each once, in the order they
/// first appear: a name holding one prints in the map as another name.
pub(crate) fn changed(text: &str) -> Vec<char> {
    let mut found = Vec::new();
    for ch in text.chars().filter(|&ch| map_changes(ch)) {
        if !found.contains(&ch) {
            found.push(ch);
        }
    }
    found
}

fn map_changes(ch: char) -> bool {
    matches!(ch, '\t' | '\n' | '\r' | '\u{2028}' | '\u{2029}') || is_removed(ch)
}

/// What the map leaves out: control characters, format characters and default-ignorable code
/// points.
fn is_removed(ch: char) -> bool {
    ch.is_control() || is_format(ch) || is_default_ignorable(ch)
}

fn is_variation_selector(ch: char) -> bool {
    matches!(ch, '\u{FE00}'..='\u{FE0F}' | '\u{E0100}'..='\u{E01EF}')
}

/// Unicode's category Cf, format characters (Unicode 16).
fn is_format(ch: char) -> bool {
    matches!(
        ch,
        '\u{00AD}'
            | '\u{0600}'..='\u{0605}'
            | '\u{061C}'
            | '\u{06DD}'
            | '\u{070F}'
            | '\u{0890}'..='\u{0891}'
            | '\u{08E2}'
            | '\u{180E}'
            | '\u{200B}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'..='\u{2064}'
            | '\u{2066}'..='\u{206F}'
            | '\u{FEFF}'
            | '\u{FFF9}'..='\u{FFFB}'
            | '\u{110BD}'
            | '\u{110CD}'
            | '\u{13430}'..='\u{1343F}'
            | '\u{1BCA0}'..='\u{1BCA3}'
            | '\u{1D173}'..='\u{1D17A}'
            | '\u{E0001}'
            | '\u{E0020}'..='\u{E007F}'
    )
}

/// Unicode's property Default_Ignorable_Code_Point (DerivedCoreProperties.txt, Unicode 18).
fn is_default_ignorable(ch: char) -> bool {
    matches!(
        ch,
        '\u{00AD}'
            | '\u{034F}'
            | '\u{061C}'
            | '\u{115F}'..='\u{1160}'
            | '\u{17B4}'..='\u{17B5}'
            | '\u{180B}'..='\u{180F}'
            | '\u{200B}'..='\u{200F}'
            | '\u{202A}'..='\u{202E}'
            | '\u{2060}'..='\u{206F}'
            | '\u{3164}'
            | '\u{FE00}'..='\u{FE0F}'
            | '\u{FEFF}'
            | '\u{FFA0}'
            | '\u{FFF0}'..='\u{FFF8}'
            | '\u{1BCA0}'..='\u{1BCA3}'
            | '\u{1D173}'..='\u{1D17A}'
            | '\u{E0000}'..='\u{E0FFF}'
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    const ENGLAND: &str = "\u{1F3F4}\u{E0067}\u{E0062}\u{E0065}\u{E006E}\u{E0067}\u{E007F}";
    const SCOTLAND: &str = "\u{1F3F4}\u{E0067}\u{E0062}\u{E0073}\u{E0063}\u{E0074}\u{E007F}";
    const WALES: &str = "\u{1F3F4}\u{E0067}\u{E0062}\u{E0077}\u{E006C}\u{E0073}\u{E007F}";

    #[test]
    fn validate_reports_what_nobody_writes_on_purpose() {
        assert_eq!(
            reported("a\u{202E}b\u{200B}c\u{202E}\u{E0041}"),
            ['\u{202E}', '\u{200B}', '\u{E0041}']
        );
        assert_eq!(
            reported("\u{7}\u{1B}\u{7F}\u{85}\r"),
            ['\u{7}', '\u{1B}', '\u{7F}', '\u{85}', '\r']
        );
        assert_eq!(reported("⚠\u{FE0F}\u{FE0F}"), ['\u{FE0F}']);
        assert_eq!(
            reported("\u{115F}\u{1160}\u{3164}\u{FFA0}\u{17B4}\u{17B5}\u{2065}\u{FFF0}"),
            [
                '\u{115F}', '\u{1160}', '\u{3164}', '\u{FFA0}', '\u{17B4}', '\u{17B5}', '\u{2065}',
                '\u{FFF0}'
            ]
        );
        assert_eq!(reported("a\u{E0080}\u{E01F0}"), ['\u{E0080}', '\u{E01F0}']);
        for text in [
            "a\tb\nc",
            "⚠\u{FE0F}",
            "👩\u{200D}💻",
            "می\u{200C}خواهم",
            "\u{200E}\u{200F}\u{061C}",
            "soft\u{AD}hyphen",
            "a\u{034F}b\u{180B}\u{2800}",
            ENGLAND,
            SCOTLAND,
            WALES,
            &format!("{ENGLAND}{WALES} and {SCOTLAND}\u{FE0F}"),
        ] {
            assert!(reported(text).is_empty(), "{text:?}");
        }
    }

    #[test]
    fn tags_that_spell_no_recommended_flag_are_reported() {
        // Texas, a flag Unicode does not recommend, and England's tags without the flag.
        let texas = "\u{1F3F4}\u{E0075}\u{E0073}\u{E0074}\u{E0078}\u{E007F}";
        assert_eq!(reported(texas).len(), 5);
        assert_eq!(reported(&ENGLAND[4..]).len(), 5);
        // England's flag, then one more tag.
        assert_eq!(reported(&format!("{ENGLAND}\u{E0041}")), ['\u{E0041}']);
        // A flag cut short.
        assert_eq!(reported(&ENGLAND[..ENGLAND.len() - 4]).len(), 4);
    }

    #[test]
    fn the_finding_names_each_character() {
        assert_eq!(
            finding(&['\u{202E}']),
            "holds U+202E, a character that is invisible or changes how text shows: remove it"
        );
        assert_eq!(
            finding(&['\u{202E}', '\u{200B}', '\u{E0041}']),
            "holds U+202E, U+200B and U+E0041, characters that are invisible or change how \
             text shows: remove them"
        );
        assert_eq!(
            file_name_finding(&['\u{1B}', '\u{200B}']),
            "the file's name holds U+001B and U+200B, characters that are invisible or change \
             how text shows: rename the file"
        );
    }

    #[test]
    fn text_output_writes_out_what_validate_reports() {
        assert_eq!(
            written("Refund\u{202E} a\tcancelled\u{1B}[31m booking\n"),
            "Refund<U+202E> a\tcancelled<U+001B>[31m booking\n"
        );
        assert_eq!(
            written_name("refund\tbook\ning\r"),
            "refund<U+0009>book<U+000A>ing<U+000D>"
        );
        assert_eq!(written("tag\u{E0049}"), "tag<U+E0049>");
        assert!(matches!(written(ENGLAND), Cow::Borrowed(_)));
        assert_eq!(written("⚠\u{FE0F}\u{FE0F}"), "⚠\u{FE0F}<U+FE0F>");
        assert_eq!(
            written_file_name("re\u{AD}fund\u{200D}\t\u{202E}é.yml"),
            "re<U+00AD>fund<U+200D><U+0009><U+202E>é.yml"
        );
        assert!(matches!(
            written_file_name("refund-booking"),
            Cow::Borrowed(_)
        ));
    }

    #[test]
    fn the_map_shows_what_shows() {
        assert_eq!(shown("a\tb\r\nc\rd\ne\u{2028}f\u{2029}g"), "a b c d e f g");
        assert_eq!(
            shown("soft\u{AD}\u{200B}\u{7F}\u{202E}\u{85}\u{E0041}\u{FEFF}\u{2066}\u{1B}\u{7}x"),
            "softx"
        );
        assert_eq!(shown("⚠\u{FE0F} 👩\u{200D}💻 café"), "⚠ 👩💻 café");
        assert_eq!(
            shown(
                "R\u{3164}e\u{115F}f\u{1160}u\u{FFA0}n\u{17B4}d\u{17B5}\u{034F}\u{180B}\u{180F}\u{2800}"
            ),
            "Refund\u{2800}"
        );
        assert_eq!(shown(SCOTLAND), "\u{1F3F4}");
    }

    #[test]
    fn a_name_the_map_changes() {
        assert_eq!(changed("refund-booking"), []);
        assert_eq!(
            changed("re\u{1B}fund\u{200B}\t\u{1B}\u{AD}"),
            ['\u{1B}', '\u{200B}', '\t', '\u{AD}']
        );
    }
}
