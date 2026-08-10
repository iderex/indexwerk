//! A base, a strong generating set and the stabiliser chain, computed by
//! Schreier-Sims.
//!
//! A symmetry group arrives as a handful of generators. That form answers
//! almost nothing: it does not say how large the group is, it does not say
//! whether a given permutation is in it, and it gives a search no structure to
//! descend through. The chain built here is what turns those generators into
//! something that can be answered from without writing the group out. Issue #21
//! is where the shape was argued.
//!
//! # What the chain is
//!
//! Pick a point `b0`. The elements fixing it form a subgroup. Pick a second
//! point `b1` and the elements fixing both form a subgroup of that one, and so
//! on until only the identity is left. The points are the base, the chain of
//! subgroups is the stabiliser chain, and a strong generating set is a set of
//! generators that also generates every subgroup in the chain when it is cut
//! down to the elements fixing the earlier base points.
//!
//! Each level carries the orbit of its base point under that level's
//! generators, and a transversal: for every point in the orbit, one group
//! element taking the base point there. This is the Schreier vector the issue
//! asks for, stored as the element itself rather than as the generator index it
//! is usually written as. The element costs more memory and it removes the walk
//! back up the vector at every lookup, and the search of M5 does that lookup far
//! more often than it builds a chain.
//!
//! Two things follow at once. The order of the group is the product of the
//! orbit lengths, because every element is one transversal element per level
//! composed together, in exactly one way. And membership is decided by
//! stripping: take the point the candidate sends the base to, divide out the
//! transversal element that does the same, and repeat down the chain. What is
//! left is the identity exactly when the candidate was in the group.
//!
//! # Which version this is
//!
//! The deterministic one. A randomised variant tests fewer Schreier generators
//! and is faster on large inputs, and its result is a probability until it is
//! verified; issue #21 asks for the version whose answer is not one, and a
//! randomised variant is a later addition beside this rather than a replacement
//! for it.
//!
//! The algorithm is Sims', in the form set out in Holt, Eick and O'Brien,
//! *Handbook of Computational Group Theory*, Chapman and Hall/CRC 2005, section
//! 4.4.2, and in Seress, *Permutation Group Algorithms*, Cambridge University
//! Press 2003, chapter 4. Schreier's lemma, that the Schreier generators
//! generate the stabiliser, is what makes the level-by-level construction
//! finite.
//!
//! # Where the caching boundary is
//!
//! A chain is built once per symmetry, not once per expression. The slot
//! symmetry group of a Riemann monomial with a dozen tensors is large, and the
//! same symmetry is met again on every term of a sum and on every expression
//! written against the same tensors, so a chain rebuilt per expression would be
//! the whole cost of a canonicalisation spent on something that did not change.
//!
//! [`ChainCache`] is that boundary, and it is here rather than in the
//! canonicaliser because the thing being reused is this module's and because a
//! cache placed at the caller is a cache each caller reinvents. It is keyed on
//! the generators in the order they were given, which is the same thing
//! [`StabiliserChain::new`] is a function of, and it is an ordinary owned value
//! with no interior mutability and no global: a caller holds one and passes it
//! in, so two callers cannot share one by accident and nothing here has to be
//! thread-safe to be correct.
//!
//! # Determinism
//!
//! The chain is a function of the generators and their order, and of nothing
//! else. The base points are the smallest moved point at each level, the orbits
//! are enumerated breadth first in generator order, and every list here is a
//! [`Vec`] rather than a set with an unspecified iteration order. That matters
//! beyond tidiness: `docs/adr/0006-determinism.md` says a result may not depend
//! on how the work was run, and a chain that came out differently on two runs
//! would make the canonical form differ through the back door, because the
//! search of M5 descends the chain in the order it is written.
//!
//! [`StabiliserChain::render`] is the text the determinism test compares byte
//! for byte.
//!
//! # What is not here
//!
//! Signs. A [`crate::permutation::Signed`] carries the sign its symmetry
//! multiplies by, and the sign is not part of the group this module computes
//! with: the base and the strong generating set are properties of the
//! underlying permutation group, and a zero sign is not a group element at all.
//! Declaring a symmetry with its signs is #25 and using them is #26.

