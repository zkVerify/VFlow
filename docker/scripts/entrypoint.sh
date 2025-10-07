#!/usr/bin/env bash

# This script is inspired from :
# https://github.com/zkVerify/zkVerify/blob/main/docker/scripts/entrypoint.sh
# (extended to handle also parachain parameters)
#
# It performs the following tasks:
#
# - translation of environment variables to command line arguments
# - preparation before the node start (example keys injection)
# - launch of the actual node
#
# Environment variables should generally be in the form `PARA_*` for the parachain parameters and `ZKV_*` for the relay chain parameters.
# Environment variables in the form `PARA_CONF_*` and `ZKV_CONF_*` are translated to command line arguments based on these rules:
#
# 1. `PARA_CONF_` and `ZKV_CONF_` prefix are removed
# 2. if underscores (`_`) are present, they are replaced with dashes (`-`)
# 3. letters are replaced with lower case
# 4. prefix `--` is added
#
# Examples:
#
# - `PARA_CONF_BASE_PATH` -> `--base-path`
# - `PARA_CONF_BOOTNODES` -> `--bootnodes`
#
# Values of environment variables are used unmodified as values of command line arguments with the exception
# of `yes` being dropped (as a flag, example `PARA_CONF_COLLATOR`/`--validator`)

set -eEuo pipefail

####
# Function(s)
####
fn_die() {
  echo -e "\n\033[1;31m${1}\033[0m\n" >&2
  exit "${2:-1}"
}

log_bold_green() {
  echo -e "\n\033[1;32m${1}\033[0m\n"
}

log_green() {
  echo -e "\n\033[0;32m${1}\033[0m\n"
}

log_yellow() {
  echo -e "\n\033[1;33m${1}\033[0m\n"
}

get_arg_name_from_env_name() {
  local env_name="$1"
  local prefix="$2"

  # Extract the base name by removing the prefix
  local base_name="${env_name:${#prefix}}"

  # Replace underscores with hyphens and convert to lowercase
  arg_name="${base_name//_/-}"
  arg_name="${arg_name,,}"

  # Prefix the argument name with --
  echo "--${arg_name}"
}

get_arg_value_from_env_value() {
  local env_value="$1"
  local arg_value="${env_value}"

  # Check if the value is exactly "yes".
  if [ "${arg_value}" == "yes" ]; then
    # If it is "yes", set arg_value to an empty string (""), indicating no flag should be set.
    arg_value=""
  fi

  # Output the processed arg_value
  echo "${arg_value}"
}

# Function to validate chain specification and download if necessary
validate_and_download() {
  local CHAIN_VAR_NAME="$1"
  local URL_VAR_NAME="$2"

  # Dynamically retrieve the values of the variables using indirect expansion
  local CHAIN_VALUE="${!CHAIN_VAR_NAME}"
  local SPEC_FILE_URL="${!URL_VAR_NAME}"

  # Check if the chain variable is empty
  if [ -z "${CHAIN_VALUE}" ]; then
    fn_die "ERROR: '${CHAIN_VAR_NAME}' variable can not be empty or undefined. Aborting ..."
  fi

  # Echo the chain value
  echo "  ${CHAIN_VAR_NAME}=${CHAIN_VALUE}"

  # Check if CHAIN_VALUE points to an existing .json file and download it otherwise
  if [[ "${CHAIN_VALUE}" == *.json ]] && [ ! -s "${CHAIN_VALUE}" ]; then
    # Attempt to download the file if it doesn't exist
    if [ -n "${SPEC_FILE_URL}" ]; then
      log_green "INFO: Spec file '${CHAIN_VALUE}' does not exist. Downloading it from '${SPEC_FILE_URL}' ..."
      gosu "${RUN_USER}" mkdir -p "$(dirname "${CHAIN_VALUE}")" || fn_die "ERROR: could not create directory '$(dirname "${CHAIN_VALUE}")' for spec file. Aborting ..."
      cd "$(dirname "${CHAIN_VALUE}")"
      # Retry mechanism for downloading the spec file
      MAX_RETRIES=3
      RETRY_DELAY=5
      for ((i=1; i<=MAX_RETRIES; i++)); do
        log_green "INFO: Attempt #${i} of ${MAX_RETRIES} to download spec file ..."
        if gosu "${RUN_USER}" aria2c --file-allocation=none -s16 -x16 --max-tries=3 --continue=true "${SPEC_FILE_URL}" -o "$(basename "${CHAIN_VALUE}")"; then
          log_green "INFO: Spec file downloaded successfully."
          break  # Exit loop on success
        fi
        if [ "${i}" -lt "${MAX_RETRIES}" ]; then
          log_yellow "WARNING: Download failed. Retrying in ${RETRY_DELAY} seconds ..."
          sleep "${RETRY_DELAY}"
        else
          fn_die "ERROR: Failed to download spec file from '${SPEC_FILE_URL}' after ${MAX_RETRIES} attempts. Aborting ..."
        fi
      done
    else
      fn_die "ERROR: The variable '${CHAIN_VAR_NAME}' (spec file) is set to '${CHAIN_VALUE}', which is a .json file that does not exist. The variable '${URL_VAR_NAME}' is empty, therefore the file can not be downloaded. Aborting ..."
    fi
  fi
}

