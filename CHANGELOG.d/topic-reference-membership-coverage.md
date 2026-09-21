### Fixed

- The CPU `f64` TRSL-TM reference input now fails closed when any modeled document lacks an active membership at that document's event time. Previously, membership validation only required at least one active assignment somewhere in the corpus, allowing an unmatched document to enter the prevalence design with an all-zero membership row and weakening cross-classified/multiple-membership interpretation.
