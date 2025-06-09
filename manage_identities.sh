#!/bin/bash

# Purpose: Remove all other identities managed by dfx and Switch to new identity

# Check if dfx is installed
if ! command -v dfx &> /dev/null; then
    echo "Error: dfx is not installed or not in PATH. Please install dfx and try again."
    exit 1
fi

# Switch to the anonymous identity
echo "Switching to the anonymous identity..."
dfx identity use anonymous
if [ $? -ne 0 ]; then
    echo "Failed to switch to anonymous identity."
    exit 1
fi
echo "Now using the anonymous identity."

# Get the list of identities, excluding 'anonymous'
IDENTITIES=$(dfx identity list | grep -v "anonymous" | grep -v "Using identity" | awk '{print $1}')

# Check if there are any identities to remove
if [ -z "$IDENTITIES" ]; then
    echo "No identities (except anonymous) found to remove."
fi

# Confirm with the user before proceeding
echo "The following identities will be removed (anonymous will be preserved):"
echo "$IDENTITIES"
echo -n "Are you sure you want to remove these identities? (y/N): "
read -r CONFIRM

if [[ "$CONFIRM" != "y" && "$CONFIRM" != "Y" ]]; then
    echo "Operation cancelled."
    exit 0
fi

# Iterate through each identity and remove it
for IDENTITY in $IDENTITIES; do
    echo "Removing identity: $IDENTITY"
    dfx identity remove "$IDENTITY"
    if [ $? -eq 0 ]; then
        echo "Successfully removed $IDENTITY"
    else
        echo "Failed to remove $IDENTITY"
    fi
done

dfx identity import --seed-file .seedphrase DEV --storage-mode plaintext &&
dfx identity use DEV &&
echo "All identities have been removed and using new one from seed phrase!"