####
# Main
####
# Sanity check
if [ -z "${RUN_USER:-}" ]; then
  fn_die "ERROR: Required environment variable 'RUN_USER' is not defined. This should never happen. It must be set under a 'Dockerfile' used to build the image. Aborting ..."
fi

if [ -z "${BINARY:-}" ]; then
  fn_die "ERROR: Required environment variable 'BINARY' is not defined. This should never happen. Aborting ..."
else
  # If the user built the image with multiple binaries,
  # we consider the first one to be the canonical one
  # To start with another binary, the user can either:
  #  - use the --entrypoint option
  #  - pass the ENV BINARY with a single binary
  IFS=',' read -r -a BINARIES <<< "${BINARY}"
  PARA_NODE_BIN="${BINARIES[0]}"
  command -v "${PARA_NODE_BIN}" &>/dev/null || fn_die "ERROR: '${PARA_NODE_BIN}' binary can not be found on the run user's 'PATH'=${PATH}"
  log_bold_green "🔧 zkVerify parachain node binary: ${PARA_NODE_BIN}"
fi

####
# Migrate the relay chain dir to the new path if it was using the old default
####
ZKV_BASE_PATH=${ZKV_CONF_BASE_PATH:-}

if [ -z "${ZKV_BASE_PATH}" ]; then
  source_chain_dir="/data/node/polkadot"
  dest_chain_dir="/data/node/zkv_relay"

  if [ -d "${source_chain_dir}" ] && [ ! -e "${dest_chain_dir}" ]; then
    log_green "INFO: old default config detected. Moving '${source_chain_dir}' to '${dest_chain_dir}' ..."
    mv "${source_chain_dir}" "${dest_chain_dir}" || fn_die "ERROR: failed to move '${source_chain_dir}' to '${dest_chain_dir}'!"
  fi
fi

####
# Parachain node's configurations
####
log_bold_green "=== Parachain node's configuration:"
PARA_CONF_BASE_PATH=${PARA_CONF_BASE_PATH:-}
PARA_CONF_CHAIN=${PARA_CONF_CHAIN:-}
PARA_SPEC_FILE_URL="${PARA_SPEC_FILE_URL:-}"
PARA_NODE_KEY="${PARA_NODE_KEY:-}"
PARA_NODE_KEY_FILE="${PARA_NODE_KEY_FILE:-/data/node_key.dat}"
PARA_SECRET_PHRASE="${PARA_SECRET_PHRASE:-}"
PARA_SECRET_PHRASE_FILE="${PARA_SECRET_PHRASE_FILE:-/data/secret_phrase.dat}"
PARA_KEYSTORE_PATH="${PARA_KEYSTORE_PATH:-/data/keystore}"

