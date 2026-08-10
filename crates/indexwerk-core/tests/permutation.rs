//! The permutation engine of #20, tested as properties and against the order
//! written in `docs/normal-form.md`.
//!
//! # Where the random inputs come from
//!
//! From the generator at the bottom of this file rather than from a
//! property-testing crate. The means was chosen rather than carried over: this
//! workspace has no third-party dependency at all, every entry added to one is
//! a supply chain entry the dependency policy of #38 has to account for and a
//! reason somebody has to write into `docs/dependencies.md`, and what is needed
//! here is a stream of small permutations rather than a shrinking engine. The
//! cost of the choice is that a failure is not minimised for you.
//!
//! It is paid down by determinism. The generator is seeded from a constant, so
//! a failing case is reproduced by running the test again, and the case that
//! failed is the same one on every machine and at every revision.

use std::path::{Path, PathBuf};

use indexwerk_core::permutation::{Permutation, PermutationError, Sign, Signed};

/// The largest width the property tests generate.
///
/// Eight points is above every rank this library will meet in a single tensor
/// factor, and the properties below are about the group rather than about
/// scale: an associativity failure that needs nine points to appear would be a
/// failure of arithmetic on the image array, which every width already
/// exercises. Wider cases belong to the benchmark legs, not here.
const MAX_WIDTH: usize = 8;

/// How many cases each property draws.
///
/// Four hundred is chosen so the suite stays fast enough to run on every
/// change. It is not a coverage claim: with a fixed seed this is a fixed set of
/// four hundred cases rather than a sample of the whole group, and widening the
/// set means raising this number and reading the new failures.
const CASES: usize = 400;

#[test]
fn composition_is_associative() {
    let mut source = Source::new();
    for _ in 0..CASES {
        let width = source.width();
        let first = source.permutation(width);
        let second = source.permutation(width);
        let third = source.permutation(width);

        let left = first.then(&second).unwrap().then(&third).unwrap();
        let right = first.then(&second.then(&third).unwrap()).unwrap();

        assert_eq!(left, right, "associativity failed on width {width}");
    }
}

#[test]
fn inversion_is_a_two_sided_inverse() {
    let mut source = Source::new();
    for _ in 0..CASES {
        let width = source.width();
        let permutation = source.permutation(width);
        let inverse = permutation.inverse();
        let identity = Permutation::identity(width);

        assert_eq!(permutation.then(&inverse).unwrap(), identity);
        assert_eq!(inverse.then(&permutation).unwrap(), identity);
    }
}

#[test]
fn the_identity_is_neutral() {
    let mut source = Source::new();
    for _ in 0..CASES {
        let width = source.width();
        let permutation = source.permutation(width);
        let identity = Permutation::identity(width);

        assert_eq!(permutation.then(&identity).unwrap(), permutation);
        assert_eq!(identity.then(&permutation).unwrap(), permutation);
    }
}

#[test]
fn the_order_is_total_and_antisymmetric() {
    let mut source = Source::new();
    for _ in 0..CASES {
        let left = source.signed();
        let right = source.signed();

        // Total: exactly one of the three relations holds, and it holds in the
        // direction the reverse comparison agrees with.
        let less = left < right;
        let equal = left == right;
        let greater = left > right;
        assert_eq!(
            usize::from(less) + usize::from(equal) + usize::from(greater),
            1,
            "{left:?} against {right:?} satisfied more or fewer than one relation"
        );

        // Antisymmetric: comparable in both directions only where the two are
        // the same value.
        if left <= right && right <= left {
            assert_eq!(left, right);
        }
    }
}

#[test]
fn the_order_is_transitive() {
    let mut source = Source::new();
    for _ in 0..CASES {
        let mut drawn = [source.signed(), source.signed(), source.signed()];
        drawn.sort();
        let [first, second, third] = drawn;

        assert!(first <= second);
        assert!(second <= third);
        assert!(
            first <= third,
            "{first:?} <= {second:?} <= {third:?} but not the first against the third"
        );
    }
}

