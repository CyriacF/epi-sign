#!/usr/bin/env bash
# Supprime un utilisateur par son id via l'API admin.
# Usage: ./delete_user.sh <USER_ID>
# Nécessite ADMIN_KEY dans l'environnement ou dans backend/.env

set -e

USER_ID="${1:-}"
if [ -z "$USER_ID" ]; then
  echo "Usage: $0 <USER_ID>"
  echo "Exemple: $0 01ARZ3NDEKTSV4RRFFQ69G5FAV"
  exit 1
fi

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
BACKEND_DIR="$(cd "$SCRIPT_DIR/.." && pwd)"
if [ -f "$BACKEND_DIR/.env" ]; then
  set -a
  source "$BACKEND_DIR/.env"
  set +a
fi

if [ -z "${ADMIN_KEY:-}" ]; then
  echo "Erreur: ADMIN_KEY doit être défini (dans .env ou l'environnement)"
  exit 1
fi

API_BASE="${API_BASE:-http://localhost:3000}"
URL="$API_BASE/api/admin/users/$USER_ID"

echo "Suppression de l'utilisateur $USER_ID..."
HTTP=$(curl -s -w "%{http_code}" -o /tmp/delete_user_resp.txt -X DELETE "$URL" -H "X-Admin-Key: $ADMIN_KEY")

if [ "$HTTP" = "204" ]; then
  echo "Utilisateur supprimé."
elif [ "$HTTP" = "404" ]; then
  echo "Utilisateur non trouvé (id: $USER_ID)"
  exit 1
elif [ "$HTTP" = "403" ]; then
  echo "Clé admin invalide ou manquante."
  cat /tmp/delete_user_resp.txt
  exit 1
elif [ "$HTTP" = "501" ]; then
  echo "ADMIN_KEY n'est pas configuré côté serveur."
  cat /tmp/delete_user_resp.txt
  exit 1
else
  echo "Erreur HTTP $HTTP"
  cat /tmp/delete_user_resp.txt
  exit 1
fi
