# Contribution to Mitosis

Mitosis still has some unfinished designs and implementations, or you may also have your ideas. Welcome to contribute to mitosis!

We will further refine our workflow of contribution in the future.

## Conduct

The Tokio project adheres to the [Rust Code of Conduct](https://github.com/rust-lang/rust/blob/master/CODE_OF_CONDUCT.md).

## Contributing in Issues

For any contribution, you can follow the following steps:

1. Open an issue for discussion. For instance, if you believe that you have discovered a bug in mitosis, creating a new issue is the way to report it.

2. Help to resolve the issue. Typically this is done either in the form of demonstrating that the issue reported is not a problem after all, or more often, by opening a Pull Request that changes some bit of something in mitosis in a concrete and reviewable manner.

If you have reviewed existing documentation and still have questions or are having problems, you can open a github discussion asking for help.

## Pull Requests

Pull Requests are the way concrete changes are made to the code, documentation, and dependencies in the mitosis repository.

Even tiny pull requests (e.g., one character pull request fixing a typo in API documentation) are greatly appreciated. Before making a large change, it is required to first open an issue describing the change to solicit feedback and guidance. This will increase the likelihood of the PR getting merged.

### Tests

If the change being proposed alters code (as opposed to only documentation for example), it is either adding new functionality to mitosis or it is fixing existing, broken functionality. In both of these cases, the pull request should include one or more tests to ensure the correctness of the implementation. For example, one or more unit test cases are required to demonstrate the code change. See the document [here](../tests-and-benchmarks.md) for more details of the tests of mitosis.

## Credits

The contribution guide of mitosis is inspired by that of [tokio](https://github.com/tokio-rs/tokio/blob/master/CONTRIBUTING.md).