use crate::permutation::{Permutation, PermutationError};

/// One level of the stabiliser chain.
///
/// The generators are the ones that fix every base point above this level. The
/// orbit and the transversal are of this level's base point under those
/// generators, and they are recomputed whenever the generators change rather
/// than being patched, so a level's three fields cannot disagree.
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct Level {
    base: usize,
    generators: Vec<Permutation>,
    orbit: Vec<usize>,
    transversal: Vec<Option<Permutation>>,
}

impl Level {
    /// The point this level stabilises.
    pub fn base(&self) -> usize {
        self.base
    }

    /// The generators of this level's subgroup, in the order they were added.
    pub fn generators(&self) -> &[Permutation] {
        &self.generators
    }

    /// The points the base reaches, in the order the orbit was discovered.
    ///
    /// Discovery order rather than sorted order, because it is the order the
    /// transversal was built in and a reader comparing the two wants them to
    /// line up. The length of this is what the group order is a product of.
    pub fn orbit(&self) -> &[usize] {
        &self.orbit
    }

    /// The element of this level's subgroup taking the base point to `point`,
    /// or [`None`] where `point` is outside the orbit.
    pub fn transversal(&self, point: usize) -> Option<&Permutation> {
        match self.transversal.get(point) {
            Some(entry) => entry.as_ref(),
            None => None,
        }
    }
}

/// A base, a strong generating set and the stabiliser chain they belong to.
///
/// # Examples
///
/// ```
/// use indexwerk_core::permutation::Permutation;
/// use indexwerk_core::stabiliser_chain::StabiliserChain;
///
/// # fn main() -> Result<(), Box<dyn core::error::Error>> {
/// // The slot symmetry of the Riemann tensor without the first Bianchi
/// // identity: antisymmetric in each pair, symmetric under exchanging them.
/// let chain = StabiliserChain::new(
///     4,
///     vec![
///         Permutation::new(vec![1, 0, 2, 3])?,
///         Permutation::new(vec![0, 1, 3, 2])?,
///         Permutation::new(vec![2, 3, 0, 1])?,
///     ],
/// )?;
///
/// assert_eq!(chain.order(), Some(8));
/// assert!(chain.contains(&Permutation::new(vec![3, 2, 1, 0])?)?);
/// assert!(!chain.contains(&Permutation::new(vec![1, 2, 3, 0])?)?);
/// # Ok(()) }
/// ```
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct StabiliserChain {
    width: usize,
    levels: Vec<Level>,
}

impl StabiliserChain {
    /// Build the chain from generators of a group on `width` points.
    ///
    /// The generators are taken in the order given and the result is a function
    /// of that order. Reordering them gives a chain that describes the same
    /// group and is not the same chain, which is why the order is part of what
    /// [`ChainCache`] keys on.
    ///
    /// # Errors
    ///
    /// [`PermutationError::WidthMismatch`] where a generator is not on `width`
    /// points. The generators of one group act on one set of points, and the
    /// alternative to refusing is padding the narrow ones with fixed points,
    /// which silently answers a question about a different group.
    pub fn new(width: usize, generators: Vec<Permutation>) -> Result<Self, PermutationError> {
        for generator in &generators {
            if generator.width() != width {
                return Err(PermutationError::WidthMismatch {
                    left: width,
                    right: generator.width(),
                });
            }
        }


        let identity = Permutation::identity(width);
        let moving: Vec<Permutation> = generators
            .into_iter()
            .filter(|generator| *generator != identity)
            .collect();

        let mut chain = StabiliserChain {
            width,
            levels: Vec::new(),
        };

        // A group generated by nothing but identities is the trivial group, and
        // it has no level: an empty chain has an empty product for its order,
        // which is one, and the identity is the only permutation that strips to
        // nothing. Both fall out rather than being special-cased below.
        if let Some(first) = moving.first() {
            match first_moved_point(first) {
                Some(base) => chain.levels.push(Level {
                    base,
                    generators: moving,
                    orbit: Vec::new(),
                    transversal: Vec::new(),
                }),
                // `moving` holds no identity, so a first element exists and
                // moves a point. The arm is written rather than assumed because
                // this crate carries no panic path, and an empty chain is the
                // right answer for a group with nothing in it.
                None => return Ok(chain),
            }
            chain.complete_level(0)?;
        }

        Ok(chain)
    }

