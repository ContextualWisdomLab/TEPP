### Scientific validation

- Add an explicit affine EventTime-basis transform for prevalence intercept/slope recovery. Validation now has a fail-closed path to compare physically identical linear prevalence trajectories across the simulator's declared time coordinate and each rolling-origin training window's frozen centered/scaled EventTime basis without mistaking feature reparameterization for coefficient bias. The transform is validation arithmetic only and does not authenticate EventTime, availability, or source provenance.
