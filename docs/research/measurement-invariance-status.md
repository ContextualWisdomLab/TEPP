# Measurement invariance status (doctoring)

## Scope

`measurement_invariance` records an explicit configural, metric, or scalar
status. Configural structure alone cannot license shared metric meaning.
Recovered loadings are scored with computed RMSE against known truth only after
each estimate is bound to the same group × indicator × factor coordinate as its
truth parameter; sequence position alone is not parameter identity.

This slice does not fit ESEM/DSEM, solve rotational alignment, claim partial
invariance, or treat a language profile as aligned from architecture alone.

## Authority

### Normative TEPP contract

- `docs/adr/0004-shared-multilingual-latent-space.md` — equivalent meanings
  must be aligned and tested for measurement invariance.
- `docs/adr/0005-posterior-esem-dsem.md` — longitudinal invariance is a
  psychometric requirement, not an implicit property of a shared space.

### Supporting literature

Meredith (1993) distinguishes configural, weak/metric, and strong/scalar
invariance. Marsh et al. (2014) place those tests inside an ESEM program.
They do **not** authorize treating configural similarity as shared scores.
The explicit group × indicator × factor key in TEPP is an implementation
safeguard: a recovery statistic is meaningful only when the estimated loading
and known-truth loading refer to the same identified parameter. It does not
resolve ESEM factor rotation or establish invariance by itself.

Meredith, W. (1993). Measurement invariance, factor analysis and factorial
invariance. *Psychometrika, 58*(4), 525–543.
https://doi.org/10.1007/BF02294825

Marsh, H. W., Morin, A. J. S., Parker, P. D., & Kaur, G. (2014). Exploratory
structural equation modeling: An integration of the best features of
exploratory and confirmatory factor analysis. *Annual Review of Clinical
Psychology, 10*, 85–110. https://doi.org/10.1146/annurev-clinpsy-032813-153700
