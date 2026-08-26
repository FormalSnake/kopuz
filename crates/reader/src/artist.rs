//! Splitting a joined artist credit into the artists it actually credits.
//!
//! A tag reading "A$AP Rocky feat. Drake" is one string, and taking it as one
//! artist gives the collaboration its own tile beside the real A$AP Rocky,
//! usually with no photo, because no such artist exists upstream.
//!
//! The rule is deliberately narrow. A wrong split invents phantom artists *and*
//! can lose the real one, which is worse than the duplicate it set out to fix,
//! so only markers that never sit inside a single name are separators. In
//! particular `&`, `+`, `/`, ` - ` and a bare comma are NOT separators at the
//! top level: "&ME", "Simon & Garfunkel", "AC/DC", "Jay-Z", "Tyler, The
//! Creator" and "Earth, Wind & Fire" all have to survive intact. Comma-joined
//! credits are instead collapsed downstream by `utils::artist`, which only
//! drops one when its primary artist independently has a tile.

/// Featuring markers, longest first so "featuring" wins over its own "feat"
/// prefix.
const FEATURE_MARKERS: [&str; 5] = ["featuring", "feat.", "feat", "ft.", "ft"];

/// Collaboration markers: co-equal credits rather than a featured guest list.
const COLLAB_MARKERS: [&str; 3] = ["vs.", "vs", "x"];

/// Words that open the tail of one name ("Tyler, The Creator") rather than the
/// next entry in a guest list.
const CONTINUATION_WORDS: [&str; 3] = ["the", "a", "an"];

/// Delimiters a tagger writes between values it already considers separate:
/// the ID3v2.4 / Vorbis multi-value separator and its CJK equivalents.
const HARD_DELIMITERS: [char; 3] = [';', '；', '、'];

/// Bullets, which some taggers use to join a whole contributor list. Counted
/// only when padded with a space on both sides: unpadded, every one of these
/// turns up inside real names (Catalan "Col·lectiu", Japanese
/// "マイケル・ジャクソン").
const BULLETS: [char; 5] = ['•', '∙', '·', '・', '･'];

/// The individual artists a credit string names.
///
/// Returns the input as a single entry when nothing marks it as a join, and
/// de-duplicates case-insensitively while keeping the first spelling seen.
pub fn split_credit(credit: &str) -> Vec<String> {
    let mut out = Vec::new();
    for part in credit.split(HARD_DELIMITERS) {
        split_part(part, &mut out);
    }
    out
}

/// The artists a track credits, preferring the source's own per-artist values
/// (a multi-value `ARTISTS` tag, Jellyfin's `Artists` array) over the joined
/// display string. Each value still goes through [`split_credit`], because a
/// multi-value field can hold one joined credit per slot.
pub fn credited(primary: &str, structured: &[String]) -> Vec<String> {
    let mut out = Vec::new();
    for value in structured {
        for part in value.split(HARD_DELIMITERS) {
            split_part(part, &mut out);
        }
    }
    if out.is_empty() {
        split_part(primary, &mut out);
    }
    out
}

fn split_part(part: &str, out: &mut Vec<String>) {
    let part = part.trim();
    if part.is_empty() {
        return;
    }
    // Bullets bind loosest of all: they join whole credits, so they have to
    // be resolved before any marker sitting inside one of those credits.
    if let Some(segments) = bullet_segments(part) {
        push_bulleted(&segments, out);
        return;
    }
    // Collab markers bind looser than "feat.": each side of an "A x B" can
    // carry its own featured list.
    if let Some((start, end)) = find_marker(part, &COLLAB_MARKERS, true) {
        split_part(&part[..start], out);
        split_part(&part[end..], out);
        return;
    }
    match find_marker(part, &FEATURE_MARKERS, false) {
        Some((start, end)) => {
            split_part(&part[..start], out);
            push_guests(&part[end..], out);
        }
        None => push_name(part, out),
    }
}

