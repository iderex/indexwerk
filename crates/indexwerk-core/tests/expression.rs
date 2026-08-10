//! The suite for the index expression model of #24.
//!
//! It is an integration test rather than a unit module because everything it
//! exercises is public: the model is a shape other crates build against, and a
//! test that can only be written from inside is a test that proves nothing
//! about what a caller can reach.
//!
//! The property tests draw from a generator written here rather than from a
//! property-testing crate. This tree carries no third-party dependency and the
//! register of `docs/dependencies.md` accounts for every one that arrives, so
//! taking one is a supply chain entry rather than a convenience. What the crate
//! would buy is shrinking; what it would cost is an entry in that register and
//! a dependency in the core, which is the one crate that ships to every
//! consumer. The generator below is seeded from a constant, so a failure
//! reproduces from the test name alone, which is the half of shrinking that
//! matters for a data model this size.

use indexwerk_core::expression::{
    Factor, FactorError, IndexName, Manifold, Monomial, MonomialError, NameError, Slot, Tensor,
    TensorName, TextError, Variance,
};
use indexwerk_core::rational::{Rational, RationalError};

fn tensor(name: &str, rank: usize) -> Tensor {
    Tensor::declare(name, rank).unwrap()
}

fn factor(name: &str, slots: Vec<Slot>) -> Factor {
    Factor::new(tensor(name, slots.len()), slots).unwrap()
}

fn index(name: &str) -> IndexName {
    IndexName::new(name).unwrap()
}

// ---------------------------------------------------------------------------
// What construction refuses, one test per refusal, each reading the value the
// error carries rather than only its variant. An error that names nothing is an
// error a caller cannot act on, and the issue asks for the offending index by
// name.
// ---------------------------------------------------------------------------

#[test]
fn a_name_on_three_slots_is_refused_and_the_index_is_named() {
    let slots = vec![
        Slot::upper("a").unwrap(),
        Slot::lower("a").unwrap(),
        Slot::lower("a").unwrap(),
    ];
    let refusal = Monomial::new(
        Manifold::WithMetric,
        Rational::ONE,
        vec![factor("T", slots)],
    )
    .unwrap_err();

    assert_eq!(
        refusal,
        MonomialError::NameOnMoreThanTwoSlots {
            name: index("a"),
            slots: 3,
        }
    );
    assert!(refusal.to_string().contains("the index a is on 3 slots"));
}

#[test]
fn a_name_on_four_slots_across_two_factors_is_refused_too() {
    let refusal = Monomial::new(
        Manifold::WithMetric,
        Rational::ONE,
        vec![
            factor(
                "T",
                vec![Slot::upper("a").unwrap(), Slot::lower("a").unwrap()],
            ),
            factor(
                "S",
                vec![Slot::upper("a").unwrap(), Slot::lower("a").unwrap()],
            ),
        ],
    )
    .unwrap_err();

    assert_eq!(
        refusal,
        MonomialError::NameOnMoreThanTwoSlots {
            name: index("a"),
            slots: 4,
        }
    );
}

#[test]
fn a_contraction_in_one_variance_is_refused_where_the_manifold_has_no_metric() {
    let refusal = Monomial::new(
        Manifold::WithoutMetric,
        Rational::ONE,
        vec![factor(
            "T",
            vec![Slot::upper("a").unwrap(), Slot::upper("a").unwrap()],
        )],
    )
    .unwrap_err();

    assert_eq!(
        refusal,
        MonomialError::ContractionInOneVarianceWithoutMetric {
            name: index("a"),
            variance: Variance::Upper,
        }
    );
    assert!(refusal.to_string().contains("both halves upper"));
}

#[test]
fn the_same_contraction_is_admitted_where_the_manifold_has_a_metric() {
    // The negative half of the refusal above. Without it the check could be a
    // rule that refuses every contraction, and both tests would still pass.
    let monomial = Monomial::new(
        Manifold::WithMetric,
        Rational::ONE,
        vec![factor(
            "T",
            vec![Slot::upper("a").unwrap(), Slot::upper("a").unwrap()],
        )],
    )
    .unwrap();

    assert_eq!(monomial.dummy_pairs().len(), 1);
    assert_eq!(monomial.to_string(), "metric: 1 * T[^a,^a]");
}

