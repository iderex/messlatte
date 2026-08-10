# The submission file

A submission is what a method hands back: the object it retrieved and the
record of how it got there. This document is what somebody writing or reading
one outside this workspace needs. It is the authority for the format; the code
in `crates/formats` is one implementation of it, and where the two disagree
that is a defect in one of them and an issue rather than a thing to work
around.

The decision this format follows from, with its reasons, is issue #38.

## One file per start

    <case>/submission.json

A start writes its own file. An ensemble of starts is that many files, and the
file that sits over them is issue #71 and is not this one. Two consequences are
the reason for it rather than a side effect: one failed start does not
invalidate the others, and an author outside this repository can submit starts
as they finish instead of holding them until the last one lands.

The name above is a convention of the case directory rather than part of the
format. What a case directory holds is issues #37 and #39.

## The document

JSON, UTF-8, one object, in the same canonical form the trace header uses:

- Keys are written in byte order.
- A number is the shortest decimal that reads back as the same double, written
  without an exponent.

A full submission, for a retrieval on three time samples:

    {"case":"fixture-01","format":{"name":"messlatte-submission","version":"1.0"},"knowns":["delay-axis","ionisation-potential"],"retrieved":{"imaginary":[0,0.5,-0.25],"quantity":"field","real":[0.25,1,0.25],"time":{"unit":"as","values":[-100,-50,50]}},"seed":"0123456789abcdef","start":"start-000","stopping":{"merit":0.0125,"rule":{"form":"fixed-count","iterations":200},"stopped-at":200,"why":"the declared iteration count was reached"}}

The fields:

| field | what it carries |
| --- | --- |
| `case` | The identifier of the case this submission is about. It is the case directory's name. |
| `format.name` | `messlatte-submission`. |
| `format.version` | `major.minor`. |
| `knowns` | The declared knowns the method actually read, as names. |
| `retrieved.quantity` | `field` or `spectrum`. |
| `retrieved.time` | The time grid, when the quantity is `field`. Unit one of `s`, `fs`, `as`. |
| `retrieved.real` | The real part of the retrieved field, one value per time sample. |
| `retrieved.imaginary` | The imaginary part, one value per time sample. |
| `retrieved.energy` | The photon-energy grid, when the quantity is `spectrum`. Unit `J` or `eV`. |
| `retrieved.amplitude` | The spectral amplitude, one value per energy sample, never below zero. |
| `retrieved.phase` | The spectral phase in radians, one value per energy sample. |
| `seed` | The seed of this start, as sixteen lowercase hexadecimal digits. |
| `start` | The identifier of the start within the run. |
| `stopping.rule` | The rule the run was given. See below. |
| `stopping.stopped-at` | The iteration the method stopped at. |
| `stopping.merit` | The method's own figure of merit at that iteration. |
| `stopping.why` | Why it stopped, in the author's words. |
| `streaking` | Optional. The retrieved streaking field. See below. |

The seed is a string and not a number because it is sixty-four bits and this
document's numbers are doubles. A seed written as a number would read back as a
neighbour of itself, which reproduces a different start, and the whole reason
the seed is in the file is that it reproduces this one. Fixed width and lower
case, so two submissions with the same seed are the same bytes.

## What is not in it

Nothing about how well the method did. That is the scorer's output, computed
from this file and the truth, and a submission carrying its own error would be
a claim the scorer would then have to either believe or contradict. There is no
field for it, which settles the question rather than trusting anybody to leave
it empty.

## The two domains, and the conversion between them

A method may submit the complex field on a time grid or the spectral amplitude
and phase on a photon-energy grid. Which one is a property of the method and
not of the case: the methods argue in different domains, and requiring one
would make some of them carry a transform they did not choose and did not
validate.

A document carries one of the two and never both. Two representations of one
answer are two answers with nothing to say which was meant.

The conversion is defined here, once, so that a scorer reads either the same
way:

    E(t)  = the complex field, `real` + i * `imaginary`
    S(w)  = `amplitude` * exp(i * `phase`)
    w     = e / hbar, for a photon energy e on the energy grid

    S(w)  = integral over t of E(t) * exp(+i * w * t) dt
    E(t)  = (1 / 2 pi) * integral over w of S(w) * exp(-i * w * t) dw

The sign is the one the generator's own fixture uses, where a pulse built from
spectral lines is `sum over lines of exp(i * phase) * exp(-i * energy * t)`:

    git grep -n 'exp(i \* phase) \* exp(-i \* energy' -- crates/generator/tests/limits.rs

