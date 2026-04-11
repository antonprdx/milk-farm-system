#!/bin/bash
set -euo pipefail

DOMAIN="${1:?Usage: $0 <domain> <email>}"
EMAIL="${2:?Usage: $0 <domain> <email>}"

CERTS_DIR="$(dirname "$0")/certs"
mkdir -p "$CERTS_DIR"

if command -v certbot &>/dev/null; then
    certbot certonly --standalone -d "$DOMAIN" --non-interactive --agree-tos -m "$EMAIL"
    cp /etc/letsencrypt/live/"$DOMAIN"/fullchain.pem "$CERTS_DIR/"
    cp /etc/letsencrypt/live/"$DOMAIN"/privkey.pem "$CERTS_DIR/"
    echo "Certificates copied to $CERTS_DIR"
    echo "Add to crontab for auto-renewal:"
    echo "  0 3 * * * certbot renew --quiet && cp /etc/letsencrypt/live/$DOMAIN/*.pem $CERTS_DIR/ && docker compose -f docker-compose.prod.yml restart nginx"
else
    echo "certbot not found. Install it first:"
    echo "  apt install certbot"
    echo ""
    echo "Or use Docker-based approach:"
    echo "  docker run --rm -p 80:80 -v \"$CERTS_DIR:/etc/letsencrypt\" certbot/certbot certonly --standalone -d $DOMAIN --non-interactive --agree-tos -m $EMAIL"
fi
