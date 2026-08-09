# Security

## What this software is, before what counts as a vulnerability in it

A benchmark that runs on one machine. It reads files, computes, and writes
files. It opens no socket, listens on no port, authenticates nobody and stores
no credential. There is no server to attack and no session to hijack, so most
of what a security policy usually covers has no subject here.

That makes the list below short, and short is the honest length. A template
listing account takeover and privilege escalation for a program with neither
would say nothing about this repository while looking thorough.

## What counts

**A file this repository reads can make it do something other than read.** The
readers are the real attack surface. A trace, a case declaration and a
submission all arrive from outside: an outside author writes a submission, and a
case archive is something an operator downloads. A malformed one that causes
memory unsafety, an allocation sized from a number in the file, a read outside
the buffer, or a loop that does not terminate is a vulnerability and not a
parsing bug. So is a path taken from a file and used to write outside the
directory the operator named.

**A crafted input that makes the scorer report a number it did not compute.**
The output of this project is a measurement somebody else will cite. An input
that makes a score come out wrong while the run looks green is an integrity
failure, and for a benchmark that is the worst case rather than a lesser one.
Report it here rather than as an ordinary defect.

**Anything in the build or release path that lets somebody else's code in.** A
workflow that evaluates a value taken from a pull request, an action whose pin
does not hold, a dependency with a known advisory that the gate does not catch,
or a released artefact whose checksum does not cover what an operator runs.

**Anything that sends data off the host.** Nothing in this project is meant to
make a network request at run time, and personal data could reach it through a
file an operator supplies. Something that transmits, uploads or phones home is a
vulnerability whatever else it does. That rule is issue #16.

## What does not count

A panic on a file that this repository refuses on purpose is the refusal working.
Every reader here is written to refuse rather than repair, and a clear error on a
malformed file is the intended behaviour. What makes it a vulnerability is the
memory safety, the unbounded work or the escape from the named directory, not
the fact that the program stopped.

An operator running the tool on their own bad file, using their own resources,
is not a denial of service. There is no shared instance to deny.

A defect in the physics, the numerics or the scoring that is nobody's crafted
input is an ordinary issue on the tracker, and it is more likely to be read and
fixed there.

## Where to send one

Privately, through GitHub, at

    https://github.com/iderex/messlatte/security/advisories/new

That opens a draft advisory visible only to the maintainer, which is the right
channel because the tracker on this repository is public and an issue is a
disclosure.

Say what you did, what happened, and what you expected. A file that reproduces it
is worth more than a description of one. Where the report is about a reader,
attach the bytes rather than a transcription: the whole point of the class is
what the exact bytes do.

There is no bounty and no service level here. What you get is a reply, and the
fix landing in public with the report credited unless you ask otherwise.

## What is supported

The mainline. This project has made no release and promises nothing about a
file format yet, so there is no older version to receive a fix. When that
changes, what a release promises is issue #91 and this section moves with it.