# Loop through all arguments to check for --dev
for arg in "$@"; do
  if [ "${arg}" == "--dev" ]; then
    log_green "INFO: '--dev' flag detected! Running in development mode."
    # You can place any logic for development mode here
    DEV_MODE="true"
    break
  fi
done

if [ "${DEV_MODE:-false}" != "true" ]; then
  # Call the function for PARA_CONF_CHAIN
  validate_and_download "PARA_CONF_CHAIN" "PARA_SPEC_FILE_URL"
fi

prefix="PARA_CONF_"
conf_args=()
while IFS='=' read -r -d '' var_name var_value; do
  if [[ "${var_name}" == "${prefix}"* ]]; then
    # Get argument name from the environment variable name
    arg_name="$(get_arg_name_from_env_name "${var_name}" "${prefix}")"

    # If the value contains commas, handle it by splitting the values
    if [[ "${var_value}" == *","* ]]; then
      IFS=',' read -ra values <<< "${var_value}"
      for value in "${values[@]}"; do
        # Add the argument name and each value
        conf_args+=("${arg_name}")
        conf_args+=("${value}")
      done
    else
      # If there is no comma, just add the argument with the value
      if [ "${var_value}" != "yes" ]; then
        conf_args+=("${arg_name}")
        conf_args+=("${var_value}")
      else
        conf_args+=("${arg_name}")
      fi
    fi

    # Debug output
    echo "  ${var_name}=${var_value} -> ${arg_name} ${var_value}"
  fi
done < <(env -0)

# Session keys handling
if [ -n "${PARA_SECRET_PHRASE}" ]; then
  # Creating secret phrase file and keystore location if secret phrase environmental variable is provided
  mkdir -p "${PARA_KEYSTORE_PATH}" || fn_die "ERROR: was not able to create keystore '${PARA_KEYSTORE_PATH}' directory for storing session keys.  Exiting ..."
  chmod 700 "${PARA_KEYSTORE_PATH}" && chown "${RUN_USER}" "${PARA_KEYSTORE_PATH}"

  conf_args+=(--keystore-path "${PARA_KEYSTORE_PATH}")

  printf "%s" "${PARA_SECRET_PHRASE}" > "${PARA_SECRET_PHRASE_FILE}" || fn_die "ERROR: was not able to create '${PARA_SECRET_PHRASE_FILE}' file.  Exiting ..."
  chmod 0400 "${PARA_SECRET_PHRASE_FILE}" && chown "${RUN_USER}" "${PARA_SECRET_PHRASE_FILE}"
  echo -e "  PARA_SECRET_PHRASE_FILE=${PARA_SECRET_PHRASE_FILE}\n"

  unset PARA_SECRET_PHRASE

  injection_args=()
  if [ -n "${PARA_CONF_BASE_PATH}" ]; then
    injection_args+=("$(get_arg_name_from_env_name PARA_CONF_BASE_PATH ${prefix})")
    injection_args+=("$(get_arg_value_from_env_value "${PARA_CONF_BASE_PATH}")")
  fi
  if [ -n "${PARA_CONF_CHAIN}" ]; then
    injection_args+=("$(get_arg_name_from_env_name PARA_CONF_CHAIN ${prefix})")
    injection_args+=("$(get_arg_value_from_env_value "${PARA_CONF_CHAIN}")")
  fi

  injection_args+=(--keystore-path "${PARA_KEYSTORE_PATH}")

  log_green "INFO: injecting session keys with the following args:"
  echo "---------------------------------"
  echo "  ${injection_args[*]}"
  echo -e "---------------------------------\n"

  declare -A session_keys=(
    ["aura"]="Sr25519"
    ["acco"]="ecdsa"
  )

  # Loop through the keys and inject each one
  for key_type in "${!session_keys[@]}"; do
    log_green "INFO: injecting session key '${key_type}' ..."

    gosu "${RUN_USER}" "${PARA_NODE_BIN}" key insert "${injection_args[@]}" \
      --scheme "${session_keys["${key_type}"]}" \
      --suri "${PARA_SECRET_PHRASE_FILE}" \
      --key-type "${key_type}" || fn_die "ERROR: could not inject '${key_type}' session key. Exiting ..."
  done
