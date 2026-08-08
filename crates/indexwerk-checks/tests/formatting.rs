//! The format leg for the parts of the tree `cargo fmt` does not read (#16).
//!
//! This is what the `format` check runs beside `cargo fmt --all -- --check`.
//! The Rust half is the formatter's; the Markdown half and the question of a
//! language that has not arrived yet are here.

use indexwerk_checks::formatting::{Defect, check_text, check_tree, python_sources};

#[test]
fn this_tree_is_clean() {
    let complaints = check_tree();
    assert!(
        complaints.is_empty(),
        "the Markdown in this tree has whitespace defects:\n{}",
        complaints
            .iter()
            .map(ToString::to_string)
            .collect::<Vec<_>>()
            .join("\n")
    );
}

#[test]
fn the_walk_actually_reached_the_markdown() {
    // A check that reads nothing passes, and passes for the wrong reason. The
    // walk is not observable from its findings when there are none, so this
    // feeds it the shapes it has to find instead, and the tree test above is
    // what says those shapes are absent from the tree.
    assert!(!check_text("x.md", "a line\twith a tab\n").is_empty());
}

#[test]
fn a_tab_is_a_defect() {
    let found = check_text("x.md", "no tab here\n\tone here\n");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].defect, Defect::Tab);
    assert_eq!(found[0].line, 2);
}

#[test]
fn trailing_whitespace_is_a_defect_and_a_hard_line_break_is_not() {
    let break_line = check_text("x.md", "a line ending in a hard break  \nand the next\n");
    assert!(
        break_line.is_empty(),
        "two spaces are Markdown's hard line break and stay: {break_line:?}"
    );

    for trailing in [" ", "   ", "    ", " \t"] {
        let found = check_text("x.md", &format!("a line{trailing}\n"));
        assert!(
            found
                .iter()
                .any(|complaint| complaint.defect == Defect::TrailingWhitespace),
            "{trailing:?} is not a hard break and has to be refused: {found:?}"
        );
    }
}

#[test]
fn a_line_that_looks_empty_and_is_not_is_a_defect() {
    let found = check_text("x.md", "a line\n   \nanother\n");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].defect, Defect::WhitespaceOnlyLine);
    assert_eq!(found[0].line, 2);

    assert!(
        check_text("x.md", "a line\n\nanother\n").is_empty(),
        "an actually empty line is how paragraphs are separated"
    );
}

#[test]
fn a_missing_final_newline_is_a_defect() {
    let found = check_text("x.md", "one line\ntwo lines");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].defect, Defect::NoFinalNewline);
}

#[test]
fn a_blank_line_at_the_end_is_a_defect() {
    let found = check_text("x.md", "one line\n\n");
    assert_eq!(found.len(), 1, "{found:?}");
    assert_eq!(found[0].defect, Defect::BlankLineAtEnd);

    assert!(
        check_text("x.md", "one line\n").is_empty(),
        "exactly one final newline is what is asked for"
    );
}

/// The rule this check exists to not have. A working tree can carry carriage
/// returns while the committed bytes are LF, because `.gitattributes` decides
/// what is stored and a checkout predating a rule in it is not rewritten. A
/// formatter judging line endings would report that tree as failing, which is
/// the failure #16 names.
#[test]
fn a_carriage_return_is_not_judged_here() {
    assert!(
        check_text("x.md", "one line\r\ntwo lines\r\n").is_empty(),
        "line endings belong to .gitattributes, and judging them here would red a clean tree"
    );
    assert!(
        !check_text("x.md", "one line \r\n").is_empty(),
        "the carriage return is removed before judging, so what is under it is still read"
    );
}

/// An empty file has no lines to judge and no missing newline to complain
/// about, and complaining about it would be a rule nobody asked for.
#[test]
fn an_empty_file_is_left_alone() {
    assert!(check_text("x.md", "").is_empty());
}

/// The format leg has to say something about the Python sources. There are
/// none, and no Python formatter is configured anywhere in this tree, and those
/// two facts are only consistent together. This is the leg failing closed in
/// the direction that matters: the day a `.py` file lands without a formatter
/// chosen for it, this reds and names the file.
#[test]
fn no_python_source_has_arrived_without_a_formatter_being_chosen() {
    let sources = python_sources();
    assert!(
        sources.is_empty(),
        "Python sources are in this tree and no Python formatter is configured for them.\n\
         Choose one and add it to the format leg of .github/workflows/lint.yml, and to the \
         commands in CONTRIBUTING.md, in the change that brings the first of these files:\n  {}",
        sources.join("\n  ")
    );
}
