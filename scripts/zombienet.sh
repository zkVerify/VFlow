#!/bin/bash

ZOMBIENET_V=v1.3.128
BIN_DIR=relay-bin
CHAIN=${CHAIN:-"volta"} # or mainnet

case "$(uname -s)" in
    Linux*)     MACHINE=Linux;;
    Darwin*)    MACHINE=Mac;;
    *)          exit 1
esac

arch="$(uname -m)"

if [ $MACHINE = "Linux" ]; then
  if [ "$arch" == "arm64" ] || [ "$arch" == "aarch64" ]; then
        ZOMBIENET_BIN="zombienet-linux-arm64"
    else
        ZOMBIENET_BIN="zombienet-linux-x64"
    fi
elif [ $MACHINE = "Mac" ]; then
  if [ "$arch" == "arm64" ] || [ "$arch" == "aarch64" ]; then
        ZOMBIENET_BIN="zombienet-macos-arm64"
    else
        ZOMBIENET_BIN="zombienet-macos-x64"
    fi
fi



build_zkVerify() {
  echo "cloning zkVerify repository..."
  CWD=$(pwd)
  mkdir -p "$BIN_DIR"
  pushd /tmp
    git clone https://github.com/zkVerify/zkVerify.git
    pushd zkVerify
      # Get new tags from remote
      git fetch --tags > /dev/null
      # Get latest tag name
      latestTag=$(git describe --tags "$(git rev-list --tags --max-count=1)")
      # Checkout latest tag
      git checkout $latestTag
      echo "building zkVerify executable..."
      cargo build --release -p zkv-relay --features "fast-runtime"
      cp target/release/zkv-relay "$CWD/$BIN_DIR"
      cp target/release/zkv-relay-execute-worker "$CWD/$BIN_DIR"
      cp target/release/zkv-relay-prepare-worker "$CWD/$BIN_DIR"
    popd
  popd
}

zombienet_init() {
  if [ ! -f $ZOMBIENET_BIN ]; then
    echo "fetching zombienet executable..."
    curl -LO https://github.com/paritytech/zombienet/releases/download/$ZOMBIENET_V/$ZOMBIENET_BIN
    chmod +x $ZOMBIENET_BIN
  fi
  if [ ! -f $BIN_DIR/zkv-relay ]; then
   echo "Fetch_zkVerify() not yet implemented: you must execute 'zombienet build'"
  fi
  if [ ! -f $BIN_DIR/zkv-relay-execute-worker ] || [ ! -f $BIN_DIR/zkv-relay-prepare-worker ]; then
   echo "The zkVerify node binary is available, but external workers are missing."
   exit -1
  fi
}

zombienet_build() {
  if [ ! -f $ZOMBIENET_BIN ]; then
    echo "fetching zombienet executable..."
    curl -LO https://github.com/paritytech/zombienet/releases/download/$ZOMBIENET_V/$ZOMBIENET_BIN
    chmod +x $ZOMBIENET_BIN
  fi
  if [ ! -f $BIN_DIR/zkv-relay ]; then
    build_zkVerify
  fi
}

zombienet_devnet() {
  zombienet_init
  cargo build --release
  echo "spawning local relay chain plus devnet ${CHAIN} as a parachain..."
  local dir=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
  ./$ZOMBIENET_BIN spawn "$dir/../zombienet-config/devnet-${CHAIN}.toml" -p native
}


zombienet_debug() {
  zombienet_init
  echo "compiling parachain node"
  cargo build
  echo "spawning local relay chain plus devnet as a parachain..."
  local dir=$( cd -- "$( dirname -- "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )
  ./$ZOMBIENET_BIN spawn "$dir/../zombienet-config/debug.toml" -p native
}


print_help() {
  echo "This is a shell script to automate the execution of zombienet."
  echo ""
  echo "$ ./zombienet.sh init         # fetches zombienet and zkVerify executables"
  echo "$ ./zombienet.sh build        # builds zkVerify executables from source"
  echo "$ ./zombienet.sh devnet       # spawns a local relay chain plus parachain devnet-local as a parachain"
}

SUBCOMMAND=$1
case $SUBCOMMAND in
  "" | "-h" | "--help")
    print_help
    ;;
  *)
    shift
    zombienet_${SUBCOMMAND} $@
    if [ $? = 127 ]; then
      echo "Error: '$SUBCOMMAND' is not a known SUBCOMMAND." >&2
      echo "Run './zombienet.sh --help' for a list of known subcommands." >&2
        exit 1
    fi
  ;;
esac

