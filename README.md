# Wormhole

Wormhole is a substrate node containing tendermint-client pallet, which contains [tendermint_light_client].
Since this repository builds upon original [node-template] repository, you can read documentation on how to run the node [here](https://github.com/substrate-developer-hub/substrate-node-template/blob/master/README.md).

## tendermint-client pallet interfaces

### Extrinsics

tendermint-client pallet exposes following extrinsics to create a tendermint light client, and update
the light client with new tendermint header and authority sets.

1. `initClient(payload: Vec<u8>)`: Creates and initializes new tendermint light client. The payload is json encoded `TMCreateClientPayload` and if it is valid
new light client is created and initialized.

2. `updateClient(payload: Vec<u8>)`: Updates existing light client. The payload is json encoded `TMUpdateClientPayload` and if it is valid, client is updated with
new height and new validator set.

### Storage APIs

tendermint-client pallet exposes following storage apis to get the list of created clients and their status.

1. `availableClients() -> Vec<Bytes>`: Returns list of clients created till now.

2. `clientInfoMap(Bytes) -> TMClientInfo`: Returns information about particular client. Information is encoded `TMClientInfo` structure.

[tendermint_light_client]: https://github.com/ChorusOne/tendermint-light-client
[node-template]: https://github.com/substrate-developer-hub/substrate-node-template
