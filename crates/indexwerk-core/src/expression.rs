//! The index expression model: slots, free indices, dummy pairs, variance.
//!
//! The data the canonicaliser works on, and the point at which a monomial that
//! cannot mean anything is refused. Issue #24 is where the shape was argued.
//!
//! A monomial is a product of tensor factors with a rational coefficient. Each
//! factor names a tensor and occupies a run of index slots, and each slot holds
//! an index name and a variance, upper or lower. A name occurring once in the
//! monomial is a free index; a name occurring twice is a dummy pair.
//!
//! Three distinctions this model keeps, and one it refuses to keep:
//!
//! - A free index names something outside the expression and may not be
//!   renamed. A dummy pair may be renamed to any unused name. The two are
//!   different kinds of thing here rather than the same kind with a flag,
//!   because collapsing them is the classic route to a canonicaliser that
//!   returns a wrong answer confidently.
//! - Variance is part of the identity of a slot rather than a decoration on it.
//! - Whether the manifold has a metric is carried on the monomial, because
//!   without one an upper index and a lower index are different slots that no
//!   convention merges, and with one they are raised and lowered freely.
//! - A name on three or more slots is refused. It is not a case to be handled
//!   later; it is an expression with no meaning, and the refusal names the
//!   index.
//!
//! # The text form
//!
//! Every value here renders to one line and reads back from it, so a monomial
//! can be written into a fixture file, diffed and reviewed. The grammar, in
//! full:
//!
//! ```text
//! monomial    = manifold ": " coefficient *( " * " factor )
//! manifold    = "metric" / "no-metric"
//! coefficient = integer [ "/" integer ]
//! factor      = name "[" [ slot *( "," slot ) ] "]"
//! slot        = ( "^" / "_" ) name
//! name        = ALPHA *( ALPHA / DIGIT / "_" )
//! ```
//!
//! The manifold and the coefficient are always written, including when the
//! coefficient is one. A form that omits what it can infer has two spellings
//! for one value, and a fixture file whose entries are compared as text cannot
//! afford that.
//!
//! This is not the normal form. It fixes how a monomial is written down, not
//! which of several equal monomials is the representative, and it imposes no
//! order on anything. The orderings are `docs/normal-form.md`, which is a
//! placeholder at the time of writing, and nothing here may be read as fixing
//! one of them.

use crate::rational::{Rational, RationalParseError};
use core::fmt;
use core::str::FromStr;

/// Whether a slot is an upper or a lower index.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Variance {
    Upper,
    Lower,
}

impl Variance {
    /// The character the text form writes in front of the index name.
    pub const fn marker(self) -> char {
        match self {
            Variance::Upper => '^',
            Variance::Lower => '_',
        }
    }

    /// The word a refusal uses, so that an error message says which half of a
    /// pair it is talking about.
    pub const fn word(self) -> &'static str {
        match self {
            Variance::Upper => "upper",
            Variance::Lower => "lower",
        }
    }
}

impl fmt::Display for Variance {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.word())
    }
}

/// Whether the manifold this monomial lives on carries a metric.
///
/// Set on the expression rather than assumed globally: a library used for one
/// computation with a metric and one without, in one process, must not have to
/// agree with itself about which it is doing.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub enum Manifold {
    /// Indices are raised and lowered freely, so a dummy pair may sit in one
    /// variance and the canonical form fixes a convention for it.
    WithMetric,
    /// Upper and lower are different slots and no convention merges them, so a
    /// contraction pairs one of each.
    WithoutMetric,
}

impl Manifold {
    /// The token the text form writes.
    pub const fn token(self) -> &'static str {
        match self {
            Manifold::WithMetric => "metric",
            Manifold::WithoutMetric => "no-metric",
        }
    }
}

impl fmt::Display for Manifold {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(self.token())
    }
}

/// Why a name was refused.
///
/// Names are constrained so that the text form is unambiguous without quoting.
/// A model that accepts a name it cannot write down is a model with a value
/// that does not survive a round trip.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum NameError {
    /// The empty string.
    Empty,
    /// The first character is not an ASCII letter.
    FirstCharacter { name: String, character: char },
    /// A later character is not an ASCII letter, digit or underscore.
    Character { name: String, character: char },
}

impl fmt::Display for NameError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            NameError::Empty => write!(f, "a name may not be empty"),
            NameError::FirstCharacter { name, character } => write!(
                f,
                "the name {name:?} starts with {character:?}, which is not an ASCII letter"
            ),
            NameError::Character { name, character } => write!(
                f,
                "the name {name:?} contains {character:?}, which is not an ASCII letter, digit or \
                 underscore"
            ),
        }
    }
}

