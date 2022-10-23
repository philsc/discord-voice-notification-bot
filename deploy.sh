#!/bin/bash

set -o errexit
set -o nounset
set -o pipefail

readonly SSH_HOST="$1"
readonly BUNDLE="bot_image_bundle.tar"

cat > /tmp/remote_setup.sh <<EOF
set -x
clean_up() {
    rm -f /tmp/${BUNDLE}
    rm -f /tmp/remote_setup.sh
}
trap clean_up EXIT

docker load --input /tmp/${BUNDLE}

docker stop discord-voice-notification-bot || :
docker rm discord-voice-notification-bot || :
docker run -d \
    --name discord-voice-notification-bot \
    -v /data/discord-voice-notification-bot/token_file:/data/token_file \
    -e DISCORD_TOKEN_FILE=/data/token_file \
    philsc.net/discord-voice-notification-bot
EOF
clean_up() { rm /tmp/remote_setup.sh; }
trap clean_up EXIT


echo "Copying bundle to ${SSH_HOST}."
rsync -Lc --progress "${BUNDLE}" /tmp/remote_setup.sh "${SSH_HOST}":/tmp

echo "Installing bundle remotely and running it."
ssh -t "${SSH_HOST}" 'su root -c "bash /tmp/remote_setup.sh"'
