#!/bin/sh
set -e

# Start game WebSocket server
/app/server &

# Start docs site (Leptos SSR)
export MAPS_DIR=/app/public/maps
export PUBLIC_DIR=/app/docs-site-assets
export LEPTOS_SITE_ROOT=/app/docs-site-assets
export LEPTOS_SITE_ADDR=127.0.0.1:3002
export LEPTOS_SITE_PKG_DIR=pkg
/app/docs-site &

# Start nginx in foreground
exec nginx -g 'daemon off;'