#[test]
fn a_free_index_in_either_variance_is_admitted_without_a_metric() {
    // A single occurrence is a free index and the rule about matching variance
    // is about pairs, so this must pass on a manifold with no metric.
    let monomial = Monomial::new(
        Manifold::WithoutMetric,
        Rational::ONE,
        vec![factor(
            "T",
            vec![Slot::upper("a").unwrap(), Slot::upper("b").unwrap()],
        )],
    )
    .unwrap();

    assert_eq!(monomial.free_indices(), vec![index("a"), index("b")]);
}

#[test]
fn a_slot_count_that_is_not_the_declared_rank_is_refused_and_both_numbers_are_named() {
    let refusal = Factor::new(tensor("R", 4), vec![Slot::upper("a").unwrap()]).unwrap_err();

    assert_eq!(
        refusal,
        FactorError::SlotCountDoesNotMatchRank {
            tensor: TensorName::new("R").unwrap(),
            declared_rank: 4,
            slots: 1,
        }
    );
    assert!(
        refusal
            .to_string()
            .contains("declared with rank 4 and this factor gives it 1 slot")
    );
}

#[test]
fn a_monomial_with_no_factors_is_refused_by_the_constructor_that_does_not_admit_one() {
    let refusal = Monomial::new(Manifold::WithMetric, Rational::ONE, Vec::new()).unwrap_err();

    assert_eq!(refusal, MonomialError::Empty);
}

#[test]
fn the_scalar_constructor_admits_the_monomial_with_no_factors() {
    let scalar = Monomial::scalar(Manifold::WithMetric, Rational::new(3, 2).unwrap());

    assert!(scalar.factors().is_empty());
    assert_eq!(scalar.to_string(), "metric: 3/2");
    assert_eq!("metric: 3/2".parse::<Monomial>().unwrap(), scalar);
}

#[test]
fn a_name_the_text_form_could_not_write_is_refused_where_it_is_built() {
    assert_eq!(IndexName::new(""), Err(NameError::Empty));
    assert_eq!(
        IndexName::new("1a"),
        Err(NameError::FirstCharacter {
            name: "1a".to_owned(),
            character: '1',
        })
    );
    assert_eq!(
        TensorName::new("R-1"),
        Err(NameError::Character {
            name: "R-1".to_owned(),
            character: '-',
        })
    );
    // The characters the text form itself uses are the ones worth checking,
    // because a name carrying one of them would parse back as two things.
    // The underscore is deliberately absent: it is legal inside a name, and it
    // is unambiguous because a variance marker is the first character of a slot
    // and nowhere else.
    for character in ['^', '[', ']', ',', '*', ':', ' '] {
        let name = format!("a{character}b");
        assert!(
            IndexName::new(&name).is_err(),
            "{name:?} was accepted and the text form cannot write it"
        );
    }
    // An underscore inside a name is legal; only a leading one is not.
    assert!(IndexName::new("a_1").is_ok());
}

// ---------------------------------------------------------------------------
// Free indices and dummy pairs are different kinds of thing, not one kind with
// a flag.
// ---------------------------------------------------------------------------

#[test]
fn a_name_on_one_slot_is_free_and_a_name_on_two_is_a_pair() {
    let monomial: Monomial = "no-metric: 1 * T[^a,_b] * S[^b,_c]".parse().unwrap();

    assert_eq!(monomial.free_indices(), vec![index("a"), index("c")]);
    let pairs = monomial.dummy_pairs();
    assert_eq!(pairs.len(), 1);
    assert_eq!(pairs[0].name, index("b"));
    assert_eq!(pairs[0].first, Variance::Lower);
    assert_eq!(pairs[0].second, Variance::Upper);
}

#[test]
fn the_variance_of_each_half_of_a_pair_is_kept_in_the_order_the_halves_appear() {
    let monomial: Monomial = "metric: 1 * R[^a,^b,_a,_b]".parse().unwrap();
    let pairs = monomial.dummy_pairs();

    assert_eq!(pairs.len(), 2);
    assert_eq!(pairs[0].name, index("a"));
    assert_eq!(
        (pairs[0].first, pairs[0].second),
        (Variance::Upper, Variance::Lower)
    );
    assert_eq!(pairs[1].name, index("b"));
    assert!(monomial.free_indices().is_empty());
}

// ---------------------------------------------------------------------------
// The text form.
// ---------------------------------------------------------------------------

