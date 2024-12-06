#!/bin/bash

if [ -z "$1" ] 
  then
    echo "Target system didn't specified!"
    exit 1
fi


VERSION=$(curl -s "https://api.github.com/repos/SmthFail/tiny_system_monitor/releases/latest" | grep '"tag_name":' | sed -E 's/.*"([^"]+)".*/\1/')
SYSTEM="${1}"

curl -OL "https://github.com/SmthFail/tiny_system_monitor/releases/download/${VERSION}/tsm-${VERSION}-${SYSTEM}.tar.gz"

mkdir -p ~/.tsm/bin

tar -xzvf ./tsm-${VERSION}-${SYSTEM}.tar.gz && \
    rm -rf ./tsm-${VERSION}-${SYSTEM}.tar.gz && \
    mv ./tsm ~/.tsm/bin

if [ $? -ne 0 ]; then
    echo "Error. Exit"
    exit 1
fi

export PATH=$PATH:~/.spoofdpi/bin

echo ""
echo "Successfully installed."
echo "Please add the line below to your rcfile(.bashrc or .zshrc etc..)"
echo ""
echo ">>    export PATH=\$PATH:~/.tsm/bin"



