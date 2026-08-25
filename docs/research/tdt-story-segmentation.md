# TDT story-segmentation calibration

## Scope

This note doctors the `event_core` gate that keeps TDT story/event segmentation distinct from event-instance promotion and state-transition authority:

1. an interior story cut is detection evidence, not a promoted instance or transition;
2. `WindowDiff` and `Pk` are computed from known-truth unit partitions;
3. boundary precision/recall and calibrated cut probabilities recover known-truth cuts with lower RMSE than an always-cut detector.

No database migration is allocated. A later TDT linker or tracker may consume these scores as measurement evidence only.

## Authoritative sources

Allan, J. (Ed.). (2002). *Topic detection and tracking: Event-based information organization*. Kluwer Academic Publishers.

Beeferman, D., Berger, A., & Lafferty, J. (1999). Statistical models for text segmentation. *Machine Learning, 34*(1–3), 177–210. https://doi.org/10.1023/A:1007506220214

Pevzner, L., & Hearst, M. A. (2002). A critique and improvement of an evaluation metric for text segmentation. *Computational Linguistics, 28*(1), 19–36. https://doi.org/10.1162/089120102317341756

## Application

Allan (2002) treats story segmentation as a TDT measurement task over ordered units. Beeferman et al. (1999) score probe pairs at a fixed distance (`Pk`), and Pevzner and Hearst (2002) count mismatched window boundary totals (`WindowDiff`) so near-miss and over-segmentation errors remain visible. TEPP therefore refuses to cast a detected story partition as an event instance or a forward state transition and requires computed `WindowDiff`, `Pk`, precision, recall, and RMSE against known truth (Allan, 2002; Beeferman et al., 1999; Pevzner & Hearst, 2002).

## Verification

- `refuse_story_segmentation_as_instance` always returns `StorySegmentationIsNotEventInstance`;
- `refuse_story_segmentation_as_transition` always returns `StorySegmentationIsNotStateTransition`;
- `StorySegmentation::new` refuses fewer than two units and mismatched boundary lengths;
- `story_window_diff` and `story_pk` fail closed on empty, misaligned, or oversized windows;
- `story_boundary_precision` and `story_boundary_recall` fail closed on empty recovered or truth boundary sets;
- computed RMSE of known boundary targets is lower under calibrated probabilities than under an always-cut detector.
