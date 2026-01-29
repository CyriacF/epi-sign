use diesel::prelude::*;
use dotenvy::dotenv;
use std::env;

// Les binaires dans src/bin/ peuvent accéder au crate principal
// Mais schema est privé, donc on l'inclut directement
#[allow(dead_code)]
mod schema {
    include!("../schema.rs");
}

// Import des modules nécessaires
// On doit créer un petit wrapper pour accéder à User et hash_password
use sha2::{Digest, Sha256};

fn hash_password(password: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(password);
    let first_hash = hasher.finalize();
    let first_hash_hex = format!("{:x}", first_hash);

    let mut hasher2 = Sha256::new();
    hasher2.update(format!("{}{}", password, first_hash_hex));
    let result = hasher2.finalize();
    format!("{:x}", result)
}

#[derive(Insertable)]
#[diesel(table_name = schema::users)]
struct NewUser {
    id: String,
    username: String,
    password_hash: String,
    jwt_intra_epitech: Option<String>,
    jwt_expires_at: Option<chrono::NaiveDateTime>,
    signature_manuscrite: Option<String>,
}

fn main() {
    dotenv().ok();
    
    let database_url = env::var("DATABASE_URL")
        .expect("DATABASE_URL must be set");
    
    let mut conn = diesel::PgConnection::establish(&database_url)
        .unwrap_or_else(|_| panic!("Error connecting to {}", database_url));

    // Créer un utilisateur par défaut
    // Username: admin
    // Password: admin123
    let username = "admin";
    let password = "admin123";
    
    use schema::users;
    use ulid::Ulid;

    // Vérifier si l'utilisateur existe déjà
    let existing: Option<(String, String)> = users::table
        .select((users::id, users::username))
        .filter(users::username.eq(username))
        .first::<(String, String)>(&mut conn)
        .optional()
        .expect("Error checking for existing user");

    if existing.is_some() {
        println!("L'utilisateur '{}' existe déjà !", username);
        return;
    }

    // Créer le nouvel utilisateur
    let new_user = NewUser {
        id: Ulid::new().to_string(),
        username: username.to_string(),
        password_hash: hash_password(password),
        jwt_intra_epitech: None,
        jwt_expires_at: None,
        signature_manuscrite: None,
    };
    
    diesel::insert_into(users::table)
        .values(&new_user)
        .execute(&mut conn)
        .expect("Error creating user");

    println!("✅ Utilisateur créé avec succès !");
    println!("   Username: {}", username);
    println!("   Password: {}", password);
    println!("\n⚠️  N'oubliez pas de changer le mot de passe après la première connexion !");
}
