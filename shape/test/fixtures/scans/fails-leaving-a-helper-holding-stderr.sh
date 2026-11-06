#!/bin/sh
# starts a helper that runs for 5 seconds and keeps the script's stderr open, writes
# "config/routes.rb could not be loaded" to stderr and exits 1 at once.
sleep 5 >/dev/null &
echo "config/routes.rb could not be loaded" >&2
exit 1
