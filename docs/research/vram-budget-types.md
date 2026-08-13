# VRAM budget types and CPU `f64` fallback

## Scope

This slice delivers the first executable ADR 0006 contract in `compute_backend`:

1. classify devices into the accepted 4/6/8/12/24-GiB profiles;
2. reserve one eighth of profile capacity as unused safety memory;
3. predict peak bytes as `batch × bytes_per_observation + working_set`;
4. autotune the micro-batch by successive halving until the peak fits usable VRAM;
5. treat out-of-memory as an expected operating state with a bounded retry budget, then fall back to the CPU `f64` reference without dropping observations;
6. refuse full-corpus document-by-topic device tensors and refuse dropping observations, shrinking topic/model complexity, or moving a knowledge cutoff to fit memory;
7. keep mixed precision out of final diagnostic quantities;
8. keep raw source text out of allocation telemetry.

Live CUDA/WGPU kernels, mixed-precision device lanes, and hardware CPU/GPU parity remain accepted-target. This slice does not claim an accelerator.

## Authoritative sources

IEEE. (2019). *IEEE standard for floating-point arithmetic* (IEEE Std 754-2019). https://doi.org/10.1109/IEEESTD.2019.8766229

Micikevicius, P., Narang, S., Alben, J., Diamos, G., Elsen, E., Garcia, D., Ginsburg, B., Houston, M., Kuchaiev, O., Venkatesh, G., & Wu, H. (2018). Mixed precision training. In *International Conference on Learning Representations*. https://openreview.net/forum?id=r1gs9JgRZ

NVIDIA Corporation. (2024). *CUDA C++ programming guide*. https://docs.nvidia.com/cuda/cuda-c-programming-guide/

Rhu, M., Gimelshein, N., Clemons, J., Zulfiqar, A., & Keckler, S. W. (2016). vDNN: Virtualized deep neural networks for scalable, memory-efficient neural network design. In *2016 49th Annual IEEE/ACM International Symposium on Microarchitecture (MICRO)* (pp. 1–13). IEEE. https://doi.org/10.1109/MICRO.2016.7783721

## Formula notes

- **Profile capacity** is \(p \times 2^{30}\) bytes for \(p \in \{4,6,8,12,24\}\).
- **Safety reserve** is \(p \times 2^{30} / 8\). Usable VRAM is \(\max(0, a - s)\) for available bytes \(a\) and reserve \(s\).
- **Peak** is \(b \cdot c + w\) for batch \(b\), per-observation charge \(c\), and working set \(w\). Overflow fails closed.
- **CPU `f64` reference** is the streamed weighted sum \(\sum_i w_i x_i\) in IEEE 754 binary64 (IEEE, 2019).
- **RMSE** is computed from recovered versus known totals; tests do not hard-code expected recovery numbers.
- Mixed precision may be recorded as a transient mode only; final diagnostics remain binary64 (Micikevicius et al., 2018). Full-corpus responsibility tensors are refused rather than virtualized onto the device (Rhu et al., 2016).

## Verification

- noiseless CPU `f64` weighted sums recover a known total with machine-scale computed RMSE;
- 24-GiB profiles admit a larger autotuned micro-batch than 4-GiB profiles for the same workload;
- bounded OOM retries fall back to CPU while preserving the planned observation batch;
- full-corpus, observation-drop, complexity-reduction, cutoff-mutation, mixed-final, and source-text telemetry paths fail closed.
