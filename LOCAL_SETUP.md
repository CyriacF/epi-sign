# Guide pour lancer le projet en local

## Option 1 : Avec Docker Compose (Recommandé - Plus simple)

### Prérequis
- Docker et Docker Compose installés

### Étapes

1. **Lancer tous les services** :
```bash
docker compose up -d
```

2. **Vérifier que les services sont lancés** :
```bash
docker compose ps
```

3. **Accéder à l'application** :
- Frontend : http://localhost:8081
- Backend API : http://localhost:3001
- Swagger : http://localhost:3001/api/docs
- Base de données PostgreSQL : localhost:5432

4. **Voir les logs** :
```bash
# Tous les services
docker compose logs -f

# Un service spécifique
docker compose logs -f backend
docker compose logs -f frontend
docker compose logs -f db
```

5. **Arrêter les services** :
```bash
docker compose down
```

---

## Option 2 : Sans Docker (Développement local)

### Prérequis
- PostgreSQL 17 installé et lancé
- Rust (avec cargo) installé
- Node.js et npm installés

### Étapes

#### 1. Base de données PostgreSQL

**Option A : Utiliser Docker uniquement pour la base de données** :
```bash
docker compose up -d db
```

**Option B : Installer PostgreSQL localement** :
```bash
# macOS
brew install postgresql@17
brew services start postgresql@17

# Linux (Ubuntu/Debian)
sudo apt-get install postgresql-17
sudo systemctl start postgresql

# Créer la base de données
createdb postgres
# Ou avec psql
psql -U postgres -c "CREATE DATABASE postgres;"
```

#### 2. Configuration de la base de données

Créer un fichier `.env` dans le dossier `backend/` :
```bash
cd backend
cat > .env << EOF
DATABASE_URL=postgres://postgres:postgres@localhost:5432/postgres
JWT_SECRET=secret
REGISTER_KEY=EpiteslaGANG
EOF
```

**Optionnel — Webhook bilan EDSquare** : après une validation de code EDSquare pour plusieurs personnes, le backend peut envoyer un webhook POST avec le bilan (qui a été validé, qui a échoué). Pour l’activer, ajoute dans le même `.env` :
```bash
EDSQUARE_WEBHOOK_URL=https://votre-url-de-webhook
```
Où mettre le webhook :
- **Discord** : Salon → Intégrations → Webhooks → Nouveau webhook → copier l’URL.
- **Slack** : Apps → Incoming Webhooks → ajouter une URL.
- **API custom** : toute URL acceptant un POST JSON. Le corps envoyé est de la forme :
  `{ "event": "edsquare_validation_multi", "global_success": true, "validated": ["Alice", "Bob"], "failed": [{"username": "Charlie", "message": "..."}] }`
Si la variable est absente ou vide, aucun webhook n’est envoyé.

**Optionnel — Webhook bilan signature multiple** : après une signature multiple (POST `/api/sign` avec plusieurs ULIDs), le backend peut envoyer un webhook avec le bilan (URL signée, qui a été validé, qui a échoué). Pour l’activer, ajoute dans le `.env` :
```bash
SIGN_WEBHOOK_URL=https://votre-url-de-webhook
```
Même format que EDSquare (Discord : message lisible ; API custom : `{ "event": "sign_multi", "url", "validated", "failed" }`).

**Optionnel — Supprimer d’autres utilisateurs (admin)** : pour pouvoir supprimer un utilisateur par son id depuis la machine (curl, script, etc.), définis une clé admin dans le `.env` :
```bash
ADMIN_KEY=ta-cle-secrete-admin
```
Ensuite, depuis un terminal ou un script :
```bash
# Remplacer USER_ID par l’id de l’utilisateur (ex: 01ARZ3NDEKTSV4RRFFQ69G5FAV) et TA_CLE par ta valeur ADMIN_KEY
curl -X DELETE "http://localhost:3000/api/admin/users/USER_ID" \
  -H "X-Admin-Key: TA_CLE"
```
Réponse : `204 No Content` si l’utilisateur a été supprimé, `404` si l’id n’existe pas, `403` si la clé est absente ou invalide. Si `ADMIN_KEY` n’est pas défini, l’endpoint renvoie `501`.

