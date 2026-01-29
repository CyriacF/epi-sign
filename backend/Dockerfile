# ─── Builder (all on Debian Bookworm) ────────────────────────────────────────────
FROM rust:1.87.0-bookworm AS builder

RUN apt-get update && \
    apt-get install -y \
    build-essential \
    pkg-config \
    libpq-dev \
    libssl-dev \
    && rm -rf /var/lib/apt/lists/*

    
    # Install diesel_cli (Postgres) for the GNU target
    RUN cargo install \
    diesel_cli \
    --no-default-features \
    --features postgres \
    --root /app/diesel-install
    
WORKDIR /app

# Copy manifests first (to cache deps)
COPY Cargo.toml Cargo.lock ./

COPY src/ ./src/

# Build the Linux server (default x86_64-unknown-linux-gnu)
RUN cargo build --release



# ─── Runner (Debian Bookworm-slim) ───────────────────────────────────────────────
FROM debian:bookworm-slim AS runner

RUN apt-get update && \
    apt-get install -y \
    libpq5 \
    openssl \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/* \
    && apt-get clean

WORKDIR /app

COPY migrations ./migrations
COPY diesel.toml ./
COPY --from=builder /app/target/release/epi-sign-backend ./server
COPY --from=builder /app/diesel-install/bin/diesel /usr/local/bin/diesel

# Create startup script
RUN echo '#!/bin/bash\n\
set -e\n\
\n\
# Check if this is the first run by looking for a marker file\n\
if [ ! -f /app/.migrations-complete ]; then\n\
    echo "Running database migrations..."\n\
    \n\
    # Fix: Drop edsquare_cookies table if it exists without primary key\n\
    # This prevents diesel from failing when reading the schema\n\
    echo "Checking for problematic edsquare_cookies table..."\n\
    if [ -n "$DATABASE_URL" ]; then\n\
        # Drop the problematic table\n\
        psql "$DATABASE_URL" -c "DROP TABLE IF EXISTS edsquare_cookies CASCADE;" 2>/dev/null || true\n\
        \n\
        # Remove migration entries for edsquare_cookies migrations to force re-execution\n\
        psql "$DATABASE_URL" -c "DELETE FROM __diesel_schema_migrations WHERE version LIKE '\''2026-01-23-142642%'\'' OR version LIKE '\''2026-01-23-150000%'\'' OR version LIKE '\''2026-01-23-142640%'\'';" 2>/dev/null || true\n\
        \n\
        echo "Cleaned up edsquare_cookies table and migration entries"\n\
    fi\n\
    \n\
    # Run migrations manually with psql to avoid diesel schema reading issues\n\
    echo "Running migrations with psql..."\n\
    \n\
    # Ensure __diesel_schema_migrations table exists\n\
    psql "$DATABASE_URL" -c "CREATE TABLE IF NOT EXISTS __diesel_schema_migrations (version VARCHAR(50) PRIMARY KEY, run_on TIMESTAMP NOT NULL DEFAULT NOW());" 2>/dev/null || true\n\
    \n\
    # Get list of migrations in order (sorted by name)\n\
    for migration_dir in $(ls -d /app/migrations/*/ | sort); do\n\
        if [ -f "$migration_dir/up.sql" ]; then\n\
            migration_name=$(basename "$migration_dir")\n\
            \n\
            # Skip diesel initial setup\n\
            if [ "$migration_name" = "00000000000000_diesel_initial_setup" ]; then\n\
                continue\n\
            fi\n\
            \n\
            # Check if migration has already been run\n\
            if ! psql "$DATABASE_URL" -tAc "SELECT 1 FROM __diesel_schema_migrations WHERE version = '\''$migration_name'\''" 2>/dev/null | grep -q 1; then\n\
                echo "Running migration: $migration_name"\n\
                if psql "$DATABASE_URL" -f "$migration_dir/up.sql" 2>&1; then\n\
                    # Mark migration as run\n\
                    psql "$DATABASE_URL" -c "INSERT INTO __diesel_schema_migrations (version) VALUES ('\''$migration_name'\'') ON CONFLICT DO NOTHING;" 2>/dev/null || true\n\
                    echo "Migration $migration_name completed successfully"\n\
                else\n\
                    echo "ERROR: Migration $migration_name failed"\n\
                    exit 1\n\
                fi\n\
            else\n\
                echo "Migration $migration_name already run, skipping"\n\
            fi\n\
        fi\n\
    done\n\
    \n\
    echo "All migrations completed"\n\
    \n\
    # Create marker file to indicate migrations have been run\n\
    touch /app/.migrations-complete\n\
    echo "Migrations completed successfully"\n\
else\n\
    echo "Migrations already completed, skipping..."\n\
fi\n\
\n\
# Start the server\n\
exec ./server' > /app/start.sh

RUN chmod +x /app/start.sh server /usr/local/bin/diesel

EXPOSE 80

CMD ["/app/start.sh"]