/// The segments of a bullet-joined list, or None when there is no space-padded
/// bullet to split on.
fn bullet_segments(part: &str) -> Option<Vec<&str>> {
    let mut segments = Vec::new();
    let mut start = 0;
    for (i, ch) in part.char_indices() {
        if !BULLETS.contains(&ch) {
            continue;
        }
        let end = i + ch.len_utf8();
        if !part[..i].ends_with(char::is_whitespace)
            || !part[end..].starts_with(char::is_whitespace)
        {
            continue;
        }
        segments.push(&part[start..i]);
        start = end;
    }
    if segments.is_empty() {
        return None;
    }
    segments.push(&part[start..]);
    Some(segments)
}

/// A bullet list is usually every contributor a release touched, performers and
/// songwriters and producers alike, which is more than the artists a track
/// should be filed under.
///
/// One shape gives itself away: "<credit> • <everyone who worked on it>", where
/// the credit's own artists are repeated in the tail. There the head is the
/// answer and the rest is personnel, so take only the head. Anything else is
/// split in full, because a bullet list with no such evidence is more often a
/// real collaboration than a contributor dump, and a stray songwriter tile is a
/// smaller wrong than one tile carrying the entire joined string.
fn push_bulleted(segments: &[&str], out: &mut Vec<String>) {
    if let Some((head, tail)) = segments.split_first() {
        let head_names = split_credit(head);
        if head_names.len() > 1 {
            let tail_names: Vec<String> = tail.iter().flat_map(|s| split_credit(s)).collect();
            let repeated = head_names
                .iter()
                .all(|name| tail_names.iter().any(|t| same_artist(t, name)));
            if repeated {
                for name in head_names {
                    push_name(&name, out);
                }
                return;
            }
        }
    }
    for segment in segments {
        split_part(segment, out);
    }
}

/// Everything after a featuring marker is a guest list, so a comma or an
/// ampersand in it is a separator. That inference holds only here: at the top
/// level the same characters are ordinary parts of a band name.
fn push_guests(tail: &str, out: &mut Vec<String>) {
    for segment in comma_segments(tail) {
        for name in segment.split(" & ").flat_map(|n| n.split(" and ")) {
            split_part(name, out);
        }
    }
}

fn comma_segments(tail: &str) -> Vec<String> {
    let mut segments: Vec<String> = Vec::new();
    for raw in tail.split(',') {
        let piece = raw.trim();
        if piece.is_empty() {
            continue;
        }
        if opens_continuation(piece)
            && let Some(last) = segments.last_mut()
        {
            last.push_str(", ");
            last.push_str(piece);
            continue;
        }
        segments.push(piece.to_string());
    }
    segments
}

fn opens_continuation(piece: &str) -> bool {
    piece.split_whitespace().next().is_some_and(|word| {
        CONTINUATION_WORDS
            .iter()
            .any(|w| word.eq_ignore_ascii_case(w))
    })
}

fn push_name(name: &str, out: &mut Vec<String>) {
    let name = name.trim();
    if name.is_empty() {
        return;
    }
    if !out.iter().any(|seen| same_artist(seen, name)) {
        out.push(name.to_string());
    }
}

fn same_artist(a: &str, b: &str) -> bool {
    a.to_lowercase() == b.to_lowercase()
}

/// The byte range of the first separator marker in `part`.
///
/// A marker only separates when a space precedes it, so "Taylor Swift." keeps
/// the "ft." inside "Swift" and "Jay-Z" is never touched. It must also be
/// followed by a space, unless it ends in a period: "かいりきベア feat.缶缶"
/// has none there. `require_space_after` additionally rejects a marker at the
/// very end of the string, which is what keeps "Malcolm X" whole.
fn find_marker(part: &str, markers: &[&str], require_space_after: bool) -> Option<(usize, usize)> {
    let bytes = part.as_bytes();
    for (i, ch) in part.char_indices() {
        if i == 0 || !part[..i].ends_with(char::is_whitespace) || ch.is_whitespace() {
            continue;
        }
        for marker in markers {
            let end = i + marker.len();
            if end > bytes.len() || !bytes[i..end].eq_ignore_ascii_case(marker.as_bytes()) {
                continue;
            }
            let after_ok = match part[end..].chars().next() {
                Some(next) => next.is_whitespace(),
                None => !require_space_after,
            } || (marker.ends_with('.') && end < bytes.len());
            if after_ok {
                return Some((i, end));
            }
        }
    }
    None
}

