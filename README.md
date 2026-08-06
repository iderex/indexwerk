# indexwerk

The standard for tensor computer algebra is xAct, free software that requires a proprietary closed-source product. The measured distance is a factor of nine, 270 ms against 2400 ms for the Einstein tensor of the FLRW metric, and the Mathematica side parallelises across kernels where Python is held by the global interpreter lock. The bottleneck is one sharply defined problem, canonicalisation of index expressions under permutation symmetry groups, which is pure combinatorics on groups and flies on sixteen cores in a compiled language. Cadabra already exists as a good standalone open solution, so the gap is not the core alone but the application layer above it: xPert, xPand, Invar and Harmonics have no open counterpart, and those are the packages perturbation theory needs.

Planning happens on the issue tracker first. Every decision that shapes
the architecture is written down there with its reasons before the code
that depends on it exists.

See [NOTICE.md](NOTICE.md) for the intended-use notice.