    /// How many points the group acts on.
    pub fn width(&self) -> usize {
        self.width
    }

    /// The base, in order.
    pub fn base(&self) -> Vec<usize> {
        self.levels.iter().map(Level::base).collect()
    }

    /// The levels of the chain, outermost first.
    pub fn levels(&self) -> &[Level] {
        &self.levels
    }

    /// The strong generating set: every generator of every level, each listed
    /// once, in the order the levels hold them.
    ///
    /// Deduplicated because a generator that fixes the first base points is a
    /// generator of several levels and appears in each, and a reader asking for
    /// the strong generating set is asking for a set.
    pub fn strong_generators(&self) -> Vec<Permutation> {
        let mut strong: Vec<Permutation> = Vec::new();
        for level in &self.levels {
            for generator in &level.generators {
                if !strong.contains(generator) {
                    strong.push(generator.clone());
                }
            }
        }
        strong
    }

    /// The length of each level's orbit, outermost first.
    ///
    /// The group order is the product of these. It is exposed beside
    /// [`StabiliserChain::order`] because the product is the thing that can
    /// exceed what a machine integer holds and the factors are not.
    pub fn basic_orbit_lengths(&self) -> Vec<usize> {
        self.levels.iter().map(|level| level.orbit.len()).collect()
    }

    /// The order of the group, or [`None`] where it does not fit a `u128`.
    ///
    /// The product of the orbit lengths, computed with a checked
    /// multiplication. [`None`] is the honest answer rather than a wrapped one:
    /// a group on 35 points can be larger than any `u128`, the search of M5
    /// never needs the order as a number, and a silently wrapped order would be
    /// a wrong number quoted as a right one. Where the order is wanted for a
    /// group that large, [`StabiliserChain::basic_orbit_lengths`] is the
    /// factorisation of it and nothing is lost.
    pub fn order(&self) -> Option<u128> {
        let mut order: u128 = 1;
        for level in &self.levels {
            // An orbit is a subset of the points, so its length fits every
            // integer type a point index fits, and the widening is exact.
            let length = level.orbit.len() as u128;
            order = order.checked_mul(length)?;
        }
        Some(order)
    }

    /// Whether `candidate` is an element of the group.
    ///
    /// Decided by stripping the candidate down the chain rather than by
    /// searching, so the cost is the depth of the chain and not the size of the
    /// group.
    ///
    /// # Errors
    ///
    /// [`PermutationError::WidthMismatch`] where the candidate is not on this
    /// group's points. A permutation of another width is neither in the group
    /// nor out of it; the question does not typecheck, and answering `false`
    /// would report a mistake as a mathematical fact.
    pub fn contains(&self, candidate: &Permutation) -> Result<bool, PermutationError> {
        if candidate.width() != self.width {
            return Err(PermutationError::WidthMismatch {
                left: self.width,
                right: candidate.width(),
            });
        }
        let (residue, _) = self.strip(candidate, 0)?;
        Ok(residue == Permutation::identity(self.width))
    }

    /// The chain as text, one fact per line.
    ///
    /// This is what "the same generators produce the same chain" is asserted
    /// on. A derived [`core::fmt::Debug`] would do the same job today and it is
    /// not a promise anybody makes about its bytes, whereas this format is one
    /// and moving it is a visible change.
    pub fn render(&self) -> String {
        let mut text = String::new();
        text.push_str(&format!("width {}\n", self.width));
        for (index, level) in self.levels.iter().enumerate() {
            text.push_str(&format!("level {} base {}\n", index, level.base));
            text.push_str(&format!("  orbit {}\n", join_points(&level.orbit)));
            for generator in &level.generators {
                text.push_str(&format!(
                    "  generator {}\n",
                    join_points(generator.images())
                ));
            }
            for &point in &level.orbit {
                if let Some(element) = level.transversal(point) {
                    text.push_str(&format!(
                        "  transversal {} {}\n",
                        point,
                        join_points(element.images())
                    ));
                }
            }
        }
        text
    }

