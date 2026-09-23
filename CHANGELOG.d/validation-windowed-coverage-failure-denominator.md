### Fixed

- Keep the unconditional attempted-DGP denominator beside rolling-origin interval-coverage evidence. Successful window coverage is collapsed within each DGP before Monte Carlo inference, while numerical DGP failures remain visible through attempted/success/failure counts and failure-rate MCSE. All-failed and singleton-success experiments no longer require fabricating coverage uncertainty. Structural experiment invalidity remains outside the failure denominator. (#724)