#[test]
fn composing_signed_permutations_multiplies_their_signs() {
    let mut source = Source::new();
    for _ in 0..CASES {
        let width = source.width();
        let left = Signed::new(source.permutation(width), source.sign());
        let right = Signed::new(source.permutation(width), source.sign());

        let expected = match (left.sign(), right.sign()) {
            (Sign::Zero, _) | (_, Sign::Zero) => Sign::Zero,
            (one, other) if one == other => Sign::Plus,
            _ => Sign::Minus,
        };

        assert_eq!(left.then(&right).unwrap().sign(), expected);
        // The permutation half is unaffected by the sign it travels with.
        assert_eq!(
            left.then(&right).unwrap().permutation(),
            &left.permutation().then(right.permutation()).unwrap()
        );
    }
}

#[test]
fn a_zero_sign_is_representable_and_propagates_through_composition() {
    // The case this exists for: an antisymmetric pair carrying the same dummy
    // on both slots. The symmetry sends the monomial to itself with the sign
    // reversed, so the monomial is zero, and the engine has to be able to say
    // so rather than return a plausible term.
    let vanished = Signed::new(Permutation::identity(4), Sign::Zero);

    let mut source = Source::new();
    for _ in 0..CASES {
        let other = Signed::new(source.permutation(4), source.sign());

        assert_eq!(vanished.then(&other).unwrap().sign(), Sign::Zero);
        assert_eq!(other.then(&vanished).unwrap().sign(), Sign::Zero);
    }

    // It composes with itself and stays zero, and it has no inverse, because a
    // vanished expression is not a group element to compose back out of.
    assert_eq!(vanished.then(&vanished).unwrap().sign(), Sign::Zero);
    assert_eq!(vanished.inverse(), None);
    assert!(Signed::identity(4).inverse().is_some());
}

#[test]
fn an_image_array_that_is_not_a_bijection_is_refused() {
    assert_eq!(
        Permutation::new(vec![0, 3, 1]),
        Err(PermutationError::ImageOutOfRange {
            position: 1,
            image: 3,
            width: 3
        })
    );
    assert_eq!(
        Permutation::new(vec![0, 1, 1]),
        Err(PermutationError::RepeatedImage {
            image: 1,
            first: 1,
            second: 2
        })
    );
    assert!(Permutation::new(Vec::new()).is_ok());
}

#[test]
fn permutations_of_different_widths_are_refused_rather_than_padded() {
    let narrow = Permutation::identity(2);
    let wide = Permutation::identity(3);

    assert_eq!(
        narrow.then(&wide),
        Err(PermutationError::WidthMismatch { left: 2, right: 3 })
    );
    assert_eq!(
        wide.then(&narrow),
        Err(PermutationError::WidthMismatch { left: 3, right: 2 })
    );
}

#[test]
fn the_order_is_the_one_the_normal_form_document_fixes() {
    let entries = fixture();

    // The fixture is a list in ascending order, so consecutive entries are
    // strictly ascending. A file that lost half its lines would still satisfy
    // that, which is what the coverage assertions below are for.
    for pair in entries.windows(2) {
        assert!(
            pair[0] < pair[1],
            "{:?} is not below {:?} in the fixture order",
            pair[0],
            pair[1]
        );
    }

    // Sorting a deliberately disordered copy has to reproduce the file exactly.
    // This is the half that catches an order which is merely consistent with
    // the file rather than equal to it.
    let mut shuffled = entries.clone();
    let mut source = Source::new();
    for index in (1..shuffled.len()).rev() {
        let other = source.below(index + 1);
        shuffled.swap(index, other);
    }
    assert_ne!(shuffled, entries, "the shuffle left the list in place");
    shuffled.sort();
    assert_eq!(shuffled, entries);

    // What the fixture has to reach for the assertions above to mean anything.
    for width in 0..=3 {
        assert!(
            entries.iter().any(|entry| entry.width() == width),
            "the fixture carries no permutation of width {width}"
        );
    }
    for sign in [Sign::Plus, Sign::Minus, Sign::Zero] {
        assert!(
            entries.iter().any(|entry| entry.sign() == sign),
            "the fixture carries no {sign} sign"
        );
    }
}

#[test]
fn the_document_quotes_the_fixture_it_names() {
    // "the same fixture the document quotes" is the clause this is for. A
    // quotation that has drifted from the file is worse than no quotation,
    // because a reader takes the document as the authority and never opens the
    // file.
    let document = std::fs::read_to_string(workspace_root().join("docs/normal-form.md"))
        .expect("docs/normal-form.md");
    let marker = "The file opens:";
    let offset = document
        .find(marker)
        .expect("docs/normal-form.md introduces its quotation of the fixture");

    let quoted: Vec<&str> = document[offset + marker.len()..]
        .lines()
        .skip_while(|line| line.trim().is_empty())
        .take_while(|line| line.starts_with("    "))
        .map(|line| &line[4..])
        .collect();

    assert!(!quoted.is_empty(), "the document quotes no lines");

    let written = fixture_lines();
    assert!(quoted.len() <= written.len());
    assert_eq!(quoted, written[..quoted.len()]);
}