    /// Divide `candidate` down the chain from `from`, and say how far it got.
    ///
    /// The returned permutation fixes the base point of every level it passed,
    /// and the returned index is the level it stopped at, which is the number of
    /// levels where it stopped because the chain ran out.
    fn strip(
        &self,
        candidate: &Permutation,
        from: usize,
    ) -> Result<(Permutation, usize), PermutationError> {
        let mut residue = candidate.clone();
        for (index, level) in self.levels.iter().enumerate().skip(from) {
            let point = match residue.image(level.base) {
                Some(point) => point,
                // The base point is a point of this group and the residue is on
                // this group's width, so it has an image. Returning here is the
                // same answer as an orbit miss and reaches no further.
                None => return Ok((residue, index)),
            };
            match level.transversal(point) {
                Some(element) => residue = residue.then(&element.inverse())?,
                None => return Ok((residue, index)),
            }
        }
        Ok((residue, self.levels.len()))
    }

    /// Recompute one level's orbit and transversal from its generators.
    fn recompute_orbit(&mut self, index: usize) -> Result<(), PermutationError> {
        let width = self.width;
        // `index` is a level of this chain at every call site below, each of
        // which either read it from `self.levels` or has just pushed it.
        let level = &mut self.levels[index];
        let mut orbit = vec![level.base];
        let mut transversal: Vec<Option<Permutation>> = vec![None; width];
        transversal[level.base] = Some(Permutation::identity(width));

        let mut frontier = 0;
        while frontier < orbit.len() {
            let point = orbit[frontier];
            frontier += 1;
            for generator in &level.generators {
                let image = match generator.image(point) {
                    Some(image) => image,
                    // A generator is on this width and `point` came out of the
                    // orbit of a point of it, so this arm is not reached; it
                    // skips rather than aborting because this crate carries no
                    // panic path.
                    None => continue,
                };
                if transversal[image].is_some() {
                    continue;
                }
                let reached = match &transversal[point] {
                    Some(element) => element.then(generator)?,
                    None => continue,
                };
                transversal[image] = Some(reached);
                orbit.push(image);
            }
        }

        level.orbit = orbit;
        level.transversal = transversal;
        Ok(())
    }

    /// Make level `index` and everything below it a stabiliser chain.
    ///
    /// Every Schreier generator of the level is stripped through the levels
    /// below. One that does not strip to the identity is an element of a deeper
    /// stabiliser that the chain did not know about, so it is added as a
    /// generator of every level between this one and where it stopped, and each
    /// of those levels is rebuilt. When the loop finishes with nothing left to
    /// add, Schreier's lemma says the level below generates the stabiliser of
    /// this level's base point, which is what the chain claims.
    fn complete_level(&mut self, index: usize) -> Result<(), PermutationError> {
        self.recompute_orbit(index)?;

        let identity = Permutation::identity(self.width);
        // Taken before the loop and not read again inside it. Nothing below
        // adds a generator to this level, so the orbit and the transversal it
        // was computed from do not move while the Schreier generators are
        // enumerated, which is the condition Schreier's lemma is stated under.
        let orbit = self.levels[index].orbit.clone();
        let generators = self.levels[index].generators.clone();

        for point in orbit {
            let Some(reaching) = self.levels[index].transversal(point).cloned() else {
                continue;
            };
            for generator in &generators {
                let Some(image) = generator.image(point) else {
                    continue;
                };
                let Some(returning) = self.levels[index].transversal(image).cloned() else {
                    continue;
                };

                let schreier = reaching.then(generator)?.then(&returning.inverse())?;
                if schreier == identity {
                    continue;
                }

                let (residue, stopped) = self.strip(&schreier, index + 1)?;
                if residue == identity {
                    continue;
                }

                if stopped == self.levels.len() {
                    match first_moved_point(&residue) {
                        Some(base) => self.levels.push(Level {
                            base,
                            generators: Vec::new(),
                            orbit: Vec::new(),
                            transversal: Vec::new(),
                        }),
                        // The residue is not the identity, so it moves a point.
                        None => continue,
                    }
                }

                for level in self.levels.iter_mut().take(stopped + 1).skip(index + 1) {
                    level.generators.push(residue.clone());
                }
                for deeper in ((index + 1)..=stopped).rev() {
                    self.complete_level(deeper)?;
                }
            }
        }

        Ok(())
    }
}