impl core::error::Error for NameError {}

fn validate_name(text: &str) -> Result<(), NameError> {
    let mut characters = text.chars();
    let first = match characters.next() {
        Some(character) => character,
        None => return Err(NameError::Empty),
    };
    if !first.is_ascii_alphabetic() {
        return Err(NameError::FirstCharacter {
            name: text.to_owned(),
            character: first,
        });
    }
    for character in characters {
        if !(character.is_ascii_alphanumeric() || character == '_') {
            return Err(NameError::Character {
                name: text.to_owned(),
                character,
            });
        }
    }
    Ok(())
}

/// The name of an index.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct IndexName(String);

impl IndexName {
    /// # Errors
    ///
    /// [`NameError`] when the text is not a name the text form can write.
    pub fn new(text: &str) -> Result<Self, NameError> {
        validate_name(text)?;
        Ok(IndexName(text.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for IndexName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// The name of a tensor.
#[derive(Clone, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct TensorName(String);

impl TensorName {
    /// # Errors
    ///
    /// [`NameError`] when the text is not a name the text form can write.
    pub fn new(text: &str) -> Result<Self, NameError> {
        validate_name(text)?;
        Ok(TensorName(text.to_owned()))
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Display for TensorName {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// One index slot: a name and a variance.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Slot {
    name: IndexName,
    variance: Variance,
}

impl Slot {
    pub const fn new(name: IndexName, variance: Variance) -> Self {
        Slot { name, variance }
    }

    /// An upper slot, from a name that is validated here.
    ///
    /// # Errors
    ///
    /// [`NameError`] when the text is not a name.
    pub fn upper(name: &str) -> Result<Self, NameError> {
        Ok(Slot::new(IndexName::new(name)?, Variance::Upper))
    }

    /// A lower slot, from a name that is validated here.
    ///
    /// # Errors
    ///
    /// [`NameError`] when the text is not a name.
    pub fn lower(name: &str) -> Result<Self, NameError> {
        Ok(Slot::new(IndexName::new(name)?, Variance::Lower))
    }

    pub const fn name(&self) -> &IndexName {
        &self.name
    }

    pub const fn variance(&self) -> Variance {
        self.variance
    }
}

impl fmt::Display for Slot {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}{}", self.variance.marker(), self.name)
    }
}

/// A tensor, declared by name and rank.
///
/// The rank is declared here and checked against the slots a factor supplies,
/// which is the only reason the declaration is a separate value rather than a
/// name on the factor. Slot symmetries are #25 and are deliberately not part of
/// this type yet.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Tensor {
    name: TensorName,
    rank: usize,
}

impl Tensor {
    pub const fn new(name: TensorName, rank: usize) -> Self {
        Tensor { name, rank }
    }

    /// Declare a tensor from a name that is validated here.
    ///
    /// # Errors
    ///
    /// [`NameError`] when the text is not a name.
    pub fn declare(name: &str, rank: usize) -> Result<Self, NameError> {
        Ok(Tensor::new(TensorName::new(name)?, rank))
    }

    pub const fn name(&self) -> &TensorName {
        &self.name
    }

    pub const fn rank(&self) -> usize {
        self.rank
    }
}

/// Why a factor was refused.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum FactorError {
    /// The factor supplied a number of slots the tensor's declared rank does
    /// not admit. The tensor is named, and so are both numbers, because the
    /// caller reading this has to know which of the two is wrong.
    SlotCountDoesNotMatchRank {
        tensor: TensorName,
        declared_rank: usize,
        slots: usize,
    },
}

impl fmt::Display for FactorError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            FactorError::SlotCountDoesNotMatchRank {
                tensor,
                declared_rank,
                slots,
            } => write!(
                f,
                "the tensor {tensor} is declared with rank {declared_rank} and this factor gives \
                 it {slots} slot(s)"
            ),
        }
    }
}

impl core::error::Error for FactorError {}

/// One tensor factor: a tensor and the run of slots it occupies.
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Factor {
    tensor: Tensor,
    slots: Vec<Slot>,
}

