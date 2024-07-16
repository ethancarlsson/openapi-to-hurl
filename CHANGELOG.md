# [1.1.0](https://github.com/ethancarlsson/openapi-to-hurl/compare/v1.0.1...v1.1.0) (2024-07-16)


### Bug Fixes

* Correctly use OAS3.1 schema validation for exclusiveMinimum and exclusiveMaximum ([d2c85bd](https://github.com/ethancarlsson/openapi-to-hurl/commit/d2c85bd2b87776fb1a128b7e46886e1656aecac5))


### Features

* Wrap errors to make it clear what user should do when getting a spec error ([88ed836](https://github.com/ethancarlsson/openapi-to-hurl/commit/88ed83661e9ee0315390e52b2130cebf2d8410b4))



## [1.0.1](https://github.com/ethancarlsson/openapi-to-hurl/compare/v1.0.0...v1.0.1) (2024-07-08)


### Bug Fixes

* Allow http status codes other than 200 ([0e404d6](https://github.com/ethancarlsson/openapi-to-hurl/commit/0e404d64c39a07e732df68dbd9f95ee30fd17a36))



# [1.0.0](https://github.com/ethancarlsson/openapi-to-hurl/compare/v0.4.0...v1.0.0) (2024-02-23)


### Bug Fixes

* Ensure a valid enum is always selected by always using the first ([7af0e88](https://github.com/ethancarlsson/openapi-to-hurl/commit/7af0e88f646e6bf21e12b3f1975f3134b256ddbf))


### Code Refactoring

* Change and rename cli arguments ([2905870](https://github.com/ethancarlsson/openapi-to-hurl/commit/2905870dc5b79e34e7f2536123a0db9844e6a893))
* Remove 200 validation and rename validation options ([5569739](https://github.com/ethancarlsson/openapi-to-hurl/commit/5569739bfd12c8ab96631cbebb6b51af2d121078))


### Features

* `header-vars` can now be used with `'r'` for short ([abfc1ac](https://github.com/ethancarlsson/openapi-to-hurl/commit/abfc1ac37ad514d1c50ad21fff8f7010227c4fc3))
* Accept stdin as input to the program when used in pipeline ([a042c73](https://github.com/ethancarlsson/openapi-to-hurl/commit/a042c731534283b4e3d755cf8e8f1e6ca972567b))
* Add a `required` option to the `query-params` argument ([3f12146](https://github.com/ethancarlsson/openapi-to-hurl/commit/3f121465789bf4fd03db62582d613bc504f94fb0))
* Add ability to generate files in a flat directory structure ([2273525](https://github.com/ethancarlsson/openapi-to-hurl/commit/2273525b0c21fe7602dd7edacec79900934d78b5))
* Adds a --version command to get the current version ([b1ad2c8](https://github.com/ethancarlsson/openapi-to-hurl/commit/b1ad2c8ecc2c2fa19feda3cee446b5ab413482bb))
* Automatically output to directory if the directory argument is provided ([954cbeb](https://github.com/ethancarlsson/openapi-to-hurl/commit/954cbeb9226c33c9ec8f4977b928c19adec8dca1))


### Performance Improvements

* Introduce benchmarking for other scenarios ([de400c3](https://github.com/ethancarlsson/openapi-to-hurl/commit/de400c3aa29c02028442e4c17f257021f1f95a1f))


### BREAKING CHANGES

* Hurl files will now be generated in a flat directory structure. To use the
previous structure, which grouped operations in directories by path, use the option
`--grouping path`.
* `required` is now the default option to `query-params`. This ensures that
the hurl file will more likely be correct but minimal by default.
* A number of arguments and options have been renamed.
- `select-operation-id` -> `operation-id`
- `variables-update-strategy` -> `variables-file-update`
- `handle-errors` -> `error-handling`
* The option `validate-response` has been renamed to `validation` the options
for `validation` are: `none`, `non-error-code`, `body`, `body-with-optionals`
* The `output-to` has been removed. Using the argument will result in an error
* The program will now default to returning results to stdout.
This is so that we "[e]xpect the output of every program to become the input to another, as yet unknown, program"
Also, the primary argument name changed from "path" to "input".



# [0.4.0](https://github.com/ethancarlsson/openapi-to-hurl/compare/v0.3.0...v0.4.0) (2024-02-18)


### Features

* Add tags as a filtering option ([6b25371](https://github.com/ethancarlsson/openapi-to-hurl/commit/6b25371ef8b958d83ac76f93308816ce4f9c23c6))
* Allow user to generate validation for optionals ([e2cea76](https://github.com/ethancarlsson/openapi-to-hurl/commit/e2cea768d230505618f34e6e663fe3ae0e08b7c0))
* Allow user to ignore and log errors instead of terminating program ([75e4fc1](https://github.com/ethancarlsson/openapi-to-hurl/commit/75e4fc102f91f0410a525f4466006fa72310a90c))
* Allow users to set log level and improve debug logging ([c8a7d9a](https://github.com/ethancarlsson/openapi-to-hurl/commit/c8a7d9aabcd8f4b35581b594e09caa0198ac9dff))



# [0.3.0](https://github.com/ethancarlsson/openapi-to-hurl/compare/v0.2.2...v0.3.0) (2024-02-17)


### Bug Fixes

* Fix handling of allOf in json request bodies ([e7e9e41](https://github.com/ethancarlsson/openapi-to-hurl/commit/e7e9e410eb4f5aeda4c09f737744673fc15f14c0))


### Features

* Add `non-error` validation option and deprecate `http-200` ([36c97a6](https://github.com/ethancarlsson/openapi-to-hurl/commit/36c97a678e6c75871f94abbab0f9bf426a4104c7))
* Full validation for json schemas ([fa23083](https://github.com/ethancarlsson/openapi-to-hurl/commit/fa230838b5acb72d20d6c39ebe11e7eca71273a0))
* Full validation for plain text ([099c08e](https://github.com/ethancarlsson/openapi-to-hurl/commit/099c08e1a8e647b3d1d6d0d7d9069922c6711ed0))
* Validate json objects that use allOf properties ([887dfbe](https://github.com/ethancarlsson/openapi-to-hurl/commit/887dfbe1fdc2911c0c23c728d9a73cb974e6fb36))



