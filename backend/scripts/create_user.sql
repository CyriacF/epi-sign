-- Script SQL pour créer un utilisateur de base
-- Username: admin
-- Password: admin123
-- Hash pré-calculé: SHA256(SHA256("admin123") + "admin123") = 3b77faabcad11e04c4b51f9c669733288af7c3c141f2789e3b52dd8b141afc24

-- Vérifier si l'utilisateur existe déjà
DO $$
DECLARE
    user_exists BOOLEAN;
    user_id TEXT;
    password_hash TEXT := '3b77faabcad11e04c4b51f9c669733288af7c3c141f2789e3b52dd8b141afc24';
BEGIN
    -- Vérifier si l'utilisateur existe
    SELECT EXISTS(SELECT 1 FROM users WHERE username = 'admin') INTO user_exists;
    
    IF user_exists THEN
        RAISE NOTICE 'L''utilisateur admin existe déjà !';
    ELSE
        -- Générer un nouvel ID (ULID simulé avec un UUID)
        user_id := gen_random_uuid()::TEXT;
        
        -- Insérer l'utilisateur avec le hash pré-calculé
        INSERT INTO users (id, username, password_hash, jwt_intra_epitech, jwt_expires_at, signature_manuscrite)
        VALUES (
            user_id,
            'admin',
            password_hash,
            NULL,
            NULL,
            NULL
        );
        
        RAISE NOTICE '✅ Utilisateur créé avec succès !';
        RAISE NOTICE '   Username: admin';
        RAISE NOTICE '   Password: admin123';
    END IF;
END $$;
