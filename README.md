# core_tests

Tests for [Kinode core](https://github.com/kinode-dao/kinode) runtime and processes.

## Warnings

1. Following the directions below will place your Nectar node password and Alchemy API keys into your shell history.
2. The `kit run-tests` script deletes your node's filesystem: don't use on a node whose filesystem you care about.

## Usage

Requires:
* https://github.com/kinode-dao/kinode
* https://github.com/kinode-dao/kit

After installing `kit`, modify `tests.toml` and use `kit run-tests` to run test processes.
E.g., to run the `chat_test` here, run

```
necdev run-tests tests.toml
```

## Discussion

The above command will run the `key_value_test` twice, and the `chat_test` and `sqlite_test` once.
The purpose of running twice is to demonstrate the input syntax for tests.
Tests are process packages with one process within.
They accept a `TesterRequest::Run` `Request`.
They are input to the `kit run-tests` script via a `tests.toml` file.

The `tests.toml` file is an array of tests.
Each test will have the runtime reset to a fresh boot at the start.
Each `test_package_paths` within can specify a series of tests to run without resetting the state between them.

So in the example given above, the following will occur:
1. Reset state & launch two fake nodes: `first.uq` and `second.uq`.
2. Load `chat` into the "master" node: the first of the nodes specified, here, `first.uq`.
3. Run `sqlite_test`, `key_value_test`, and `chat_test` without resetting state.
3. Reset state & launch node.
4. Run `key_value_test`.