/// The workspace root, from the crate this test belongs to.
fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("the core crate sits two directories below the workspace root")
        .to_path_buf()
}

/// The lines of the fixture that carry data, comments and blanks dropped.
fn fixture_lines() -> Vec<String> {
    let path = workspace_root().join("conformance/order/permutations.txt");
    let text = std::fs::read_to_string(&path).expect("conformance/order/permutations.txt");
    text.lines()
        .map(str::trim_end)
        .filter(|line| !line.is_empty() && !line.starts_with('#'))
        .map(str::to_owned)
        .collect()
}

/// The fixture, parsed.
fn fixture() -> Vec<Signed> {
    fixture_lines().iter().map(|line| parse(line)).collect()
}

/// One fixture line: the sign, a space, the width, a colon, and the image array
/// as decimal points separated by commas.
fn parse(line: &str) -> Signed {
    let (sign, rest) = line
        .split_once(' ')
        .unwrap_or_else(|| panic!("{line:?} has no sign"));
    let sign = match sign {
        "+" => Sign::Plus,
        "-" => Sign::Minus,
        "0" => Sign::Zero,
        other => panic!("{other:?} is not a sign"),
    };

    let (width, images) = rest
        .split_once(':')
        .unwrap_or_else(|| panic!("{line:?} has no width"));
    let width: usize = width
        .parse()
        .unwrap_or_else(|_| panic!("{width:?} is not a width"));
    let images: Vec<usize> = if images.is_empty() {
        Vec::new()
    } else {
        images
            .split(',')
            .map(|point| {
                point
                    .parse()
                    .unwrap_or_else(|_| panic!("{point:?} is not a point"))
            })
            .collect()
    };
    assert_eq!(
        images.len(),
        width,
        "{line:?} declares a width its image array does not have"
    );

    Signed::new(
        Permutation::new(images).unwrap_or_else(|error| panic!("{line:?}: {error}")),
        sign,
    )
}

/// A deterministic source of small permutations and signs.
///
/// Xorshift with a multiply, seeded from a constant. The stream is fixed, so a
/// failure reported by any test above is reproduced by running that test again
/// and is the same case on every machine.
struct Source {
    state: u64,
}

impl Source {
    /// The first sixteen hexadecimal digits of the fractional part of pi. Any
    /// non-zero constant would do; what matters is that it is written here
    /// rather than read from a clock.
    const SEED: u64 = 0x243F_6A88_85A3_08D3;

    fn new() -> Self {
        Source { state: Self::SEED }
    }

    fn next(&mut self) -> u64 {
        let mut state = self.state;
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        self.state = state;
        state.wrapping_mul(0x2545_F491_4F6C_DD1D)
    }

    /// A value below `bound`, and zero where the bound is zero.
    fn below(&mut self, bound: usize) -> usize {
        if bound == 0 {
            return 0;
        }
        (self.next() % bound as u64) as usize
    }

    /// A width in `0..=MAX_WIDTH`. Zero and one are included deliberately: an
    /// empty permutation and a one-point one are where an off-by-one in the
    /// image array shows up.
    fn width(&mut self) -> usize {
        self.below(MAX_WIDTH + 1)
    }

    fn permutation(&mut self, width: usize) -> Permutation {
        let mut images: Vec<usize> = (0..width).collect();
        for index in (1..width).rev() {
            let other = self.below(index + 1);
            images.swap(index, other);
        }
        Permutation::new(images).expect("a shuffle of 0..width is a permutation of it")
    }

    fn sign(&mut self) -> Sign {
        match self.below(3) {
            0 => Sign::Plus,
            1 => Sign::Minus,
            _ => Sign::Zero,
        }
    }

    /// A signed permutation of an arbitrary width, so that the width component
    /// of the order is exercised as well as the image array.
    fn signed(&mut self) -> Signed {
        let width = self.width();
        Signed::new(self.permutation(width), self.sign())
    }
}
