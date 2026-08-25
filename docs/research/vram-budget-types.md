# VRAM budget types, executable OOM retries, and CPU `f64` reference

## Scope

This slice delivers the first executable ADR 0006 contract in `compute_backend`:

1. classify devices into the accepted 4/6/8/12/24-GiB profiles;
2. reserve one eighth of profile capacity as unused safety memory;
3. predict peak bytes as `batch × bytes_per_observation + working_set`;
4. autotune the micro-batch by successive halving until the predicted peak fits usable VRAM;
5. after each observed OOM, emit a smaller executable GPU plan with an incremented retry count, then fall back to the CPU `f64` reference after the bounded retry budget or a failed unit batch;
6. refuse full-corpus document-by-topic device tensors and refuse dropping observations, shrinking topic/model complexity, or moving a knowledge cutoff to fit memory;
7. keep mixed precision out of final diagnostic quantities and compare CPU/candidate outputs with a normalized parity tolerance;
8. keep raw source text out of allocation telemetry;
9. use compensated deterministic summation for the sequential CPU `f64` numerical reference.

Live CUDA/WGPU kernels, deterministic fixed-pool CPU multithreading, mixed-precision device lanes, and hardware CPU/GPU parity remain accepted-target. This slice does not claim an accelerator or a multithreaded production estimator.

## Authoritative sources

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://standards.ieee.org/ieee/754/6210/

Micikevicius, P., Narang, S., Alben, J., Diamos, G., Elsen, E., Garcia, D., Ginsburg, B., Houston, M., Kuchaiev, O., Venkatesh, G., & Wu, H. (2018). Mixed precision training. In *International Conference on Learning Representations*. https://openreview.net/forum?id=r1gs9JgRZ

NVIDIA Corporation. (2024). *CUDA C++ programming guide*. https://docs.nvidia.com/cuda/cuda-c-programming-guide/

Ogita, T., Rump, S. M., & Oishi, S. (2005). Accurate sum and dot product. *SIAM Journal on Scientific Computing, 26*(6), 1955–1988. https://doi.org/10.1137/030601818

Rhu, M., Gimelshein, N., Clemons, J., Zulfiqar, A., & Keckler, S. W. (2016). vDNN: Virtualized deep neural networks for scalable, memory-efficient neural network design. In *2016 49th Annual IEEE/ACM International Symposium on Microarchitecture (MICRO)* (pp. 1–13). IEEE. https://doi.org/10.1109/MICRO.2016.7783721

## Formula notes

- **Profile capacity** is \(p \times 2^{30}\) bytes for \(p \in \{4,6,8,12,24\}\).
- **Safety reserve** is \(p \times 2^{30} / 8\). Usable VRAM is \(\max(0, a - s)\) for available bytes \(a\) and reserve \(s\).
- **Peak** is \(b \cdot c + w\) for batch \(b\), per-observation charge \(c\), and working set \(w\). Overflow fails closed.
- **OOM retry** is stateful: retry count \(r\) increments after each observed OOM, batch is halved when \(r\leq r_{max}\), and the peak is recomputed from the original workload. No loop is counted as a retry unless an executable plan is returned to the caller.
- **CPU `f64` reference** uses deterministic compensated summation in IEEE 754 binary64 so cancellation-heavy low-order terms are not needlessly discarded (IEEE, 2019; Ogita et al., 2005).
- Streamed document/topic cardinalities are not multiplied into a hypothetical full-corpus allocation; the forbidden full-corpus policy is rejected by the controller.
- Mixed precision may be recorded as a transient mode only; final diagnostics remain binary64 (Micikevicius et al., 2018).

## Verification

- cancellation-heavy CPU `f64` weighted sums recover the low-order term and known totals with computed RMSE;
- 24-GiB profiles admit a larger autotuned micro-batch than 4-GiB profiles for the same workload;
- each accepted OOM retry returns a smaller GPU plan and an exact retry count before CPU fallback;
- streamed extreme cardinalities remain valid because no full tensor is sized;
- negative parity tolerances, full-corpus placement, observation drop, complexity reduction, cutoff mutation, mixed-final precision, and source-text telemetry fail closed.
