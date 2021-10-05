# borscht

Implementation of the [BIRCH algorithm](https://en.wikipedia.org/wiki/BIRCH)
([original paper](https://dl.acm.org/doi/10.1145/235968.233324)), a fast (single
database scan) constrained-memory hierarchical clustering algorithm ideal for very large data
sets that cannot be held in memory.

Includes modifications to the original BIRCH algorithm; currently only
[BETULA](https://arxiv.org/abs/2006.12881), but with more planned.