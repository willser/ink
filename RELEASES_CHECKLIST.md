# Release Checklist

Obviously there has to be an ink! PR, it has to be approved by core team
members, and the CI has to be green.

Generally, we publish individual crates if there is a `PATCH` release for
them. For `MINOR` and `MAJOR` we bump the versions of all crates in this
mono-repository in lockstep.

Reasons for doing it this way:
* Keeps our `RELEASES.md` simple and easy to grasp; we can still bundle all
  changes there together and don't have to go to a more fine-grained crate level.
* Easier to follow reported issues. All crates are closely tied together
  anyway and if someone reports an issue with `3.1` we know what they mean.
* Easier to keep the workshop/documentation/playground/ui's in sync.
* Easier for users to follow changes; those will all be listed in one section
  of the changelog, bundled together, released at the same time.

## For `PATCH` releases

1. Make sure you've added the changes to the `[Patch Released]` section of the `RELEASES.md`.
1. Make sure you've merged the latest `master` into your branch.
1. Execute `cargo publish --dry-run` for the crate which got a `PATCH` update.
1. If successful, execute `cargo publish`.
1. Merge the PR if everything went fine.
1. Post announcement to those Element channels:
	* [Smart Contracts & Parity ink!](https://matrix.to/#/#ink:matrix.parity.io)
	* [ink! Announcements](https://matrix.to/#/#ink-announcements:matrix.parity.io)

## For `MINOR` and `MAJOR` releases

1. Make sure you've moved the changes in the `RELEASES.md` from `[Unreleased]` into a new section for the release.
1. Make sure you've merged the latest `master` into your branch.
1. Execute `cargo unleash em-dragons --dry-run` in the ink! repository.
1. If successful, execute `cargo unleash em-dragons`.
1. Merge the PR if everything went fine.
1. Replace `vX.X.X` with the new version in the following command and then execute it: 
	```
	git checkout master && git pull && git tag vX.X.X && git push origin vX.X.X
	```
1. Create a GitHub release for this tag. In the [tag overview](https://github.com/paritytech/ink/tags) you'll see your new tag appear. Click the `…` on the right of the tag and then `Create release`.
1. Paste the release notes that appear in the [`RELEASES.md`](https://github.com/paritytech/ink/blob/master/RELEASES.md) there,
the title of the release should be `vX.X.X`.
1. Post announcement to those Element channels:
	* [Smart Contracts & Parity ink!](https://matrix.to/#/#ink:matrix.parity.io)
	* [ink! Announcements](https://matrix.to/#/#ink-announcements:matrix.parity.io)