impl Factor {
    /// # Errors
    ///
    /// [`FactorError::SlotCountDoesNotMatchRank`] when the slot count is not
    /// the declared rank.
    pub fn new(tensor: Tensor, slots: Vec<Slot>) -> Result<Self, FactorError> {
        if slots.len() != tensor.rank() {
            return Err(FactorError::SlotCountDoesNotMatchRank {
                tensor: tensor.name().clone(),
                declared_rank: tensor.rank(),
                slots: slots.len(),
            });
        }
        Ok(Factor { tensor, slots })
    }

    pub const fn tensor(&self) -> &Tensor {
        &self.tensor
    }

    pub fn slots(&self) -> &[Slot] {
        &self.slots
    }
}

impl fmt::Display for Factor {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}[", self.tensor.name())?;
        for (position, slot) in self.slots.iter().enumerate() {
            if position > 0 {
                f.write_str(",")?;
            }
            write!(f, "{slot}")?;
        }
        f.write_str("]")
    }
}

/// A name occurring twice, with the variance of each half, in the order the
/// halves appear.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct DummyPair {
    pub name: IndexName,
    pub first: Variance,
    pub second: Variance,
}

/// Why a monomial was refused.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum MonomialError {
    /// No factors, where a factorless monomial is not what the caller asked
    /// for. [`Monomial::scalar`] is the constructor that admits one.
    Empty,
    /// A name on three or more slots. Two slots are a contraction; anything
    /// more is an expression with no meaning, and the index is named.
    NameOnMoreThanTwoSlots { name: IndexName, slots: usize },
    /// A dummy pair in one variance on a manifold with no metric, where
    /// nothing can raise or lower either half. The index and the variance both
    /// halves sit in are named.
    ContractionInOneVarianceWithoutMetric { name: IndexName, variance: Variance },
}

impl fmt::Display for MonomialError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            MonomialError::Empty => write!(
                f,
                "a monomial with no factors was built where one is not admitted; \
                 Monomial::scalar is the constructor that admits it"
            ),
            MonomialError::NameOnMoreThanTwoSlots { name, slots } => write!(
                f,
                "the index {name} is on {slots} slots; one slot is a free index and two are a \
                 contraction, and more is neither"
            ),
            MonomialError::ContractionInOneVarianceWithoutMetric { name, variance } => write!(
                f,
                "the index {name} is contracted with both halves {variance} on a manifold \
                 declared without a metric, where nothing raises or lowers either half"
            ),
        }
    }
}

impl core::error::Error for MonomialError {}

/// A product of tensor factors with a rational coefficient.
///
/// # Worked examples
///
/// All free. Two indices, each on one slot, neither of them renameable.
///
/// ```
/// use indexwerk_core::expression::{Manifold, Monomial};
///
/// # fn main() -> Result<(), Box<dyn core::error::Error>> {
/// let monomial: Monomial = "metric: 1 * T[^a,_b]".parse()?;
/// assert_eq!(monomial.manifold(), Manifold::WithMetric);
/// assert_eq!(monomial.free_indices().len(), 2);
/// assert!(monomial.dummy_pairs().is_empty());
/// # Ok(()) }
/// ```
///
/// All dummy. A Riemann monomial contracted with itself: two pairs, no free
/// index, and every name renameable.
///
/// ```
/// use indexwerk_core::expression::Monomial;
///
/// # fn main() -> Result<(), Box<dyn core::error::Error>> {
/// let monomial: Monomial = "metric: 1 * R[^a,^b,_a,_b]".parse()?;
/// assert!(monomial.free_indices().is_empty());
/// assert_eq!(monomial.dummy_pairs().len(), 2);
/// # Ok(()) }
/// ```
///
/// Mixed, on a manifold with no metric, so each pair sits one half upper and
/// one half lower. `b` is the pair; `a` and `c` are free.
///
/// ```
/// use indexwerk_core::expression::Monomial;
///
/// # fn main() -> Result<(), Box<dyn core::error::Error>> {
/// let monomial: Monomial = "no-metric: 1 * T[^a,_b] * S[^b,_c]".parse()?;
/// let pairs = monomial.dummy_pairs();
/// assert_eq!(pairs.len(), 1);
/// assert_eq!(pairs[0].name.as_str(), "b");
/// assert_eq!(monomial.free_indices().len(), 2);
/// # Ok(()) }
/// ```
///
/// A coefficient other than one. It is exact, it is in lowest terms, and it
/// carries the sign.
///
/// ```
/// use indexwerk_core::expression::Monomial;
///
/// # fn main() -> Result<(), Box<dyn core::error::Error>> {
/// let monomial: Monomial = "metric: -6/4 * R[^a,_a]".parse()?;
/// assert_eq!(monomial.coefficient().numerator(), -3);
/// assert_eq!(monomial.coefficient().denominator(), 2);
/// assert_eq!(monomial.to_string(), "metric: -3/2 * R[^a,_a]");
/// # Ok(()) }
/// ```
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Monomial {
    manifold: Manifold,
    coefficient: Rational,
    factors: Vec<Factor>,
}

