# Contributing

## Introduction

This file documents the procedures for developing the Step/Dir project, be it as a contributor or a maintainer. Contribution, be it in the form of issues or pull requests, are always welcome, so don't be shy!


## Release management

This repository contains multiple crates: The main `step-dir` crate, as well as one additional crate per supported driver. All version for the different crates are kept in sync, for the following reasons:

- They all share the same lifecycle. Breaking changes to step-dir will require breaking changes to all driver crates, and breaking changes to drivers without breaking changes to step-dir are probably going to be rare.
- Syncing versions like that makes it much easier to manage release tags in the repository.
- It also makes it much easier to handle changelogs, as all crates can share just one.

This means if a new version of `step-dir` is released, all driver crates are released around the same time, sharing the same version number. Updating the versions will be done in the same commit, and that commit will be tagged with the version number for that release.

If a new release of some, but not all, crates is required, the version of all other crates is still updated, and a release is published. This is not ideal, but it prevents a situation where a crate must skip a version number, which could be confusing. Such a situation can occur because each release is tagged in the repository, and we can't create multiple tags for the same version number (and if we could, we really shouldn't).