#[test]
fn a_rank_zero_factor_writes_and_reads_back_as_empty_brackets() {
    let monomial = Monomial::new(
        Manifold::WithMetric,
        Rational::ONE,
        vec![factor("phi", Vec::new())],
    )
    .unwrap();

    assert_eq!(monomial.to_string(), "metric: 1 * phi[]");
    assert_eq!("metric: 1 * phi[]".parse::<Monomial>().unwrap(), monomial);
}

#[test]
fn text_that_is_not_a_monomial_is_refused_and_says_which_part_is_wrong() {
    assert!(matches!(
        "1 * T[^a]".parse::<Monomial>(),
        Err(TextError::NoManifold { .. })
    ));
    assert!(matches!(
        "affine: 1 * T[^a]".parse::<Monomial>(),
        Err(TextError::UnknownManifold { .. })
    ));
    assert!(matches!(
        "metric: one * T[^a]".parse::<Monomial>(),
        Err(TextError::Coefficient(_))
    ));
    assert!(matches!(
        "metric: 1 * T^a".parse::<Monomial>(),
        Err(TextError::MalformedFactor { .. })
    ));
    assert!(matches!(
        "metric: 1 * T[a]".parse::<Monomial>(),
        Err(TextError::MalformedSlot { .. })
    ));
    assert!(matches!(
        "metric: 1 * T[^a,^a,^a]".parse::<Monomial>(),
        Err(TextError::Monomial(
            MonomialError::NameOnMoreThanTwoSlots { .. }
        ))
    ));
    assert!(matches!(
        "no-metric: 1 * T[^a,^a]".parse::<Monomial>(),
        Err(TextError::Monomial(
            MonomialError::ContractionInOneVarianceWithoutMetric { .. }
        ))
    ));
}

#[test]
fn the_coefficient_is_reduced_and_carries_the_sign() {
    assert_eq!(Rational::new(-6, 4).unwrap(), Rational::new(3, -2).unwrap());
    assert_eq!(Rational::new(-6, 4).unwrap().to_string(), "-3/2");
    assert_eq!(Rational::new(0, -7).unwrap(), Rational::ZERO);
    assert_eq!(Rational::new(1, 0), Err(RationalError::ZeroDenominator));
    assert_eq!(Rational::integer(4).to_string(), "4");
    assert!(Rational::ZERO.is_zero());
}

// ---------------------------------------------------------------------------
// The round trip, as a property over random monomials.
// ---------------------------------------------------------------------------

/// A deterministic generator, xorshift with a multiply, seeded from a constant
/// written at the call site. Reproducibility is the whole point: a failure here
/// is reproduced by running the test again, with no corpus file and no seed to
/// recover from a log.
struct Rng(u64);

impl Rng {
    fn next_value(&mut self) -> u64 {
        let mut state = self.0;
        state ^= state >> 12;
        state ^= state << 25;
        state ^= state >> 27;
        self.0 = state;
        state.wrapping_mul(0x2545_f491_4f6c_dd1d)
    }

    /// A value in `0..bound`. `bound` is small here, so the modulo bias is
    /// smaller than the number of draws would reveal and it does not matter for
    /// a generator whose job is coverage rather than uniformity.
    fn below(&mut self, bound: usize) -> usize {
        if bound == 0 {
            return 0;
        }
        usize::try_from(self.next_value() % bound as u64).unwrap_or(0)
    }

    fn variance(&mut self) -> Variance {
        if self.below(2) == 0 {
            Variance::Upper
        } else {
            Variance::Lower
        }
    }
}

const INDEX_POOL: &[&str] = &["a", "b", "c", "d", "e", "f", "g", "h"];