#[cfg(test)]
mod tests {
    use super::*;

    fn split(s: &str) -> Vec<String> {
        split_credit(s)
    }

    #[test]
    fn plain_names_pass_through() {
        assert_eq!(split("A$AP Rocky"), ["A$AP Rocky"]);
        assert_eq!(split("  Reol  "), ["Reol"]);
        assert_eq!(split(""), Vec::<String>::new());
    }

    #[test]
    fn featuring_markers_split() {
        assert_eq!(split("A$AP Rocky feat. Drake"), ["A$AP Rocky", "Drake"]);
        assert_eq!(split("A$AP Rocky Feat. Drake"), ["A$AP Rocky", "Drake"]);
        assert_eq!(split("A$AP Rocky FEAT Drake"), ["A$AP Rocky", "Drake"]);
        assert_eq!(split("A$AP Rocky ft. Drake"), ["A$AP Rocky", "Drake"]);
        assert_eq!(split("A$AP Rocky ft Drake"), ["A$AP Rocky", "Drake"]);
        assert_eq!(split("A$AP Rocky featuring Drake"), ["A$AP Rocky", "Drake"]);
    }

    #[test]
    fn featuring_marker_without_trailing_space_splits() {
        assert_eq!(split("かいりきベア feat.缶缶"), ["かいりきベア", "缶缶"]);
    }

    #[test]
    fn guest_list_splits_on_comma_and_ampersand() {
        assert_eq!(
            split("Kanye West feat. Jay-Z, Rihanna & Bon Iver"),
            ["Kanye West", "Jay-Z", "Rihanna", "Bon Iver"]
        );
        assert_eq!(
            split("Calvin Harris ft. Dua Lipa and Young Thug"),
            ["Calvin Harris", "Dua Lipa", "Young Thug"]
        );
    }

    #[test]
    fn guest_list_keeps_a_trailing_article_with_its_name() {
        assert_eq!(
            split("Kali Uchis feat. Tyler, The Creator"),
            ["Kali Uchis", "Tyler, The Creator"]
        );
    }

    #[test]
    fn collaboration_markers_split() {
        assert_eq!(split("Chris Brown x Tyga"), ["Chris Brown", "Tyga"]);
        assert_eq!(split("Metallica vs. Slayer"), ["Metallica", "Slayer"]);
        assert_eq!(split("Metallica vs Slayer"), ["Metallica", "Slayer"]);
    }

    #[test]
    fn hard_delimiters_split() {
        assert_eq!(
            split("Daft Punk;Pharrell Williams"),
            ["Daft Punk", "Pharrell Williams"]
        );
        assert_eq!(split("初音ミク、鏡音リン"), ["初音ミク", "鏡音リン"]);
    }

    // The names a split must never shatter. Each is a single real artist whose
    // name contains a character or word that looks like a join.
    #[test]
    fn real_names_are_never_split() {
        for name in [
            "&ME",
            "Simon & Garfunkel",
            "AC/DC",
            "Tyler, The Creator",
            "Earth, Wind & Fire",
            "Florence + the Machine",
            "Jay-Z",
            "Blink-182",
            "Malcolm X",
            "Taylor Swift.",
            "MYTH & ROID",
            "Emerson, Lake & Palmer",
            "Sleeping With Sirens",
            "Nothing But Thieves",
            "Crosby, Stills & Nash",
            "塞壬唱片-MSR",
        ] {
            assert_eq!(split(name), [name], "must not split {name:?}");
        }
    }

