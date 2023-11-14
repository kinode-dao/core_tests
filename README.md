# core_tests

Tests for Uqbar core runtime and processes.

## Warnings

1. Following the directions below will place your Uqbar node password and Alchemy API keys into your shell history.
2. The `uqdev run-tests` script deletes your node's filesystem: don't use on a node whose filesystem you care about.

## Usage

Requires:
* https://github.com/uqbar-dao/uqbar
* https://github.com/uqbar-dao/uqdev

After installing uqdev, modify `tests.toml` and use `uqdev run-tests` to run test processes.
E.g., to run the `key_value_test` and `sqlite_test` here, run

```
uqdev run-tests tests.toml
```

## Discussion

The above command will run the `key_value_test` twice and the `sqlite_test` once.
The purpose of running twice is to demonstrate the input syntax for tests.
Tests are process packages with one process within.
They accept a `TesterRequest::Run` `Request`.
They are input to the `uqbar-run-tests` script in a string of a twice-nested json array containing paths to those tests.
Each outer array will have the runtime reset to a fresh boot at the start.
Each inner array can specify a series of tests to run without resetting the state between them.

So in the example given above, the following will occur:
1. Reset state & launch node.
2. Run `sqlite_test` and `key_value_test` without resetting state.
3. Reset state & launch node.
4. Run `key_value_test`.
