# CDCL SAT solver from scratch

SAT solvers are [incredibly useful](https://cacm.acm.org/research/the-silent-revolution-of-sat/) yet under-appreciated
pieces of technology. Outside computer science, not many people know what SAT solvers are, let alone how they work.
This is a shame as the key ideas underpinning SAT solvers are actually quite simple and widely applicable!

The goal of this repo is to demystify SAT solvers by building one from scratch, one step at a time.
Specifically, we implement a minimal version of
the [conflict-driven clause learning (CDCL)](https://en.wikipedia.org/wiki/Conflict-driven_clause_learning)
algorithm which powers most modern SAT solvers. The implementation captures all the key ideas behind CDCL (unit
propagation, conflict analysis, clause learning and backjumping) using only ~300 lines of Rust (excluding debug
code).

The implementation has been optimized for conceptual understanding over performance and does not contain various tricks
and heuristics that are important in practice (like clause forgetting, VSIDS and 2WL). Furthermore, its test coverage
is limited. If you're looking for a production-grade SAT solver, see
the [SAT Competition](https://satcompetition.github.io/) and other resources.
