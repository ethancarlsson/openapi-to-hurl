# Openapi to Hurl

A tool to create Hurl files (https://hurl.dev/) from openapi specs. The project 
is incomplete and in the early stages of development.

## TODO
- [ ] response validation options
    - [x] No validation
    - [x] Response code validation
    - [ ] Full response validation
- [ ] Support for request/response formats
    - [x] JSON
    - [ ] XML
    - [ ] GraphQL
    - [x] Plain Text
- [ ] Improve JSON formatting
    - [ ] Offer unformatted JSON option
    - [ ] Add new line at the end of objects and array structs
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
- [ ] Refactor
    - [ ] Create an intermediate type (a "HttpOperation"?) that is independent
        of both openapi and hurl, then move it to it's own module.
