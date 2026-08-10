# messlatte

The reconstruction landscape is splintered across FROG-CRAB, PROOF, VTGPA and several ptychographic variants, differing in accuracy, robustness and cost, many resting on the central momentum approximation. Validation is the real problem: every method is iterative, stopped when the figure of merit stops moving, and proved by computing the spectrogram back and comparing it visually with the measured one, which is defensible for narrowband pulses and not for broadband ones. A record value that defines the discipline is credentialed by eye on a plot and carries no error bars. This board provides simulated streaking and RABBITT traces with known ground truth across bandwidth, noise, CEP jitter, delay jitter and incomplete sampling, clean reference implementations, and an evaluation of which method distorts under which conditions. The central change is many random starts and the scatter of solutions as the output rather than a point estimate.

Planning happens on the issue tracker first. Every decision that shapes
the architecture is written down there with its reasons before the code
that depends on it exists.

See [NOTICE.md](NOTICE.md) for the intended-use notice.

## License

The code is AGPL-3.0, decided by the maintainer on 2026-08-08. The documents
are CC-BY-4.0, decided on 2026-08-09, and the generated case archive is
CC0-1.0, decided on the same day. Which file is which is written in
[NOTICE.md](NOTICE.md) and not repeated here.

The full text of the code license is in [LICENSE](LICENSE). Read that file
rather than this line, and if you want the platform's own reading of it, run:

    gh api repos/iderex/messlatte --jq '.license.spdx_id'

That reading names the family and not the option. The notice this project
attached to itself grants version 3 or later, which is why the manifests say
`AGPL-3.0-or-later`.
