# Belay

Belay makes it easy to run your CI checks locally.

```bash
# in a git repo with either gitlab or github CI configured
$ belay
Checking 'build':
Success!
Checking 'test':
... test output
..
.
Success!
```

Belay can also setup pre-commit or pre-push git hooks in your repo.

```bash
# to create a pre-push hook
$ belay hook push

# to create a pre-commit hook
$ belay hook commit
```

## License

Licensed under either of

 * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
 * MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

### Contribution

Unless you explicitly state otherwise, any contribution intentionally submitted for inclusion in the work by you, as defined in the Apache-2.0 license, shall be dual licensed as above, without any additional terms or conditions.
