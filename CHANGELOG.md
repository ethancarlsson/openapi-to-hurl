# [0.2.0](https://github.com/ethancarlsson/openapi-to-hurl/compare/v0.1.0...v0.2.0) (2024-01-29)


### Bug Fixes

* Give more detail in error messages ([33cc1f9](https://github.com/ethancarlsson/openapi-to-hurl/commit/33cc1f98c59d83a3a64fcd2d6bb1880b4a838100))


### Features

* Add `--content-type <CONTENT_TYPE>` for content type selection ([5a21403](https://github.com/ethancarlsson/openapi-to-hurl/commit/5a214033c563e2cb5fa0d8690fe12c4a37396dc1))
* Allow user to specify `--formatting no-formatting` ([1a1bced](https://github.com/ethancarlsson/openapi-to-hurl/commit/1a1bcedcb54801f5853953c4f535eb9147f5e224))



# [0.1.0](https://github.com/ethancarlsson/openapi-to-hurl/compare/efce9c41afe8ab4fda27d67183685ab4924d0264...v0.1.0) (2024-01-22)


### Bug Fixes

* add new line after each header variable ([6f0f92c](https://github.com/ethancarlsson/openapi-to-hurl/commit/6f0f92c0ccf8a1a4a0a7dff62c1ce4af61538c42))
* Handle existing directory error ([a92a899](https://github.com/ethancarlsson/openapi-to-hurl/commit/a92a8997e69c79195cd46d193cf17643ba75d87f))


### Code Refactoring

* accept array as multiple options ([2d6da74](https://github.com/ethancarlsson/openapi-to-hurl/commit/2d6da74d1ea5ab0b9519fdca54d2941a4180529a))


* feat!: use operation_id as the filename ([71b9578](https://github.com/ethancarlsson/openapi-to-hurl/commit/71b9578548156dcb5ce82599d9d3139f3dc8f61b))


### Features

* add console output mode ([7aeaec2](https://github.com/ethancarlsson/openapi-to-hurl/commit/7aeaec2fe52649db1d6e56dd2918d0e7529ec17e))
* add logging ([ff8b88d](https://github.com/ethancarlsson/openapi-to-hurl/commit/ff8b88d307cd6bb2a9284176c93e2534a404071a))
* add request body support for JSON ([a3a8af7](https://github.com/ethancarlsson/openapi-to-hurl/commit/a3a8af77ae75fe8e50a69410f41f129ec5b19e80))
* Allow options for automatic query params ([6a55fd8](https://github.com/ethancarlsson/openapi-to-hurl/commit/6a55fd8a948742617aca3c5339e4b2fa23b16cd4))
* indicate how many files were created ([8b42b2f](https://github.com/ethancarlsson/openapi-to-hurl/commit/8b42b2ff7e4d097ff0079fb2c16e18a261d5fd53))
* merge new variables to existing files ([0a44e07](https://github.com/ethancarlsson/openapi-to-hurl/commit/0a44e07723e9057b60a68a3241fa67f2c4117b05))
* Nest methods in path directories ([efce9c4](https://github.com/ethancarlsson/openapi-to-hurl/commit/efce9c41afe8ab4fda27d67183685ab4924d0264))
* Select specific operations in oai ([74cfb3c](https://github.com/ethancarlsson/openapi-to-hurl/commit/74cfb3c19cf3cae5fee136c527708bb65beee189))


### BREAKING CHANGES

* This will break any integrations that rely on the file
name matching the HTTP method
* will no longer use comma seperated list. Programs using a
comma seperated list will now have to pass multiple arguments of `i` or
`select-operation-id`
* if programs relied on overwriting the existing files they
will now need to use the option --variable-update-strategy overwrite
* CLI will now default to no query params



