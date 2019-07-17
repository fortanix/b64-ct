# Fast and secure Base64 encoding/decoding

This crate provides an implementation of Base64 encoding/decoding that is
designed to be resistant against software side-channel attacks (such as timing
& cache attacks), see the [documentation] for details. On certain platforms it
also uses SIMD making it very fast. This makes it suitable for e.g. decoding
cryptographic private keys in PEM format.

The API is very similar to the base64 implementation in the old rustc-serialize
crate, making it easy to use in existing projects.

[documentation]: https://docs.rs/b64-ct

# Implementation

Depending on the runtime CPU architecture, this crate uses different
implementations with different security properties.

* x86 with AVX2: All lookup tables are implemented with SIMD
  instructions. No secret-dependent memory accceses.
* Other platforms: Lookups are limited to 64-byte aligned lookup tables. On
  platforms with 64-byte cache lines this may be sufficient to prevent
  certain cache side-channel attacks. However, it's known that this is [not
  sufficient for all platforms].

We graciously welcome contributed support for other platforms!

[not sufficient on some platforms]: https://ts.data61.csiro.au/projects/TS/cachebleed/

# Contributing

We gratefully accept bug reports and contributions from the community.
By participating in this community, you agree to abide by [Code of Conduct](./CODE_OF_CONDUCT.md).
All contributions are covered under the Developer's Certificate of Origin (DCO).

## Developer's Certificate of Origin 1.1

By making a contribution to this project, I certify that:

(a) The contribution was created in whole or in part by me and I
have the right to submit it under the open source license
indicated in the file; or

(b) The contribution is based upon previous work that, to the best
of my knowledge, is covered under an appropriate open source
license and I have the right under that license to submit that
work with modifications, whether created in whole or in part
by me, under the same open source license (unless I am
permitted to submit under a different license), as indicated
in the file; or

(c) The contribution was provided directly to me by some other
person who certified (a), (b) or (c) and I have not modified
it.

(d) I understand and agree that this project and the contribution
are public and that a record of the contribution (including all
personal information I submit with it, including my sign-off) is
maintained indefinitely and may be redistributed consistent with
this project or the open source license(s) involved.

# License

This project is primarily distributed under the terms of the Mozilla Public License (MPL) 2.0, see [LICENSE](./LICENSE) for details.
