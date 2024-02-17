# Openapi to Hurl

A tool to create Hurl files (https://hurl.dev/) from openapi specs. The project 
is incomplete and in the early stages of development.

## TODO
- [ ] response validation options
    - [x] No validation
    - [x] Response code validation
    - [ ] Full response validation
        - [x] Plain Text
        - [x] JSON
        - [ ] XML
- [ ] Support for request/response formats
    - [x] JSON
    - [ ] XML
    - [ ] GraphQL
    - [x] Plain Text
- [x] Improve JSON formatting
    - [x] Offer unformatted JSON option
    - [x] Add new line at the end of objects and array structs
- [ ] Implement filtering options
    - [x] Filter by operationId
    - [ ] Filter by tag
- [x] Implement output options
    - [x] To files and directories
    - [x] To console
- [ ] CI/CD
    - [x] Setup github actions to run tests and build
    - [x] Setup CI/CD for benchmarking
    - [x] Setup CI/CD for changelog and versioning
    - [ ] Trigger release to cargo on version bump
- [ ] Refactor
    - [ ] Add options to have this work like a true unix tool that takes
        text as input and outputs text
