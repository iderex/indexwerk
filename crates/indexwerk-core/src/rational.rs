//! The exact rational a monomial carries as its coefficient.
//!
//! There is no floating point here and there cannot be: the core refuses the
//! types outright, and `docs/adr/0007-exact-arithmetic.md` is where that was
//! decided. A coefficient of a third is a third, not the nearest binary
//! fraction to one, because two monomials collect exactly when their
//! coefficients add exactly.
//!
//! A value is always in lowest terms with a positive denominator, so equality
//! is the derived one and two spellings of one number are one value. That is
//! what lets a canonical form be compared with `==` rather than with a
//! normalising comparison somebody has to remember to call.

use core::fmt;

/// A rational number in lowest terms.
///
/// The sign is carried in the numerator and nowhere else. A monomial has one
/// sign, and a sign stored beside the coefficient as well as inside it is one
/// fact in two places, which is a fact that can disagree with itself.
#[derive(Clone, Copy, PartialEq, Eq, Hash, Debug)]
pub struct Rational {
    numerator: i64,
    denominator: i64,
}

/// Why a rational could not be built.
#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum RationalError {
    /// A denominator of zero. Refused at construction rather than divided by
    /// later.
    ZeroDenominator,
    /// Reducing the fraction did not fit back into the width it came from.
    /// `i64::MIN` over an odd negative denominator is the case that reaches
    /// here, and it reaches here rather than wrapping.
    OutOfRange { numerator: i64, denominator: i64 },
}

impl fmt::Display for RationalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RationalError::ZeroDenominator => write!(f, "a rational cannot have denominator zero"),
            RationalError::OutOfRange {
                numerator,
                denominator,
            } => write!(
                f,
                "{numerator}/{denominator} does not fit in this representation once reduced"
            ),
        }
    }
}

impl core::error::Error for RationalError {}

/// Greatest common divisor of two values that are not negative, iterative so it
/// needs no stack depth argument.
fn gcd(mut a: i128, mut b: i128) -> i128 {
    while b != 0 {
        let remainder = a % b;
        a = b;
        b = remainder;
    }
    a
}

impl Rational {
    /// The rational `numerator / denominator`, reduced.
    ///
    /// # Errors
    ///
    /// [`RationalError::ZeroDenominator`] when the denominator is zero, and
    /// [`RationalError::OutOfRange`] when the reduced form does not fit.
    pub fn new(numerator: i64, denominator: i64) -> Result<Self, RationalError> {
        if denominator == 0 {
            return Err(RationalError::ZeroDenominator);
        }
        let mut wide_numerator = i128::from(numerator);
        let mut wide_denominator = i128::from(denominator);
        if wide_denominator < 0 {
            wide_numerator = -wide_numerator;
            wide_denominator = -wide_denominator;
        }
        if wide_numerator == 0 {
            return Ok(Rational {
                numerator: 0,
                denominator: 1,
            });
        }
        let divisor = gcd(wide_numerator.abs(), wide_denominator);
        let reduced_numerator = wide_numerator / divisor;
        let reduced_denominator = wide_denominator / divisor;
        match (
            i64::try_from(reduced_numerator),
            i64::try_from(reduced_denominator),
        ) {
            (Ok(numerator), Ok(denominator)) => Ok(Rational {
                numerator,
                denominator,
            }),
            _ => Err(RationalError::OutOfRange {
                numerator,
                denominator,
            }),
        }
    }

    /// The rational `value / 1`. Cannot fail, so it is not a `Result`.
    pub const fn integer(value: i64) -> Self {
        Rational {
            numerator: value,
            denominator: 1,
        }
    }

    /// One.
    pub const ONE: Rational = Rational::integer(1);

    /// Zero.
    pub const ZERO: Rational = Rational::integer(0);

    pub const fn numerator(self) -> i64 {
        self.numerator
    }

    /// Always positive.
    pub const fn denominator(self) -> i64 {
        self.denominator
    }

    pub const fn is_zero(self) -> bool {
        self.numerator == 0
    }
}

impl fmt::Display for Rational {
    /// The text form: `3`, `-3`, `3/2`. The denominator is written only when it
    /// is not one, and it is never negative, because the value it renders never
    /// is.
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.denominator == 1 {
            write!(f, "{}", self.numerator)
        } else {
            write!(f, "{}/{}", self.numerator, self.denominator)
        }
    }
}

/// Why a rational could not be read back out of text.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum RationalParseError {
    /// More than one solidus, so it is not a fraction.
    NotAFraction { text: String },
    /// A part that is not a decimal integer.
    NotAnInteger { text: String },
    /// The parts were integers and the pair is not a rational.
    NotARational { text: String, reason: RationalError },
}

impl fmt::Display for RationalParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RationalParseError::NotAFraction { text } => {
                write!(f, "{text:?} is not a fraction")
            }
            RationalParseError::NotAnInteger { text } => {
                write!(f, "{text:?} is not a decimal integer")
            }
            RationalParseError::NotARational { text, reason } => {
                write!(f, "{text:?} is not a rational: {reason}")
            }
        }
    }
}

impl core::error::Error for RationalParseError {}

impl core::str::FromStr for Rational {
    type Err = RationalParseError;

    /// Reads `3`, `-3` and `3/2`. Nothing else, and in particular no
    /// surrounding whitespace, because the caller of a round trip is entitled
    /// to compare bytes.
    fn from_str(text: &str) -> Result<Self, Self::Err> {
        let mut parts = text.split('/');
        let numerator_text = parts.next().unwrap_or_default();
        let denominator_text = parts.next();
        if parts.next().is_some() {
            return Err(RationalParseError::NotAFraction {
                text: text.to_owned(),
            });
        }
        let numerator = match numerator_text.parse::<i64>() {
            Ok(value) => value,
            Err(_) => {
                return Err(RationalParseError::NotAnInteger {
                    text: numerator_text.to_owned(),
                });
            }
        };
        let denominator = match denominator_text {
            None => 1,
            Some(part) => match part.parse::<i64>() {
                Ok(value) => value,
                Err(_) => {
                    return Err(RationalParseError::NotAnInteger {
                        text: part.to_owned(),
                    });
                }
            },
        };
        Rational::new(numerator, denominator).map_err(|reason| RationalParseError::NotARational {
            text: text.to_owned(),
            reason,
        })
    }
}
