//! Permutations of index slots, their signs, and the total order the canonical
//! form is a minimum in.
//!
//! The lowest layer of the engine. Everything above it is built on the
//! representation chosen here, and issue #20 is where that shape was argued.
//!
//! A permutation is stored as an image array whose width is a runtime value.
//! The number of index slots varies from one expression to the next, and a
//! width fixed at compile time would mean a rebuild to canonicalise a bigger
//! tensor.
//!
//! # The sign
//!
//! A slot symmetry is signed: exchanging two slots of an antisymmetric tensor
//! is not the same object as exchanging two slots of a symmetric one. So a
//! [`Signed`] permutation carries a [`Sign`], and the sign multiplies under
//! composition.
//!
//! [`Sign::Zero`] is the third value and it is not a decoration. An
//! antisymmetric pair carrying the same dummy on both slots is zero, and that is
//! the first real case anybody meets rather than an edge case. A zero sign is
//! representable, it survives composition in both directions, and it has no
//! inverse, which is why [`Signed::inverse`] returns an [`Option`].
//!
//! # The order
//!
//! The order is not chosen here. Section 7 of `docs/normal-form.md` fixes it,
//! that document is the authority, and
//! `conformance/order/permutations.txt` is the fixture it quotes. The suite
//! sorts that fixture with the implementation below and requires the result to
//! be the file, so a disagreement between the two reds the build rather than
//! waiting to be noticed by a reader.
//!
//! In short: width first, narrower first, then the image array position by
//! position, then the sign with positive before negative before zero. The
//! reasons for each of the three are in that section and are not repeated here.

use core::cmp::Ordering;
use core::fmt;

/// The sign a symmetry carries.
///
/// The declaration order is the order of section 7 of `docs/normal-form.md`,
/// positive before negative before zero, and [`Ord`] is derived from it. Moving
/// a variant here moves the order, which is why the fixture in
/// `conformance/order/permutations.txt` carries all three for every permutation
/// it lists.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub enum Sign {
    Plus,
    Minus,
    /// The expression vanishes by its own symmetry.
    Zero,
}

impl Sign {
    /// The product of two signs.
    ///
    /// Zero absorbs, which is what makes a vanishing monomial survive every
    /// further composition instead of being multiplied back into existence.
    pub const fn times(self, other: Sign) -> Sign {
        match (self, other) {
            (Sign::Zero, _) | (_, Sign::Zero) => Sign::Zero,
            (Sign::Plus, Sign::Plus) | (Sign::Minus, Sign::Minus) => Sign::Plus,
            (Sign::Plus, Sign::Minus) | (Sign::Minus, Sign::Plus) => Sign::Minus,
        }
    }

    /// The character the fixture and the normal form document write.
    pub const fn marker(self) -> char {
        match self {
            Sign::Plus => '+',
            Sign::Minus => '-',
            Sign::Zero => '0',
        }
    }
}

impl fmt::Display for Sign {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.marker())
    }
}

/// Why a permutation was refused.
#[derive(Clone, PartialEq, Eq, Debug)]
pub enum PermutationError {
    /// An image outside `0..width`. The position it sits at is named, because
    /// an image array is read by position and a value alone does not say where
    /// to look.
    ImageOutOfRange {
        position: usize,
        image: usize,
        width: usize,
    },
    /// One point is the image of two positions, so the array is a map and not a
    /// bijection.
    RepeatedImage {
        image: usize,
        first: usize,
        second: usize,
    },
    /// Two permutations of different widths were composed. They act on
    /// different sets of points and no convention makes them one group.
    WidthMismatch { left: usize, right: usize },
}

impl fmt::Display for PermutationError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PermutationError::ImageOutOfRange {
                position,
                image,
                width,
            } => write!(
                f,
                "position {position} carries the image {image}, which is outside the \
                 {width} points this permutation is on"
            ),
            PermutationError::RepeatedImage {
                image,
                first,
                second,
            } => write!(
                f,
                "the point {image} is the image of both position {first} and position \
                 {second}, so this array is not a bijection"
            ),
            PermutationError::WidthMismatch { left, right } => write!(
                f,
                "a permutation of {left} points was composed with one of {right}; they act \
                 on different points and no convention merges them"
            ),
        }
    }
}

impl core::error::Error for PermutationError {}

/// A permutation of `width` points, stored as an image array.
///
/// `image(position)` is where `position` goes. The width is carried rather than
/// inferred at each call so that two permutations of different widths are
/// refused rather than silently padded.
///
/// # Examples
///
/// ```
/// use indexwerk_core::permutation::Permutation;
///
/// # fn main() -> Result<(), Box<dyn core::error::Error>> {
/// let swap = Permutation::new(vec![1, 0, 2])?;
/// assert_eq!(swap.image(0), Some(1));
/// assert_eq!(swap.then(&swap)?, Permutation::identity(3));
/// # Ok(()) }
/// ```
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Permutation {
    images: Vec<usize>,
}

impl Permutation {
    /// Build a permutation from its image array.
    ///
    /// # Errors
    ///
    /// [`PermutationError::ImageOutOfRange`] for an image outside the width,
    /// and [`PermutationError::RepeatedImage`] where the array is not a
    /// bijection. Between them they are the whole invariant every other method
    /// on this type relies on.
    pub fn new(images: Vec<usize>) -> Result<Self, PermutationError> {
        let width = images.len();
        let mut seen: Vec<Option<usize>> = vec![None; width];
        for (position, &image) in images.iter().enumerate() {
            if image >= width {
                return Err(PermutationError::ImageOutOfRange {
                    position,
                    image,
                    width,
                });
            }
            // `image < width` was just established and `seen` has exactly
            // `width` entries, so both of these are inside the vector.
            if let Some(first) = seen[image] {
                return Err(PermutationError::RepeatedImage {
                    image,
                    first,
                    second: position,
                });
            }
            seen[image] = Some(position);
        }
        Ok(Permutation { images })
    }

