# 0008. Nothing leaves the host unless the operator deliberately federates

Status: accepted.

Issue: [#10](https://github.com/iderex/indexwerk/issues/10).

## The decision

Neither the core, the C interface nor the Python package opens a socket,
resolves a name, reads a credential, or writes anything outside the paths the
caller passed in. There is no usage reporting, no crash reporting, no version
check, and no download of data at first run. The default and only behaviour is
fully local.

If federation is ever offered, meaning any feature that sends anything to a
machine the operator does not run, it is a separate opt-in that the operator
turns on deliberately. It is documented before it is built, it is never on by
default, and it is never enabled by an upgrade.

## The reasons

### The input is unpublished research

The thing an operator hands to this library is an unpublished tensor expression.
That is their research before it is published, and in a competitive field it is
the most sensitive material they have. A crash report containing the expression
that crashed is an exfiltration of exactly that, however well meant, and it
arrives at the moment the operator is least able to notice it.

### The personal data question is not softened by the subject matter

Names, affiliations, file paths containing a home directory, and machine
identifiers all reach a telemetry payload without anybody intending it. That the
payload is about physics changes none of it. The reliable way to guarantee that
none of it leaves the host is for the library to have no route out at all,
rather than a route with a policy in front of it.

### A library with no network use runs where this work actually happens

An air-gapped environment, a cluster job with no egress, and a container with
networking disabled are all places this computation is run. A library that
reaches the network on some path fails in those places, and it fails at the
point of use rather than at the point of installation.

### A guarantee nobody can check is worth nothing

So this decision is paired with tests rather than with a paragraph. The tests
are named below and they are owed by other milestones; this record is the
statement they enforce, not the enforcement.

## What follows from this record elsewhere

A test that fails if any network call is attempted, in M7, on
[#36](https://github.com/iderex/indexwerk/issues/36).

A dependency policy that refuses a transitive dependency pulling in a network
stack, in M8, on [#38](https://github.com/iderex/indexwerk/issues/38).

The same statement in the operator documentation, where a reader will find it,
in M10, on [#49](https://github.com/iderex/indexwerk/issues/49) and
[#50](https://github.com/iderex/indexwerk/issues/50).

Until those land, this record is prose and nothing refuses a violation of it.
