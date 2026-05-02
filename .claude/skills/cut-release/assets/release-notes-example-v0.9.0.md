## What's New

This release adds initial support for displaying eBPF link information and ships a one-line installer script. It also drops the `procfs` dependency in favor of reading kernel info directly, and migrates CI from `cross` to native GitHub ARM64 runners.

### Features
- Add initial `bpf_link_info` support (#196)
- Add installer script for bpftop (#220)

### Fixes
- Apply `cargo fmt` across the tree (#211)

### Maintenance
- Drop the `procfs` dependency (#222)
- Bump `rand` to 0.8.6 to clear GHSA-cq8v-f236-94qc (#224)
- Migrate CI from `cross` to native GitHub runners (#218)
- Add `cargo fmt` check to CI (#219)
- Drop needless borrow in `bpf_type` sort comparator
- Bump `actions/github-script` from 7.1.0 to 9.0.0 (#217)

## New Contributors
* @EricccTaiwan made their first contribution in https://github.com/jfernandez/bpftop/pull/211

## Contributors
* Cheng-Yang Chou @EricccTaiwan
* Jose Fernandez @jfernandez
* ver-nyan @ver-nyan

**Full Changelog**: https://github.com/jfernandez/bpftop/compare/v0.8.0...v0.9.0
