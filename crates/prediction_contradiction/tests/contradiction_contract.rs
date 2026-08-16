//! Contradicting predictions cannot be promoted to observed fact.

use prediction_contradiction::{
    ClosedEventInterval, PredictionContradictionError, contradiction_recovery_rate,
    intervals_contradict, refuse_promotion_when_contradict,
};

fn interval(start: i64, end: i64) -> ClosedEventInterval {
    ClosedEventInterval::new(start, end).expect("interval")
}

#[test]
fn disjoint_prediction_cannot_become_observed_fact() {
    let predicted = interval(0, 10);
    let observed = interval(20, 30);
    assert!(intervals_contradict(predicted, observed).expect("compare"));
    assert_eq!(
        refuse_promotion_when_contradict(predicted, observed),
        Err(PredictionContradictionError::PredictionContradictsObservation)
    );
    let overlapping = interval(5, 15);
    refuse_promotion_when_contradict(predicted, overlapping).expect("consistent");
    assert!(!intervals_contradict(predicted, overlapping).expect("overlap"));
}

#[test]
fn recovered_contradictions_match_known_truth_better_than_promoting_all() {
    let pairs = [
        (interval(0, 10), interval(20, 30)),
        (interval(0, 10), interval(5, 15)),
        (interval(40, 50), interval(0, 10)),
    ];
    let truth = [true, false, true];
    let recovered = [
        intervals_contradict(pairs[0].0, pairs[0].1).expect("p0"),
        intervals_contradict(pairs[1].0, pairs[1].1).expect("p1"),
        intervals_contradict(pairs[2].0, pairs[2].1).expect("p2"),
    ];
    let collapsed = [false, false, false];
    let recovered_rate = contradiction_recovery_rate(&truth, &recovered).expect("recovered");
    let collapsed_rate = contradiction_recovery_rate(&truth, &collapsed).expect("collapsed");
    let expected = {
        let mut matches = 0_u32;
        for (truth_flag, decided_flag) in truth.iter().zip(recovered.iter()) {
            if truth_flag == decided_flag {
                matches += 1;
            }
        }
        f64::from(matches) / f64::from(u32::try_from(truth.len()).expect("len"))
    };
    assert!((recovered_rate - expected).abs() < f64::EPSILON);
    assert!(recovered_rate > collapsed_rate);
}

#[test]
fn empty_or_invalid_interval_payloads_fail_closed() {
    assert_eq!(
        ClosedEventInterval::new(10, 10),
        Err(PredictionContradictionError::InvalidIntervalPayload)
    );
    assert_eq!(
        ClosedEventInterval::new(10, 9),
        Err(PredictionContradictionError::InvalidIntervalPayload)
    );
    assert_eq!(
        contradiction_recovery_rate(&[], &[]),
        Err(PredictionContradictionError::InvalidIntervalPayload)
    );
    assert_eq!(
        contradiction_recovery_rate(&[true], &[]),
        Err(PredictionContradictionError::InvalidIntervalPayload)
    );
    assert_eq!(
        contradiction_recovery_rate(&[true, false], &[true]),
        Err(PredictionContradictionError::InvalidIntervalPayload)
    );
}