#### 3. Backend (Rust)

```bash
cd backend

# Installer diesel-cli (si pas déjà installé)
cargo install diesel_cli --no-default-features --features postgres

# Lancer les migrations
diesel migration run

# Lancer le serveur en mode développement
cargo run
```

Le backend sera accessible sur : http://localhost:3000

#### 4. Frontend (SvelteKit)

Dans un nouveau terminal :
```bash
cd frontend

# Installer les dépendances (première fois seulement)
npm install

# Lancer le serveur de développement
npm run dev
```

Le frontend sera accessible sur : http://localhost:5173 (ou le port indiqué par Vite)

**Note** : Le frontend est configuré pour proxy les requêtes `/api` vers `http://localhost:3000`

---

## Créer un utilisateur de base

Si vous n'avez pas de clé de registration (`REGISTER_KEY`), vous pouvez créer un utilisateur de base directement dans la base de données :

### Option 1 : Avec le script SQL (Recommandé - Plus simple)

**Avec Docker** :
```bash
docker compose exec -T db psql -U postgres -d postgres < backend/scripts/create_user.sql
```

**Ou avec le script shell** :
```bash
# Sur Unix/Mac
cd backend
chmod +x scripts/create_user.sh
./scripts/create_user.sh

# Sur Windows
cd backend
scripts\create_user.bat
```

L'utilisateur créé sera :
- **Username** : `admin`
- **Password** : `admin123`

⚠️ **Important** : Changez le mot de passe après la première connexion !

### Option 2 : Via SQL direct

```sql
-- Se connecter à la base de données
psql -U postgres -d postgres

-- Insérer l'utilisateur (le hash correspond à "admin123")
INSERT INTO users (id, username, password_hash) 
VALUES (
    '01HZ0000000000000000000000',
    'admin',
    'a665a45920422f9d417e4867efdc4fb8a04a1f3fff1fa07e998e86f7f7a27ae3'
);
```

## Tester la fonctionnalité EDSquare

### 1. Se connecter
- Aller sur http://localhost:8081 (ou http://localhost:5173 en dev)
- Se connecter avec l'utilisateur créé (admin/admin123)

### 2. Créer une signature manuscrite
- Aller sur `/dashboard`
- Créer une signature manuscrite dans votre profil

### 3. Se connecter à EDSquare
- Aller sur `/edsquare`
- Cliquer sur l'icône de connexion en haut
- Entrer votre email et mot de passe EDSquare
- Les cookies seront sauvegardés automatiquement

### 4. Valider un code EDSquare
- Entrer le `planning_event_id` et le `code`
- Cliquer sur "Valider le code"

---

## Dépannage

### Le backend ne démarre pas
- Vérifier que PostgreSQL est lancé : `docker compose ps` ou `pg_isready`
- Vérifier la variable `DATABASE_URL` dans `.env`
- Vérifier que les migrations sont à jour : `cd backend && diesel migration run`

### Le frontend ne peut pas se connecter au backend
- Vérifier que le backend est lancé sur le port 3000
- Vérifier la configuration du proxy dans `frontend/vite.config.ts`
- Vérifier les CORS dans le backend

### Erreur de migration
```bash
cd backend
diesel migration revert
diesel migration run
```

### Voir les logs du backend
```bash
# Avec Docker
docker compose logs -f backend

# Sans Docker (en local)
# Les logs s'affichent directement dans le terminal où vous avez lancé `cargo run`
```

### Réinitialiser la base de données
```bash
# Avec Docker
docker compose down -v
docker compose up -d db
cd backend && diesel migration run

# Sans Docker
dropdb postgres
createdb postgres
cd backend && diesel migration run
```

---

## URLs importantes

- **Frontend (production)** : http://localhost:8081
- **Frontend (dev)** : http://localhost:5173
- **Backend API** : http://localhost:3001 (Docker) ou http://localhost:3000 (local)
- **Swagger/API Docs** : http://localhost:3001/api/docs (Docker) ou http://localhost:3000/api/docs (local)
- **Base de données** : localhost:5432