/// Chains kept for the symmetries already met.
///
/// Why this exists at all is in the module documentation. What it is: an owned
/// value a caller holds, keyed on the width and the generators in order, that
/// counts how many chains it has actually built so a test can show a repeat
/// request did not build a second one.
///
/// The store is a list rather than a hash map, and the reason is not size. A
/// hash map iterates in an order this tree does not control, and a cache whose
/// contents are read back in an arbitrary order is a place a nondeterministic
/// answer can enter a canonical form. There is nothing here to iterate today,
/// and the shape that could not is cheaper than the note explaining why the one
/// that could is safe.
///
/// # Examples
///
/// ```
/// use indexwerk_core::permutation::Permutation;
/// use indexwerk_core::stabiliser_chain::ChainCache;
///
/// # fn main() -> Result<(), Box<dyn core::error::Error>> {
/// let symmetry = vec![Permutation::new(vec![1, 0, 2])?];
/// let mut cache = ChainCache::new();
///
/// let order = cache.chain(3, &symmetry)?.order();
/// assert_eq!(order, Some(2));
/// assert_eq!(cache.computations(), 1);
///
/// let again = cache.chain(3, &symmetry)?.order();
/// assert_eq!(again, order);
/// assert_eq!(cache.computations(), 1);
/// # Ok(()) }
/// ```
#[derive(Clone, Default, Debug)]
pub struct ChainCache {
    entries: Vec<Entry>,
    computations: usize,
}

#[derive(Clone, Debug)]
struct Entry {
    width: usize,
    generators: Vec<Permutation>,
    chain: StabiliserChain,
}

impl ChainCache {
    pub fn new() -> Self {
        ChainCache::default()
    }

    /// The chain for this symmetry, built on the first request and reused
    /// afterwards.
    ///
    /// # Errors
    ///
    /// Whatever [`StabiliserChain::new`] refuses, and only on a request that
    /// reaches it. A refused symmetry is not stored, so the refusal is the same
    /// on every request rather than being cached as a chain that does not exist.
    pub fn chain(
        &mut self,
        width: usize,
        generators: &[Permutation],
    ) -> Result<&StabiliserChain, PermutationError> {
        let index = match self.position(width, generators) {
            Some(index) => index,
            None => {
                let chain = StabiliserChain::new(width, generators.to_vec())?;
                self.computations += 1;
                self.entries.push(Entry {
                    width,
                    generators: generators.to_vec(),
                    chain,
                });
                self.entries.len() - 1
            }
        };
        // `index` is either a position just read out of `entries` or the last
        // index of it after a push, and nothing removes an entry.
        Ok(&self.entries[index].chain)
    }

    /// How many chains this cache has built.
    ///
    /// The number a test reads to show that a repeat request was answered from
    /// the store. It counts builds and not requests, which are different
    /// numbers and the second one is not kept.
    pub fn computations(&self) -> usize {
        self.computations
    }

    /// How many symmetries are stored.
    pub fn len(&self) -> usize {
        self.entries.len()
    }

    /// Whether anything is stored.
    pub fn is_empty(&self) -> bool {
        self.entries.is_empty()
    }

    fn position(&self, width: usize, generators: &[Permutation]) -> Option<usize> {
        self.entries
            .iter()
            .position(|entry| entry.width == width && entry.generators == generators)
    }
}

/// The smallest point a permutation moves, or [`None`] for the identity.
///
/// Smallest rather than first-found, so that the base of a level is a function
/// of the permutation and not of how it was reached. The base points a chain
/// ends up with are what the search of M5 descends in, and a base that moved
/// with the generator order would move the search with it.
fn first_moved_point(permutation: &Permutation) -> Option<usize> {
    (0..permutation.width()).find(|&point| permutation.image(point) != Some(point))
}

/// Points separated by single spaces, which is the only shape [`StabiliserChain::render`] writes a list in.
fn join_points(points: &[usize]) -> String {
    points
        .iter()
        .map(usize::to_string)
        .collect::<Vec<String>>()
        .join(" ")
}
