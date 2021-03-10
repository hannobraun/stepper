# Contributing to Stepper

## Introduction

This file documents the procedures for developing the Stepper project. It targets contributors and maintainers. Contributions, be it in the form of issues or pull requests, are always welcome, so don't be shy!

At this point, this document is far from a comprehensive guide. It will be extended over time. Please open an issue, if anything is unclear, or if you need information not present here.


## Release management

This repository contains multiple crates: The main `stepper` crate, as well as one additional crate per supported driver. All version for the different crates are kept in sync, for the following reasons:

- They all share the same lifecycle. Breaking changes to `stepper` will require breaking changes to all driver crates, and breaking changes to drivers without breaking changes to `stepper` are probably going to be rare.
- Syncing versions like that makes it much easier to manage release tags in the repository.
- It also makes it much easier to handle changelogs, as all crates can share just one.

This means if a new version of `stepper` is released, all driver crates are released around the same time, sharing the same version number. Updating the versions will be done in the same commit, and that commit will be tagged with the version number for that release.

If a new release of some, but not all, crates is required, the version of all other crates is still updated, and a release is published. This is not ideal, but it prevents a situation where a crate must skip a version number, which could be confusing. Such a situation can occur because each release is tagged in the repository, and we can't create multiple tags for the same version number (and if we could, we really shouldn't).

### Publishing a release

This procedure should really be automated, but for now it is at least documented.

1. Make sure your local `main` branch is up-to-date:

``` bash
git switch main
git pull --rebase
```

2. Choose new version number, according to [Semantic Versioning]. The following steps will use `a.b.c` as a placeholder for the chosen version.

3. Create a release branch (replace `a.b.c` with actual version)

``` bash
git switch -c publish-a.b.c
```

4. Update changelog: Go through all pull requests since the last release and mention the relevant ones. Use existing changelog entries as the template. Commit this to the repository.

5. Update version in top-level `Cargo.toml`, README.md, and re-generate driver crates. Commit changes.

``` bash
# missing: Update version in top-level Cargo.toml and README.md
cargo task generate-drivers
```

6. Push branch, open a pull request. This makes sure the CI runs and gives other maintainers a chance to weigh in.

7. Once ready, publish `stepper`: Run `cargo publish` from the repository root.

8. Publish all driver crates. Go into each directory in `drivers/`, run `cargo publish` there.

9. Tag the release

``` bash
git tag va.b.c
```

10. Merge pull request, clean up your local branch.

``` bash
# missing: merge pull request
git switch main
git pull --rebase
git remote prune origin
git branch -d publish-a.b.c
```

11. Push the release tag

``` bash
git push --tag
```


[Semantic Versioning]: https://semver.org/
