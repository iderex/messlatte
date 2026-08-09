# The dipole table: the target's transition amplitude, its phase, and the sign

The authority for the format of a dipole table, for the phase convention a
table is converted into, and for what the loader does at the edge of the
tabulated range. `crates/formats/src/dipole.rs` is one implementation of this
document and not the other way round.

Issue #44 is where these choices are argued.

## Why the tables are data

A streaking measurement does not see the pulse. It sees the pulse multiplied by
the transition amplitude of whatever gas was in the interaction region, so a
reconstruction that treats that amplitude as flat charges the atom's spectral
phase to the pulse. That is #6, and it is why a case declares which dipole it
was made with.

The amplitudes and phases of real targets come from published calculations.
They are the one place somebody else's numbers would enter this repository, and
which targets and whose tables may be used is a maintainer decision that has not
been taken. Keeping them as data is what lets the loader, the format and the
flat case be finished before that decision is: a case names a table the way it
names a seed, and a table nobody has chosen yet is a file that is not there
rather than a code path that is missing.

## What the energy axis measures

The photoelectron's kinetic energy, not the photon energy. The forward model
evaluates the matrix element at the shifted momentum the electron leaves with,
so that is the quantity the table has to be indexed by. The ionisation potential
that separates the two belongs to the truth of a case (#36) and is not in this
file.

## The file

One JSON document, written in the canonical order and the canonical number
spelling `crates/formats/src/json.rs` fixes, so that identical content is
identical bytes and a hash over a table means something.

    {
      "amplitude": { "normalisation": <text>, "values": [ <number>, ... ] },
      "energy":    { "unit": <text>, "values": [ <number>, ... ] },
      "format":    { "name": "messlatte-dipole", "version": "1.0" },
      "phase":     { "convention": <text>, "values": [ <number>, ... ] },
      "source":    <text>,
      "target":    <text>
    }

`energy.unit` is one of the units the conversion layer admits for an energy in a
file, which is `J` or `eV`. Atomic units are not admitted in a file, for the
reason #13 gives: they belong inside the numerics and a file is what an operator
reads.

The energy column stays in that unit after the file has been read, and the
energy a caller asks about is converted into it once per lookup. That is a
departure from #13's rule that a quantity is converted at the edge and travels
in atomic units afterwards, and it is taken for one reason. A table is a file
somebody else wrote. Converting the column into atomic units and back is not the
identity on a double, so a loader holding the column in atomic units would write
out a table that is not the one it read, at the last bit, in the file a hash is
taken over. What the column measures does not leave the loader either way: an
amplitude and a phase carry no unit, and linear interpolation gives the same
answer on an axis scaled by any positive factor.

The three columns are the same length and there are at least two samples, so
there is an interval to interpolate in. The energies strictly increase.

`amplitude` is a modulus and is never negative. A real matrix element that
changes sign carries that sign as a phase of pi rather than as a negative
amplitude, because the two are the same number to a plot and different numbers
to everything that reads the phase.

`amplitude.normalisation` is the sentence saying what the amplitude is relative
to. Nothing in this repository turns it into a cross section, and #8 removes a
global amplitude scale from every case that does not declare an absolute photon
number, so what the column carries is a shape over energy and a statement about
where that shape came from.

`source` is where the numbers came from, given precisely enough to look up.
`target` is what they are the matrix element of. Both are refused when blank,
because a table whose source was never filled in is a table nobody can check.

`format.version` is `major.minor`. A major version this reader does not know is
refused rather than guessed at, and a higher minor version is read with the
fields this reader does not recognise ignored. The rule that shape comes from is
#41, applied here in the same narrow form the trace header applies it in.

## The phase convention, and the one thing that must not be got wrong

A complex matrix element can be written two ways, and both are in print:

    d(E) = |d(E)| exp(+i phase(E))
    d(E) = |d(E)| exp(-i phase(E))

The tabulated numbers are the same shape in both and their meanings are
opposite. A table carries which one its numbers are written in, in
`phase.convention`, spelled as one of

    "exp(+i phase)"
    "exp(-i phase)"

and the loader converts into this repository's convention once, at load. Nothing
downstream converts again, and a table in memory is always in this repository's
convention whatever the file it came from said. A table written back out is
written in this convention and says so, because a writer that re-emitted the
other spelling would be claiming to have preserved a meaning it has already
changed.

**This repository's convention is `exp(+i phase)`**, which is the sign that
makes the dipole phase enter the photoelectron amplitude the same way the phase
accumulated in the streaking field does. That accumulated phase is
`docs/format/streaking-field.md` and #43.

The residual there is worth stating rather than leaving to be discovered. The
sign with which the accumulated phase enters the amplitude is fixed by the
streaking operator, which is #42 and is open. If that operator turns out to be
written with the opposite sign, then the spelling above and the accumulated
phase's spelling move together, because the rule here is that the two enter with
the same sign. The relative sign between them is the only thing a table can be
wrong about, and it is fixed by this document today.

Why it is worth this much text. A sign error in a dipole phase looks exactly
like a chirp in the pulse. It moves no amplitude, it reddens no test that reads
a trace, and it changes the direction of the chirp every reconstruction reports.

## Interpolation, and what it assumes

Linear in energy, on the amplitude and on the tabulated phase separately,
between the two samples that bracket the asked-for energy. It is written as a
weighted sum of the two bracketing samples rather than as an offset from the
lower one, so an energy landing exactly on a sample carries weight one and
weight zero and returns that sample's own numbers rather than a value one
rounding away from them.

That is a statement about the interpolation and not about the whole route. An
energy that reached the lookup through the conversion layer may already be a
rounding away from the sample it was meant to be, and then the answer is a
rounding away too. The two are separate and only the first is fixed here.

Linear interpolation of a phase assumes the tabulated phase is continuous. A
table whose phase was wrapped into an interval of width two pi before it was
written interpolates wrongly across every wrap, and nothing here can see it: a
jump of nearly two pi between two samples is what a wrap looks like and is also
what a genuinely fast-varying phase looks like, so a refusal for it would delete
real tables. Tables are shipped unwrapped, and that is a convention this
document states rather than a rule anything refuses.

Linear rather than a spline. A spline through a table with a resonance in it
overshoots, and the overshoot is a feature of the interpolator that a reader
would read as a feature of the target. Where a table is too coarse for linear
interpolation, the repair is a finer table and not a smoother curve through the
one there is.

## The range, and why the edge refuses

Outside the tabulated range the loader refuses. It does not extrapolate, hold
the last value, or fall back to a flat dipole.

A tabulated amplitude falls off at the edges of its range because the
calculation stopped there, not because the target does, so every extrapolation
from the last two samples is a statement about physics that nobody made. A case
whose momentum window reaches beyond its table is a case that needs a wider
table, and the refusal is what says so instead of quietly producing a trace with
an invented tail.

## The worked example

Three samples, in the source's own convention rather than this one, so that the
conversion is visible in the numbers.

    target      worked-example
    unit        eV
    convention  exp(-i phase)

| energy (eV) | amplitude | phase as the source writes it | phase as this loads it |
| --- | --- | --- | --- |
| 20 | 1 | 0.25 | -0.25 |
| 30 | 2 | 0.75 | -0.75 |
| 40 | 1.5 | 0.5 | -0.5 |

Two interpolated points, both at the midpoint of an interval, so a reader can
check them with arithmetic:

| energy (eV) | amplitude | phase |
| --- | --- | --- |
| 25 | 1.5 | -0.5 |
| 35 | 1.75 | -0.625 |

Every number above is exact in binary, so what the interpolation costs at these
points is the rounding of the energies into hartree and nothing else.

The sign is checkable without any of the decimals. Between 20 and 30 eV the
loaded phase falls, so this target advances the photoelectron rather than
delaying it: the group delay is the derivative of the phase with respect to
energy and it is negative there. Read in the source's own spelling, without the
conversion, the same three numbers say the opposite. That is the whole content
of the convention rule and it is one sign.

## The table that ships

`data/dipole/flat.json`, target `flat`, amplitude one and phase zero from 0 to
1000 eV.

It is a table like any other. The flat-dipole world is the world most of these
methods were designed in, and it is reached by naming this file rather than by a
branch in the generator, so there is no code path that exists only for it and no
second place for it to disagree with itself.

Its source is a definition rather than a measurement, and it says so in the
field where a measured table names its publication. Two samples are enough: the
interpolation is linear, so two equal values reproduce the constant exactly
everywhere between them.

Its range is a choice and not a measurement. It covers the photoelectron
energies the extreme-ultraviolet and soft-X-ray cases this board is built for
land in, and a case that needs more energy than that needs a table that says so
rather than an edge that stretches.

## What this document does not fix

Which real targets are used and whose tables they come from, which is entry 6 of
#34 and is a maintainer decision with a legal half. How a case names the table it
was made with, which is the case declaration, #37. How the truth of a case
records the dipole it used, which is #36. The streaking operator that evaluates
this matrix element at the shifted momentum, which is #42.
