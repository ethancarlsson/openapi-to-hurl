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
    - [ ] Plain Text
- [ ] Improve JSON formatting
    - [ ] Offer unformatted JSON option
    - [ ] Add new line at the end of objects and array structs
- [ ] Implement filtering options
    - [x] Filter by operationId
    - [ ] Filter by tag
- [x] Implement output options
    - [x] To files and directories
    - [x] To console
