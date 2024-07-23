# ic-config-canister

This canister stores `(replica-version, json-config)` pairs and allows to query them via HTTP.

Assuming that `dfx` has been started with `dfx start` and is running, you deploy the canister locally with:
```
dfx deploy
```

In order to add `(replica-version, json-config)` use the following command:
```
dfx canister call ic-config-canister add '("version_foo", "{\"config_bar\":1}")'
```
Note that the quotes should be properly escaped in the JSON config.


You can query the list of stored versions by making a GET HTTP request to the `/versions` endpoint.
```
curl "http://be2us-64aaa-aaaaa-qaabq-cai.localhost:4943/versions"
```
Replace the URL prefix with your local canister id.

You can query the config of a particular replica version by requesting `/config?version=version_foo` endpoint.
```
curl "http://be2us-64aaa-aaaaa-qaabq-cai.localhost:4943/config?version=version_foo"
```
Replace the URL prefix with your local canister id.