    /// The permutation that moves nothing.
    pub fn identity(width: usize) -> Self {
        Permutation {
            images: (0..width).collect(),
        }
    }

    /// How many points this permutation is on.
    pub fn width(&self) -> usize {
        self.images.len()
    }

    /// The image array, in position order.
    pub fn images(&self) -> &[usize] {
        &self.images
    }

    /// Where a point goes, or [`None`] where the point is outside the width.
    pub fn image(&self, point: usize) -> Option<usize> {
        self.images.get(point).copied()
    }

    /// This permutation followed by `other`.
    ///
    /// Application order rather than the other convention: `p.then(q)` sends a
    /// point through `p` and then through `q`. The name says which it is,
    /// because the two conventions differ by an inversion and a library that
    /// leaves the choice to a reader has picked the wrong one for half of them.
    ///
    /// # Errors
    ///
    /// [`PermutationError::WidthMismatch`] where the widths differ.
    pub fn then(&self, other: &Self) -> Result<Self, PermutationError> {
        if self.width() != other.width() {
            return Err(PermutationError::WidthMismatch {
                left: self.width(),
                right: other.width(),
            });
        }
        let images = self
            .images
            .iter()
            // Every image of a permutation of this width is a point of it, and
            // the widths were just compared, so `other.images` is indexed
            // inside its length.
            .map(|&point| other.images[point])
            .collect();
        Ok(Permutation { images })
    }

    /// The two-sided inverse.
    ///
    /// It always exists: the constructor refuses an array that is not a
    /// bijection, so there is no case here to return an error for.
    pub fn inverse(&self) -> Self {
        let mut images = vec![0; self.width()];
        for (position, &image) in self.images.iter().enumerate() {
            // `image` is a point of this permutation by the constructor's
            // invariant, so it indexes a vector of exactly this width.
            images[image] = position;
        }
        Permutation { images }
    }
}

impl Ord for Permutation {
    /// Section 7 of `docs/normal-form.md`: width first, narrower first, then
    /// the image array position by position.
    ///
    /// Comparing the arrays alone would not do. `[1, 0]` and `[0, 1, 2]` are
    /// ordered one way by width and the other way by a lexicographic
    /// comparison that treats a prefix as smaller, and the document picks
    /// width.
    fn cmp(&self, other: &Self) -> Ordering {
        self.images
            .len()
            .cmp(&other.images.len())
            .then_with(|| self.images.cmp(&other.images))
    }
}

impl PartialOrd for Permutation {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

/// A permutation with the sign its symmetry carries.
///
/// # Examples
///
/// ```
/// use indexwerk_core::permutation::{Permutation, Sign, Signed};
///
/// # fn main() -> Result<(), Box<dyn core::error::Error>> {
/// let swap = Signed::new(Permutation::new(vec![1, 0])?, Sign::Minus);
/// assert_eq!(swap.then(&swap)?.sign(), Sign::Plus);
///
/// let vanishes = Signed::new(Permutation::identity(2), Sign::Zero);
/// assert_eq!(vanishes.then(&swap)?.sign(), Sign::Zero);
/// assert_eq!(vanishes.inverse(), None);
/// # Ok(()) }
/// ```
#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Signed {
    permutation: Permutation,
    sign: Sign,
}

impl Signed {
    pub const fn new(permutation: Permutation, sign: Sign) -> Self {
        Signed { permutation, sign }
    }

    /// The identity, positively signed.
    pub fn identity(width: usize) -> Self {
        Signed::new(Permutation::identity(width), Sign::Plus)
    }

    pub const fn permutation(&self) -> &Permutation {
        &self.permutation
    }

    pub const fn sign(&self) -> Sign {
        self.sign
    }

    /// How many points this acts on.
    pub fn width(&self) -> usize {
        self.permutation.width()
    }

    /// This element followed by `other`, with the signs multiplied.
    ///
    /// # Errors
    ///
    /// [`PermutationError::WidthMismatch`] where the widths differ.
    pub fn then(&self, other: &Self) -> Result<Self, PermutationError> {
        Ok(Signed::new(
            self.permutation.then(&other.permutation)?,
            self.sign.times(other.sign),
        ))
    }

    /// The two-sided inverse, or [`None`] where the sign is zero.
    ///
    /// A zero sign records that the expression vanishes. That is not a group
    /// element and it has no inverse, and returning one anyway would let a
    /// caller compose their way back out of a vanishing result.
    pub fn inverse(&self) -> Option<Self> {
        match self.sign {
            Sign::Zero => None,
            sign => Some(Signed::new(self.permutation.inverse(), sign)),
        }
    }
}

impl Ord for Signed {
    /// Section 7 of `docs/normal-form.md`: the permutation first, the sign
    /// second.
    fn cmp(&self, other: &Self) -> Ordering {
        self.permutation
            .cmp(&other.permutation)
            .then_with(|| self.sign.cmp(&other.sign))
    }
}

impl PartialOrd for Signed {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