    #[test]
    fn a_real_name_still_splits_off_its_features() {
        assert_eq!(
            split("Earth, Wind & Fire feat. The Emotions"),
            ["Earth, Wind & Fire", "The Emotions"]
        );
        assert_eq!(split("Jay-Z ft. Alicia Keys"), ["Jay-Z", "Alicia Keys"]);
    }

    // The shapes below are taken verbatim from a real library.
    #[test]
    fn space_padded_bullets_split() {
        assert_eq!(
            split("A$AP Rocky • Rakim Mayers"),
            ["A$AP Rocky", "Rakim Mayers"]
        );
        assert_eq!(
            split(
                "A$AP Rocky • Bones • Frans Mernick • Hector Delgado • Rakim Mayers • \
                 Elmo O'Connor"
            ),
            [
                "A$AP Rocky",
                "Bones",
                "Frans Mernick",
                "Hector Delgado",
                "Rakim Mayers",
                "Elmo O'Connor"
            ]
        );
        assert_eq!(
            split("A$AP Rocky • Joe Fox • Rakim Mayers • Brian Burton • Ben Nichols"),
            [
                "A$AP Rocky",
                "Joe Fox",
                "Rakim Mayers",
                "Brian Burton",
                "Ben Nichols"
            ]
        );
    }

    /// "<credit> • <every contributor>" keeps only the credit: the tail repeats
    /// the credit's own artists and then adds the songwriters and producers.
    #[test]
    fn a_credit_followed_by_its_contributor_list_keeps_only_the_credit() {
        assert_eq!(
            split(
                "A$AP Rocky feat. Joe Fox x Future x M.I.A. • A$AP Rocky • Joe Fox • Future • \
                 M.I.A. • Rakim Mayers • Rameses Magnus-George • Axel Morgan • Ricci Rierra • \
                 Nayvadius Wilburn"
            ),
            ["A$AP Rocky", "Joe Fox", "Future", "M.I.A."]
        );
    }

    /// Without that repetition there is no evidence of a personnel dump, so the
    /// list is taken at face value.
    #[test]
    fn a_bulleted_list_that_repeats_nothing_is_split_in_full() {
        assert_eq!(
            split("Above & Beyond • Zoë Johnston"),
            ["Above & Beyond", "Zoë Johnston"]
        );
    }

    #[test]
    fn bullets_inside_a_name_are_left_alone() {
        // Unpadded, these characters are part of the name itself.
        for name in ["マイケル・ジャクソン", "Col·lectiu", "A•B"] {
            assert_eq!(split(name), [name], "must not split {name:?}");
        }
    }

    #[test]
    fn other_bullet_shapes_split() {
        assert_eq!(
            split("Ayumi Hamasaki ・ Tetsuya Komuro"),
            ["Ayumi Hamasaki", "Tetsuya Komuro"]
        );
        assert_eq!(split("Nujabes · Shing02"), ["Nujabes", "Shing02"]);
    }

    #[test]
    fn duplicates_collapse_case_insensitively() {
        assert_eq!(split("Drake feat. drake"), ["Drake"]);
        assert_eq!(split("Drake;Drake ft. Future"), ["Drake", "Future"]);
    }

    #[test]
    fn credited_prefers_structured_values() {
        let structured = ["A$AP Rocky".to_string(), "Drake".to_string()];
        assert_eq!(
            credited("A$AP Rocky feat. Drake", &structured),
            ["A$AP Rocky", "Drake"]
        );
    }

    #[test]
    fn credited_splits_a_joined_structured_value() {
        let structured = ["A$AP Rocky feat. Drake".to_string()];
        assert_eq!(credited("whatever", &structured), ["A$AP Rocky", "Drake"]);
    }

    #[test]
    fn credited_falls_back_to_the_display_string() {
        assert_eq!(
            credited("A$AP Rocky feat. Drake", &[]),
            ["A$AP Rocky", "Drake"]
        );
        assert_eq!(credited("", &[]), Vec::<String>::new());
    }
}
