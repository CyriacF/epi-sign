#!/bin/bash

# Script pour créer un utilisateur de base
# Usage: ./scripts/create_user.sh

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/../.." && pwd)"

echo "🔧 Création de l'utilisateur de base..."

# Méthode 1: Avec Docker (recommandé)
if command -v docker &> /dev/null && docker compose ps db &> /dev/null; then
    echo "📦 Utilisation de Docker..."
    docker compose exec -T db psql -U postgres -d postgres < "$SCRIPT_DIR/create_user.sql"
    exit $?
fi

# Méthode 2: Sans Docker (nécessite psql en local)
if command -v psql &> /dev/null; then
    echo "💻 Utilisation de psql local..."
    
    # Vérifier que DATABASE_URL est défini
    if [ -z "$DATABASE_URL" ]; then
        if [ -f "$PROJECT_ROOT/backend/.env" ]; then
            export $(cat "$PROJECT_ROOT/backend/.env" | grep -v '^#' | xargs)
        else
            echo "❌ Erreur: DATABASE_URL n'est pas défini et aucun fichier .env trouvé"
            echo "   Créez un fichier backend/.env avec DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres"
            exit 1
        fi
    fi
    
    psql "$DATABASE_URL" -f "$SCRIPT_DIR/create_user.sql"
    exit $?
fi

# Méthode 3: Avec cargo (si disponible)
if command -v cargo &> /dev/null; then
    echo "🦀 Utilisation de cargo..."
    cd "$PROJECT_ROOT/backend"
    
    if [ -z "$DATABASE_URL" ]; then
        if [ -f .env ]; then
            export $(cat .env | grep -v '^#' | xargs)
        else
            echo "❌ Erreur: DATABASE_URL n'est pas défini"
            exit 1
        fi
    fi
    
    cargo run --bin create_user
    exit $?
fi

echo "❌ Erreur: Aucune méthode disponible (Docker, psql ou cargo)"
exit 1