impl Monomial {
    /// A monomial with at least one factor.
    ///
    /// # Errors
    ///
    /// [`MonomialError`], one variant per defect, each naming the index or the
    /// count it is about.
    pub fn new(
        manifold: Manifold,
        coefficient: Rational,
        factors: Vec<Factor>,
    ) -> Result<Self, MonomialError> {
        if factors.is_empty() {
            return Err(MonomialError::Empty);
        }
        let monomial = Monomial {
            manifold,
            coefficient,
            factors,
        };
        monomial.check_indices()?;
        Ok(monomial)
    }

    /// A monomial with no factors: a bare coefficient.
    ///
    /// This is the constructor that admits what [`Monomial::new`] refuses, and
    /// it is separate so that the refusal is what a caller gets by accident and
    /// the scalar is what a caller gets on purpose.
    pub const fn scalar(manifold: Manifold, coefficient: Rational) -> Self {
        Monomial {
            manifold,
            coefficient,
            factors: Vec::new(),
        }
    }

    /// The occurrence rules, applied to the slots in the order they appear.
    fn check_indices(&self) -> Result<(), MonomialError> {
        let mut seen: Vec<(&IndexName, Vec<Variance>)> = Vec::new();
        for slot in self.factors.iter().flat_map(Factor::slots) {
            match seen.iter_mut().find(|(name, _)| *name == slot.name()) {
                Some((_, variances)) => variances.push(slot.variance()),
                None => seen.push((slot.name(), vec![slot.variance()])),
            }
        }
        for (name, variances) in &seen {
            if variances.len() > 2 {
                return Err(MonomialError::NameOnMoreThanTwoSlots {
                    name: (*name).clone(),
                    slots: variances.len(),
                });
            }
        }
        if self.manifold == Manifold::WithoutMetric {
            for (name, variances) in &seen {
                match variances.as_slice() {
                    [first, second] if first == second => {
                        return Err(MonomialError::ContractionInOneVarianceWithoutMetric {
                            name: (*name).clone(),
                            variance: *first,
                        });
                    }
                    _ => {}
                }
            }
        }
        Ok(())
    }

    pub const fn manifold(&self) -> Manifold {
        self.manifold
    }

    pub const fn coefficient(&self) -> Rational {
        self.coefficient
    }

    pub fn factors(&self) -> &[Factor] {
        &self.factors
    }

    /// Every slot of every factor, left to right.
    pub fn slots(&self) -> impl Iterator<Item = &Slot> {
        self.factors.iter().flat_map(Factor::slots)
    }

    /// The names on exactly one slot, in the order they first appear. These may
    /// not be renamed.
    pub fn free_indices(&self) -> Vec<IndexName> {
        self.occurrences()
            .into_iter()
            .filter(|(_, variances)| variances.len() == 1)
            .map(|(name, _)| name)
            .collect()
    }

    /// The names on exactly two slots, in the order they first appear. These
    /// may be renamed to any name not already in the monomial.
    pub fn dummy_pairs(&self) -> Vec<DummyPair> {
        self.occurrences()
            .into_iter()
            .filter_map(|(name, variances)| match variances.as_slice() {
                [first, second] => Some(DummyPair {
                    name,
                    first: *first,
                    second: *second,
                }),
                _ => None,
            })
            .collect()
    }

    fn occurrences(&self) -> Vec<(IndexName, Vec<Variance>)> {
        let mut seen: Vec<(IndexName, Vec<Variance>)> = Vec::new();
        for slot in self.slots() {
            match seen.iter_mut().find(|(name, _)| name == slot.name()) {
                Some((_, variances)) => variances.push(slot.variance()),
                None => seen.push((slot.name().clone(), vec![slot.variance()])),
            }
        }
        seen
    }
}

impl fmt::Display for Monomial {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.manifold, self.coefficient)?;
        for factor in &self.factors {
            write!(f, " * {factor}")?;
        }
        Ok(())
    }
}