That helper is a test fixture rather than a document, so it is where the
convention was first written down and not an authority for it. This section is
the authority. A method that submits in one domain and a scorer that reads in
the other agree because both use these two lines, and nothing else in the
scored path is allowed a second convention.

What this repository does not yet do with them is worth being plain about: no
code in this workspace performs this transform. The measures that will are
milestone 07. Until then the conversion is a definition a reader can implement
and not one this tree executes.

## The stopping rule

A method declares its stopping rule as a parameter of the run, and the
submission records it beside where the run actually stopped. The reason is
issue #12: these methods are stopped when a figure of merit stops moving, by
somebody watching it, and that person's patience becomes part of the published
result and is recorded nowhere. Two runs of one method with different patience
are two methods for benchmark purposes.

Three forms, and no others:

| `form` | members | what it means |
| --- | --- | --- |
| `fixed-count` | `iterations` | Stop after this many iterations, whatever the figure of merit does. |
| `relative-change` | `threshold`, `window` | Stop when the relative change in the figure of merit stays below `threshold` for `window` consecutive iterations. |
| `wall-clock` | `seconds` | Stop after this many seconds. |

The fixed count is the default for the case matrix, because it is the only form
that makes the cost comparison mean anything and the only one that cannot be
tuned per case without leaving a trace in this file. The wall-clock form is
admitted for the cost comparison and not for a scored run: the same rule on two
machines is two different methods.

A count is a whole number a double holds exactly. `200.5` iterations, a
negative count and a count above nine thousand million million are each
refused, the last because a number beyond that reads back as a neighbour of
itself.

## The retrieved streaking field

Optional, and present only where the method retrieves one.

    "streaking":{"time":{"unit":"as","values":[-100,-50,50]},"unit":"kg m/s","values":[-0.5,0.25,1.5]}

It is written as the momentum the field imparts rather than as a vector
potential in atomic units. Atomic units belong inside the numerics and a file is
what an operator reads, which is issue #13, and the momentum is the SI-spellable
quantity: in atomic units the vector potential of
[streaking-field.md](streaking-field.md) and that momentum are the same number,
so this is a spelling of that quantity and not a different one. The sign
convention is that document's.

## What a reader may assume

**A grid is strictly increasing.** A repeated sample is refused with a
decreasing one: two values at one position are two answers to one question with
nothing to say which is meant.

**A grid is not necessarily uniform.** A method may return the support it is
confident about, so a gap is a submission rather than a fault. A reader that
takes the step from the first two samples and uses it for the rest is wrong
about every such submission.

**Every column has one value per grid sample.** A file where a column and its
grid disagree is refused rather than trimmed to whichever is shorter.

**Nothing is a sentinel.** A sample the method did not retrieve is absent from
the grid rather than written as a value, and a value that is not finite is
refused by the reader and by the writer, so a file cannot carry one at all.

**A spectral amplitude is a modulus.** It is never below zero, and the sign
belongs in the phase. A negative phase is ordinary and is not refused.

**A submission without a stopping record is not scored.** The absence is
refused by name rather than reported as a missing member, because a method that
never recorded where it stopped and one that mistyped a field are the same file
to a reader and different mistakes to its author.

## The two things a document cannot decide

Two refusals are about the fit between a submission and the case being scored,
and neither can be read out of the submission alone.

**A declared known the case did not offer.** What a case offers is declared by
the case, which is issue #7, and a submission cannot be trusted to state it:
that is exactly the field a method that read too much would fill in wrongly. So
the scorer supplies the case's list and the comparison is made against it. A
method that read less than it was offered is not refused; a method that names
something the case did not declare was not solving that case.

**A case identifier that is not the one being scored.** A submission copied
from the neighbouring case directory and rerun scores one case against
another's truth, and the identifier is the only thing in the file that says so.

The case declaration that will supply both is issue #37 and does not exist yet.
Until it does, a caller states the two directly, and the refusals are proved
against that rather than against a file.

## Versions

The rule for reading a version is issue #41, applied here as it is for the
trace:

- A major version this reader does not know is refused rather than guessed at.
- A higher minor version is accepted, with members the reader does not
  recognise ignored rather than preserved.
- A writer never adds a member that changes the meaning of an existing one.

## What is not settled here

The validator's two case-dependent refusals are exercised against a stated list
rather than against a case declaration, for the reason above.

The scorer that reads a submission through this file, and the rule that a
reference implementation's output reaches the scorer only by this path, are the
last condition of issue #38 and need a method and a scorer. Neither exists, so
that condition is unmet rather than met by this document.
