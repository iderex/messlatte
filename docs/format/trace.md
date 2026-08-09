# The trace file

A trace is two files: an array of values and a header saying what they are.
This document is what somebody writing or reading one outside this workspace
needs. It is the authority for the format; the code in `crates/formats` is one
implementation of it, and where the two disagree that is a defect in one of
them and an issue rather than a thing to work around.

The decision this format follows from, with its reasons, is issue #35.

## The two files

    <case>/trace.npy    the values
    <case>/trace.json   the header

Those names are a convention of the case directory rather than part of the
format. What a case directory holds and how it is laid out is issues #37 and
#39, and until they land the names above are what this repository writes.

## The array

NPY, version 1.0, one two-dimensional array, dtype `<f8`, C order, no
compression.

The rows are the electron axis and the columns are the delay axis, so one row
is one electron-axis position across every delay. A reader takes that from this
document rather than from the shape, because a square trace would let it guess
wrong in silence.

Three things are refused rather than converted:

- Fortran order. Transposing on the way in would mean the axes a reader sees
  are decided by whoever wrote the file rather than by this document.
- Any other dtype, including a wider one. A value this format cannot hold
  exactly is one it should not round silently.
- Versions 2.0 and 3.0. They widen the header-length field and admit a
  non-ASCII header, and a header of three keys needs neither.

Why NPY and not HDF5, which is what the field exchanges: HDF5 costs a C
library, a container layout that is not byte-deterministic for identical
content, and a reader nobody here can audit line by line. That is a poor trade
for one two-dimensional array of doubles. A converter is a small optional tool
and lives outside the scored path.

The bytes are exactly determined by the content, which is what lets a case
index hash a trace. This repository's writer produces the same bytes as the
reference implementation for the same array, padding included.

## The header

JSON, UTF-8, one object. Canonical form, which means two things:

- Keys are written in byte order.
- A number is written as the shortest decimal that reads back as the same
  double, without an exponent. `3e-7` is a valid number to read and is written
  back as `0.0000003`. A document written elsewhere is normalised into this
  form rather than echoed, so what is hashed is what the document means.

A full header, for a trace of two electron samples across three delays:

    {"case":"fixture-01","cells":{"quantity":"counts"},"delay":{"unit":"fs","values":[-1,-0.5,1.5]},"electron":{"quantity":"energy","unit":"eV","values":[20,20.5]},"format":{"name":"messlatte-trace","version":"1.0"}}

The fields:

| field | what it carries |
| --- | --- |
| `case` | The identifier of the case this trace belongs to. It is the join key of the case index. |
| `cells.quantity` | `counts` or `normalised-counts`. |
| `cells.normalisation` | Present exactly when the quantity is `normalised-counts`, and says what the counts were divided by. |
| `delay.unit` | One of `s`, `fs`, `as`. |
| `delay.values` | The delays, one per column of the array. |
| `electron.quantity` | `momentum` or `energy`. |
| `electron.unit` | `kg m/s` for momentum; `J` or `eV` for energy. |
| `electron.values` | The electron-axis samples, one per row of the array. |
| `format.name` | `messlatte-trace`. |
| `format.version` | `major.minor`. |

The admitted units are a closed set rather than free text, so that the
conversion layer has a finite thing to cover. Every one of them is SI, with the
electronvolt the exception SI itself makes by listing it as accepted for use
with the SI. Atomic units are not admitted in a file: they belong inside the
numerics, and a file is what an operator reads. That boundary is issue #13.

## What a reader may assume

This is the part to read before using a trace, because each clause is a mistake
that is otherwise made quietly.

**Both axes are strictly increasing.** A repeated value is refused with a
decreasing one: two cells at one axis position are two measurements of one
thing and nothing here says which is meant.

**The delay axis is not necessarily uniform.** Sampling completeness is one of
the axes this benchmark varies, so a gap in the delays is a case rather than a
defect. A reader that takes the step from the first two samples and uses it for
the rest is wrong about every case with a gap in it. A method that needs a
uniform delay axis declares that as a requirement and is scored on the cases
that meet it.

**Cells are counts, or a stated normalisation of counts, and never an arbitrary
scale.** A normalised trace carries the sentence saying what it was divided by.
A trace of raw counts carries no such sentence, and a file carrying both is
refused rather than resolved.

**Missing samples are absent from the axis rather than written as a value.**
There is no sentinel in this format to mistake for data, and that is enforced
from the other end: a cell that is not finite is refused by the reader and by
the writer, so a file cannot carry one at all. A run with gaps has a shorter
delay axis, not a full one with holes punched in it.

**The array's shape is the axes' lengths.** A file where they disagree is
refused rather than trimmed to whichever is smaller.

## Versions

Every file this project writes carries a format version, and the rule for
reading one is issue #41. Applied here:

- A major version this reader does not know is refused rather than guessed at.
  A major version is a change of meaning, and guessing which fields still mean
  what they did is how a plausible score gets computed from a misread file.
- A higher minor version is accepted, with fields the reader does not recognise
  ignored. Ignored rather than preserved: a trace read from a later minor
  version and written again is a version 1.0 trace and says so, because a
  writer that re-emitted a field it does not understand would be claiming to
  have kept its meaning.
- A writer never adds a field that changes the meaning of an existing one. That
  is a major change wearing a minor number.

A version change owes a line in the changelog, a note in this document, and a
fixture of the old version kept in the tree with a test that reads it, so that
a compatibility claim is executed rather than asserted.

## What is not settled here

The opt-in check that reads a trace with a numerical library outside this
workspace is the fourth condition of #35 and is not in the tree. It needs a
tracked fixture case, which is #32, and a home the workspace layout admits: the
formats crate may depend on `messlatte-units` and on nothing else, so a case
that reaches the suite machinery cannot live in it.
