# Open API to Hurl

A tool to create Hurl files (https://hurl.dev/) from Open API specifications.

## Installation

### Cargo
```sh
cargo install openapi-to-hurl
```
### Brew
```sh
brew tap ethancarlsson/openapi-to-hurl
brew install openapi-to-hurl
```
NOTE: All binaries are more MacOS, Linux users will need to install using Cargo.

### Usage
openapi-to-hurl accepts the path to an Open API 3 specification, produces hurl
requests and writes them to stdout.
```sh
# Spec can be a JSON file
% openapi-to-hurl ./openapi.json 
# or a YAML file
% openapi-to-hurl ./openapi.yaml
```
openapi-to-hurl can also accept the specification from stdin
```sh
% cat ./openapi.json | openapi-to-hurl
# Or more usefully
% some-tool-generating-specs | openapi-to-hurl
```
stdout can then be passed into hurl

```sh
% cat ./openapi.json | openapi-to-hurl | hurl --variable host=https://example.com
```
NOTE: "host" is created as a variable by default. This makes it easier to switch
hosts between local, staging and production, but it also means that "host" will
need to be passed as a variable to hurl.

#### Producing .hurl Files
One of the main motivations for this tool is to make it easier to start exploring 
a new API with hurl. For this, it's more convenient that we separate each hurl 
request into a different file.

```sh
% openapi-to-hurl test_files/pet_store.json --out-dir test_hurl_files 
INFO Created or updated 4 hurl files in test_hurl_files
% ls test_hurl_files
addPet.hurl            listPets.hurl          petstore.swagger.io_v1 showPetById.hurl       updatePet.hurl
```
You can then start running these files with hurl via the CLI.
```sh
% hurl test_hurl_files/*.hurl --variables-file=test_hurl_files/petstore.swagger.io_v1
```
Or you can explore it using a plugin an editor plugin for [Neovim](https://github.com/jellydn/hurl.nvim)
or [VSCode](https://github.com/pfeiferj/vscode-hurl)


#### Test Generation
Test generation is another very clear use case for this tool.

We can generate assertions to go with the .hurl file like this
```sh
% openapi-to-hurl test_files/pet_store.json --validation body
POST {{host}}/pets
{
  "id": 3,
  "inner": {
    "test": "string"
  },
  "name": "string",
  "photo_urls": [
    "https://example.com/img.png",
    "https://example.com/img2.png"
  ],
  "tag": "string"
}

HTTP *
[Asserts]

status < 400
jsonpath "$" isCollection
jsonpath "$.id" isInteger
jsonpath "$.inner" isCollection
jsonpath "$.inner.test" isString
jsonpath "$.name" isString
jsonpath "$.photo_urls" isCollection
```

Running the following commands will test the responses for the entire API.
```sh
openapi-to-hurl test_files/pet_store.json --validation body | hurl --variable host=http://petstore.swagger.io/v1
```

However, hurl lacks tools for creating assertions on where a property can have
multiple types. So whenever this tool detects a property that is not required, 
is nullable, or can have multiple types for any other reason, it ignores the property.

This means that the command above might be useful for detecting when the API does
NOT match the specification, but it wouldn't be able to detect that the API does
match the specification.

This means this tool is best used as a starter for tests that are then manually
edited afterwards.

```sh
openapi-to-hurl test_files/pet_store.json --validation body -o output/directory
```

## Limitations
openapi-to-hurl only works with JSON and plain-text content types.

## Changelog
Changelog available at: https://github.com/ethancarlsson/openapi-to-hurl/blob/master/CHANGELOG.md

## Documentation
Online man page available at: https://ethancarlsson.github.io/openapi-to-hurl/

## [Licence](https://github.com/ethancarlsson/openapi-to-hurl/blob/master/license.md)
The MIT License (MIT)

