#!/bin/bash
set -euo pipefail

DOMAIN="${1:?Usage: $0 <domain> <email>}"
EMAIL="${2:?Usage: $0 <domain> <email>}"

CERTS_DIR="$(dirname "$0")/certs"
mkdir -p "$CERTS_DIR"

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

if command -v certbot &>/dev/null; then
    certbot certonly --standalone -d "$DOMAIN" --non-interactive --agree-tos -m "$EMAIL"
    cp /etc/letsencrypt/live/"$DOMAIN"/fullchain.pem "$CERTS_DIR/"
    cp /etc/letsencrypt/live/"$DOMAIN"/privkey.pem "$CERTS_DIR/"
    echo "Certificates copied to $CERTS_DIR"

    RENEW_SCRIPT="$CERTS_DIR/renew-and-reload.sh"
    cat > "$RENEW_SCRIPT" <<RENEW_EOF
#!/bin/bash
set -euo pipefail
certbot renew --quiet --deploy-hook "cp /etc/letsencrypt/live/$DOMAIN/*.pem $CERTS_DIR/ && docker compose -f $PROJECT_DIR/docker-compose.prod.yml restart nginx"
RENEW_EOF
    chmod +x "$RENEW_SCRIPT"

    (crontab -l 2>/dev/null | grep -v "renew-and-reload"; echo "0 3 * * * $RENEW_SCRIPT") | crontab -
    echo "Crontab installed for auto-renewal at 03:00 daily"
else
    echo "certbot not found. Install it first:"
    echo "  apt install certbot"
    echo ""
    echo "Or use Docker-based approach:"
    echo "  docker run --rm -p 80:80 -v \"$CERTS_DIR:/etc/letsencrypt\" certbot/certbot certonly --standalone -d $DOMAIN --non-interactive --agree-tos -m $EMAIL"
fi
