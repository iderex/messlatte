# messlatte

The reconstruction landscape is splintered across FROG-CRAB, PROOF, VTGPA and several ptychographic variants, differing in accuracy, robustness and cost, many resting on the central momentum approximation. Validation is the real problem: every method is iterative, stopped when the figure of merit stops moving, and proved by computing the spectrogram back and comparing it visually with the measured one, which is defensible for narrowband pulses and not for broadband ones. A record value that defines the discipline is credentialed by eye on a plot and carries no error bars. This board provides simulated streaking and RABBITT traces with known ground truth across bandwidth, noise, CEP jitter, delay jitter and incomplete sampling, clean reference implementations, and an evaluation of which method distorts under which conditions. The central change is many random starts and the scatter of solutions as the output rather than a point estimate.

Planning happens on the issue tracker first. Every decision that shapes
the architecture is written down there with its reasons before the code
that depends on it exists.

See [NOTICE.md](NOTICE.md) for the intended-use notice.