fn random_monomial(rng: &mut Rng) -> Monomial {
    let manifold = if rng.below(2) == 0 {
        Manifold::WithMetric
    } else {
        Manifold::WithoutMetric
    };
    let pairs = rng.below(4);
    let frees = rng.below(1 + INDEX_POOL.len() - 2 * pairs);
    let mut names = INDEX_POOL.iter();

    let mut slots: Vec<Slot> = Vec::new();
    for _ in 0..pairs {
        let name = names.next().copied().unwrap_or("z");
        let first = rng.variance();
        let second = match manifold {
            Manifold::WithMetric => rng.variance(),
            Manifold::WithoutMetric => match first {
                Variance::Upper => Variance::Lower,
                Variance::Lower => Variance::Upper,
            },
        };
        slots.push(Slot::new(index(name), first));
        slots.push(Slot::new(index(name), second));
    }
    for _ in 0..frees {
        let name = names.next().copied().unwrap_or("z");
        let variance = rng.variance();
        slots.push(Slot::new(index(name), variance));
    }

    // Fisher-Yates, so a pair's two halves land in different factors as often
    // as in one. A generator that always kept them adjacent would never reach
    // the case the model is about.
    for position in (1..slots.len()).rev() {
        slots.swap(position, rng.below(position + 1));
    }

    let coefficient = Rational::new(
        i64::try_from(rng.below(41)).unwrap_or(1) - 20,
        i64::try_from(rng.below(6)).unwrap_or(0) + 1,
    )
    .unwrap();

    if slots.is_empty() {
        return Monomial::scalar(manifold, coefficient);
    }

    let factor_count = 1 + rng.below(3);
    let mut runs: Vec<Vec<Slot>> = vec![Vec::new(); factor_count];
    for (position, slot) in slots.into_iter().enumerate() {
        runs[position % factor_count].push(slot);
    }
    let factors = runs
        .into_iter()
        .enumerate()
        .map(|(position, run)| factor(&format!("T{position}"), run))
        .collect();

    Monomial::new(manifold, coefficient, factors).unwrap()
}

#[test]
fn every_random_monomial_survives_its_text_form_unchanged() {
    let mut rng = Rng(0x1234_5678_9abc_def0);
    let mut seen_pair = 0;
    let mut seen_free = 0;
    let mut seen_scalar = 0;
    let mut seen_fraction = 0;
    let mut seen_without_metric = 0;
    let mut seen_split_pair = 0;

    for case in 0..2000 {
        let monomial = random_monomial(&mut rng);

        let text = monomial.to_string();
        let read_back: Monomial = text
            .parse()
            .unwrap_or_else(|error| panic!("case {case}: {text:?} did not read back: {error}"));
        assert_eq!(read_back, monomial, "case {case}: {text:?}");
        assert_eq!(read_back.to_string(), text, "case {case}");

        if !monomial.dummy_pairs().is_empty() {
            seen_pair += 1;
        }
        if !monomial.free_indices().is_empty() {
            seen_free += 1;
        }
        if monomial.factors().is_empty() {
            seen_scalar += 1;
        }
        if monomial.coefficient().denominator() != 1 {
            seen_fraction += 1;
        }
        if monomial.manifold() == Manifold::WithoutMetric {
            seen_without_metric += 1;
        }
        for pair in monomial.dummy_pairs() {
            let carrying = monomial
                .factors()
                .iter()
                .filter(|factor| factor.slots().iter().any(|slot| *slot.name() == pair.name))
                .count();
            if carrying == 2 {
                seen_split_pair += 1;
            }
        }
    }

    // A property test is worth exactly what its generator reaches, so the run
    // asserts what it reached. Without these a generator that degenerated to
    // one shape would go on passing and prove nothing.
    assert!(seen_pair > 100, "pairs reached: {seen_pair}");
    assert!(seen_free > 100, "free indices reached: {seen_free}");
    assert!(
        seen_scalar > 10,
        "factorless monomials reached: {seen_scalar}"
    );
    assert!(
        seen_fraction > 100,
        "fractional coefficients: {seen_fraction}"
    );
    assert!(
        seen_without_metric > 100,
        "manifolds without a metric: {seen_without_metric}"
    );
    assert!(
        seen_split_pair > 100,
        "pairs split across two factors: {seen_split_pair}"
    );
}

#[test]
fn every_random_monomial_agrees_with_itself_about_which_names_are_free() {
    // The round trip compares values. This compares what the value says about
    // itself, so a model that lost the free-versus-dummy distinction while
    // still rendering identically would be caught.
    let mut rng = Rng(0x0fed_cba9_8765_4321);

    for case in 0..2000 {
        let monomial = random_monomial(&mut rng);
        let slots: Vec<&IndexName> = monomial.slots().map(Slot::name).collect();

        for name in monomial.free_indices() {
            assert_eq!(
                slots.iter().filter(|held| ***held == name).count(),
                1,
                "case {case}: {name} is reported free"
            );
        }
        for pair in monomial.dummy_pairs() {
            assert_eq!(
                slots.iter().filter(|held| ***held == pair.name).count(),
                2,
                "case {case}: {} is reported as a pair",
                pair.name
            );
        }
        assert_eq!(
            monomial.free_indices().len() + 2 * monomial.dummy_pairs().len(),
            slots.len(),
            "case {case}: every slot is accounted for exactly once"
        );
    }
}
