# Coffee

An in-progress CLI tool to update a remote database each time you have a coffee, started during the COVID-19 lockdown. Used to play with rust and protos and async.

## Structure

### `coffee-common`

A set of proto's and DB helpers.

### `coffee-client`

This is the CLI client.

### `coffee-rpc-server`

RPC server for the CLI.

### Remaining

- Finish up the CLI flows for add and list coffees.
- Registration just gives an "api key" which is the sha1 of the email - should be salted, emailed etc etc.
- Web frontend as a new crate using `coffee-common` to render the coffee-drinking as a graph over time for a given api key.

## License

This is licensed under the MIT license.