/// Why a line of text is not a monomial.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum TextError {
    /// No `": "` separating the manifold from the rest.
    NoManifold { text: String },
    /// A manifold token that is neither of the two.
    UnknownManifold { token: String },
    /// The coefficient did not read as a rational.
    Coefficient(RationalParseError),
    /// A factor without its bracketed slot list.
    MalformedFactor { text: String },
    /// A slot without its variance marker.
    MalformedSlot { text: String },
    /// A name inside the line is not a name.
    Name(NameError),
    /// The line parsed and the value it describes is not a factor.
    Factor(FactorError),
    /// The line parsed and the value it describes is not a monomial.
    Monomial(MonomialError),
}

impl fmt::Display for TextError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            TextError::NoManifold { text } => write!(
                f,
                "{text:?} does not start with a manifold followed by \": \""
            ),
            TextError::UnknownManifold { token } => write!(
                f,
                "{token:?} is neither \"{}\" nor \"{}\"",
                Manifold::WithMetric.token(),
                Manifold::WithoutMetric.token()
            ),
            TextError::Coefficient(reason) => write!(f, "the coefficient is unreadable: {reason}"),
            TextError::MalformedFactor { text } => write!(
                f,
                "{text:?} is not a factor: a factor is a name followed by its slots in square \
                 brackets"
            ),
            TextError::MalformedSlot { text } => write!(
                f,
                "{text:?} is not a slot: a slot is \"^\" or \"_\" followed by an index name"
            ),
            TextError::Name(reason) => write!(f, "{reason}"),
            TextError::Factor(reason) => write!(f, "{reason}"),
            TextError::Monomial(reason) => write!(f, "{reason}"),
        }
    }
}

impl core::error::Error for TextError {}

impl From<NameError> for TextError {
    fn from(reason: NameError) -> Self {
        TextError::Name(reason)
    }
}

impl From<FactorError> for TextError {
    fn from(reason: FactorError) -> Self {
        TextError::Factor(reason)
    }
}

impl From<MonomialError> for TextError {
    fn from(reason: MonomialError) -> Self {
        TextError::Monomial(reason)
    }
}

fn parse_slot(text: &str) -> Result<Slot, TextError> {
    let mut characters = text.chars();
    let variance = match characters.next() {
        Some('^') => Variance::Upper,
        Some('_') => Variance::Lower,
        _ => {
            return Err(TextError::MalformedSlot {
                text: text.to_owned(),
            });
        }
    };
    let name = characters.as_str();
    Ok(Slot::new(IndexName::new(name)?, variance))
}

fn parse_factor(text: &str) -> Result<Factor, TextError> {
    let stripped = match text.strip_suffix(']') {
        Some(stripped) => stripped,
        None => {
            return Err(TextError::MalformedFactor {
                text: text.to_owned(),
            });
        }
    };
    let (name, inner) = match stripped.split_once('[') {
        Some(parts) => parts,
        None => {
            return Err(TextError::MalformedFactor {
                text: text.to_owned(),
            });
        }
    };
    let slots = if inner.is_empty() {
        Vec::new()
    } else {
        inner
            .split(',')
            .map(parse_slot)
            .collect::<Result<Vec<Slot>, TextError>>()?
    };
    // The rank comes from the slots the text supplies, so this route cannot
    // produce the rank mismatch the programmatic route can. Building the factor
    // through the same constructor anyway is what keeps the two routes from
    // drifting into two definitions of a factor.
    let tensor = Tensor::new(TensorName::new(name)?, slots.len());
    Factor::new(tensor, slots).map_err(TextError::Factor)
}

impl FromStr for Monomial {
    type Err = TextError;

    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let (manifold_token, rest) = match text.split_once(": ") {
            Some(parts) => parts,
            None => {
                return Err(TextError::NoManifold {
                    text: text.to_owned(),
                });
            }
        };
        let manifold = if manifold_token == Manifold::WithMetric.token() {
            Manifold::WithMetric
        } else if manifold_token == Manifold::WithoutMetric.token() {
            Manifold::WithoutMetric
        } else {
            return Err(TextError::UnknownManifold {
                token: manifold_token.to_owned(),
            });
        };
        let mut parts = rest.split(" * ");
        let coefficient_text = parts.next().unwrap_or_default();
        let coefficient = coefficient_text
            .parse::<Rational>()
            .map_err(TextError::Coefficient)?;
        let factors = parts
            .map(parse_factor)
            .collect::<Result<Vec<Factor>, TextError>>()?;
        if factors.is_empty() {
            return Ok(Monomial::scalar(manifold, coefficient));
        }
        Monomial::new(manifold, coefficient, factors).map_err(TextError::Monomial)
    }
}
