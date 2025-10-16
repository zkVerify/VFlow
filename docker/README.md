This folder contains some resources for running VFlow in a Docker environment.

## Full docker image

To generate a node image without bothering about local resources, local Rust installation and so on you can simply use:

(from the project root folder)

```bash
docker build -t zkverify/vflow-node:local -f docker/dockerfiles/vflow-node.Dockerfile .
```

This will generate a docker image named <b>zkverify/vflow-node:local</b> with a fresh source compilation.
You can then run it with:

```bash
> docker run -ti --rm --entrypoint vflow-node zkverify/vflow-node:local --dev
```
All arguments after `zkverify/vflow-node:local` image name will be passed to the node executable.

## Docker compose

At this path you can find an example docker compose to run locally 2 relay chain + 3 parachain nodes (two collators and one rpc node, exposing both the parachain and the relaychain rpcs).

```bash
docker/compose/test-docker-compose.yaml
```
Before executing it you need to generate also the chain descriptors for both chains, and after the startup of the nodes you will need to register the parachain manually in the relay chain.<br>
Here the full steps:<br>
<i>(All the commands are assumed to be executed from the docker/dockerfiles/ folder)</i>

1- Generate **relaychain** spec:

```bash
docker run --entrypoint zkv --rm zkverify/relay-node:latest  build-spec --disable-default-bootnode --chain local  > ./staging/relay-spec.json
```

2- Generate **relaychain** raw spec:

```bash
docker run --entrypoint zkv --rm -v ./staging/relay-spec.json:/tmp/relay-spec.json zkverify/relay-node:latest build-spec --chain local --disable-default-bootnode --raw > ./staging/relay-spec-raw.json
```

3- Generate **parachain** spec:

```bash
docker run --rm --entrypoint vflow-node zkverify/vflow-node:local build-spec --chain local --disable-default-bootnode > ./staging/para-spec.json
```
Before the next step, you can modify it if you want to change any parameter or add preminted account.<br>
The generated one is already configured to use the as initial collators the ones defined in docker/resources/envs/parachain. (Alith and Baltathar)<br>

4- Generate **parachain** raw spec:

```bash
docker run --rm -v ./staging/para-spec.json:/tmp/para-spec.json --entrypoint vflow-node zkverify/vflow-node:local build-spec --chain /tmp/para-spec.json  --disable-default-bootnode --raw > ./staging/para-spec-raw.json
```

5- Generate **parachain** wasm

```bash
docker run --rm -v ./staging/para-spec-raw.json:/tmp/para-spec-raw.json --entrypoint vflow-node zkverify/vflow-node:local export-genesis-wasm --chain /tmp/para-spec-raw.json > ./staging/para-genesis.wasm
```

6- Generate **parachain** geneis state

```bash
docker run --rm -v ./staging/para-spec-raw.json:/tmp/para-spec-raw.json --entrypoint vflow-node zkverify/vflow-node:local export-genesis-state --chain /tmp/para-spec-raw.json > ./staging/para-genesis-state
```

7- Start the nodes with

```bash
docker compose -f ./docker/compose/test-docker-compose.yaml up
```

All the nodes should be up and running now! <br>
You can inspect the relay chain here: https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9845#/explorer<br>
And the parachain here: https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9945#/explorer<br>


Relay chain nodes should already produce blocks, parachain ones still not because you have to register the parachain manually in the relay chain.
Connect to the relay chain [here](https://polkadot.js.org/apps/?rpc=ws%3A%2F%2F127.0.0.1%3A9845#/explorer) and follow [this tutorial](https://docs.substrate.io/tutorials/build-a-parachain/connect-a-local-parachain/): (sections 'Reserve a unique identifier' and 'Register with the local relay chain').<br>
Remember to use parachain id 2000 when registering, and the WASM + genesis state generated at point 5 and 6.<br>
