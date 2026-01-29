## Epi-Sign

## Comment run le projet

### Pré-requis

1. Avoir Docker
2. Avoir un accès à [l'intra epitech](https://intra.epitech.eu)

### Lancer le projet

**Option 1 : Avec Docker Compose (Recommandé)**
```shell
git clone git@github.com:Code-Barru/epi-sign.git
cd epi-sign
docker compose up -d
```

**Option 2 : En local (pour le développement)**
Voir le guide détaillé dans [LOCAL_SETUP.md](./LOCAL_SETUP.md)

### URLs après démarrage

- Frontend : http://localhost:8081
- Backend API : http://localhost:3001
- Swagger/API Docs : http://localhost:3001/api/docs

## Comment signer avec le projet

### Bypass l'anti-bot

Pour signer, il d'abord faut bypass le système anti-bot de l'intra d'epitech...

Après avoir lancé le container de la base de donnée :

Sur Windows :

```shell
./start-cookie-worker.bat
```

Sur Unix :

```shell
chmod u+x start-cookie-worker.sh
./start-cookie-worker.sh
```

Ensuite, les cookies sont valable 24h.

### Créer un utilisateur de base

Si vous n'avez pas de clé de registration, créez un utilisateur de base :

**Avec Docker** (Recommandé) :
```bash
docker compose exec -T db psql -U postgres -d postgres < backend/scripts/create_user.sql
```

**Ou avec le script** :
```bash
cd backend
# Sur Unix/Mac
./scripts/create_user.sh
# Sur Windows
scripts\create_user.bat
```

Cela créera un utilisateur avec :
- Username : `admin`
- Password : `admin123`

### Suite de call API à faire sur le backend

(voir la référence de l'api avec le swagger sur `http://localhost:3000/api/docs`)

Se connecter : `/api/auth/login` (ou utiliser l'interface web)

Enregistrer son jwt : `/api/users/me/update-jwt`

Récupérer la liste des Ulids des User pour lesquels ont veut signer : `/api/users/`

Vérifier que le serveur a bien des cookies : `/api/sign/status`

Signer : `/api/sign`

## Fonctionnalité EDSquare

Le projet supporte maintenant la validation de codes EDSquare :

1. **Créer une signature manuscrite** : Aller sur `/dashboard` et créer votre signature
2. **Se connecter à EDSquare** : Aller sur `/edsquare` et utiliser la section de connexion en haut
3. **Valider un code** : Entrer le `planning_event_id` et le `code` EDSquare

Les cookies EDSquare sont sauvegardés automatiquement et valables 24h.