fi

# Creating node key file if node key is provided via environmental variable or generating random one if doesn't already exist
if [ ! -s "${PARA_NODE_KEY_FILE}" ] ; then
  log_yellow "WARNING: node key file '${PARA_NODE_KEY_FILE}' does not exist. Creating one ..."
  if [ -n "${PARA_NODE_KEY}" ]; then
    log_green "INFO: creating node key file using 'PARA_NODE_KEY' environmental variable ..."
    printf "%s" "${PARA_NODE_KEY}" > "${PARA_NODE_KEY_FILE}" || fn_die "ERROR: was not able to create '${PARA_NODE_KEY_FILE}' file.  Exiting ..."
  else
    injection_args=()
    if [ -n "${PARA_CONF_CHAIN}" ]; then
      injection_args+=("$(get_arg_name_from_env_name PARA_CONF_CHAIN ${prefix})")
      injection_args+=("$(get_arg_value_from_env_value "${PARA_CONF_CHAIN}")")
    fi
    log_green "INFO: generating RANDOM node key since 'PARA_NODE_KEY' environmental variable was not set ..."
    gosu "${RUN_USER}" "${PARA_NODE_BIN}" key generate-node-key "${injection_args[@]}" --file "${PARA_NODE_KEY_FILE}" || fn_die "ERROR: was not able to generate RANDOM node key file. Exiting ..."
  fi
fi
chmod 0400 "${PARA_NODE_KEY_FILE}" && chown "${RUN_USER}" "${PARA_NODE_KEY_FILE}"
echo -e "  PARA_NODE_KEY_FILE=${PARA_NODE_KEY_FILE}\n"

conf_args+=(--node-key-file "${PARA_NODE_KEY_FILE}")
unset PARA_NODE_KEY

####
# Relaychain node's configuration
####
ZKV_SPEC_FILE_URL="${ZKV_SPEC_FILE_URL:-}"

if [ "${DEV_MODE:-false}" != "true" ]; then
  # Call the function for EVM_CONF_CHAIN
  validate_and_download "ZKV_CONF_CHAIN" "ZKV_SPEC_FILE_URL"
fi

prefix="ZKV_CONF_"
if env | grep -q "^${prefix}"; then
  log_bold_green "=== Relaychain node's configuration:"
  relaychain_appended_any=""
  while IFS='=' read -r -d '' var_name var_value; do
    if [[ "${var_name}" == "${prefix}"* ]]; then
      # Append separator only once
      if [[ -z "${relaychain_appended_any}" ]]; then
        relaychain_appended_any="true"
        conf_args+=("--")
      fi

      # Get argument name from the environment variable name
      arg_name="$(get_arg_name_from_env_name "${var_name}" "${prefix}")"

      # If the value contains commas, handle it by splitting the values
      if [[ "${var_value}" == *","* ]]; then
        IFS=',' read -ra values <<< "${var_value}"
        for value in "${values[@]}"; do
          # Add the argument name and each value
          conf_args+=("${arg_name}")
          conf_args+=("${value}")
        done
      else
        # If there is no comma, just add the argument with or without the value depending on the condition
        if [ "${var_value}" != "yes" ]; then
          conf_args+=("${arg_name}")
          conf_args+=("${var_value}")
        else
          conf_args+=("${arg_name}")
        fi
      fi

      # Debug output
      echo "  ${var_name}=${var_value} -> ${arg_name} ${var_value}"
    fi
  done < <(env -0)
fi

log_green "INFO: launching '${PARA_NODE_BIN}' with the following args:"
echo "---------------------------------"
echo "  ${conf_args[*]}" "$@"
echo -e "---------------------------------\n"

exec gosu "${RUN_USER}" "${PARA_NODE_BIN}" "${conf_args[@]}" "$@"
