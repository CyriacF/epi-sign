@echo off
REM Script pour créer un utilisateur de base
REM Usage: scripts\create_user.bat

set SCRIPT_DIR=%~dp0
set PROJECT_ROOT=%SCRIPT_DIR%\..

echo 🔧 Création de l'utilisateur de base...

REM Méthode 1: Avec Docker (recommandé)
docker compose ps db >nul 2>&1
if %errorlevel% == 0 (
    echo 📦 Utilisation de Docker...
    docker compose exec -T db psql -U postgres -d postgres < "%SCRIPT_DIR%create_user.sql"
    exit /b %errorlevel%
)

REM Méthode 2: Avec psql local
where psql >nul 2>&1
if %errorlevel% == 0 (
    echo 💻 Utilisation de psql local...
    
    REM Vérifier que DATABASE_URL est défini
    if "%DATABASE_URL%"=="" (
        if exist "%PROJECT_ROOT%\backend\.env" (
            for /f "usebackq tokens=1,* delims==" %%a in ("%PROJECT_ROOT%\backend\.env") do (
                if not "%%a"=="" if not "%%a"=="#" set %%a=%%b
            )
        ) else (
            echo ❌ Erreur: DATABASE_URL n'est pas défini
            exit /b 1
        )
    )
    
    psql "%DATABASE_URL%" -f "%SCRIPT_DIR%create_user.sql"
    exit /b %errorlevel%
)

REM Méthode 3: Avec cargo
where cargo >nul 2>&1
if %errorlevel% == 0 (
    echo 🦀 Utilisation de cargo...
    cd /d "%PROJECT_ROOT%\backend"
    
    if "%DATABASE_URL%"=="" (
        if exist .env (
            for /f "usebackq tokens=1,* delims==" %%a in (".env") do (
                if not "%%a"=="" if not "%%a"=="#" set %%a=%%b
            )
        ) else (
            echo ❌ Erreur: DATABASE_URL n'est pas défini
            exit /b 1
        )
    )
    
    cargo run --bin create_user
    exit /b %errorlevel%
)

echo ❌ Erreur: Aucune méthode disponible (Docker, psql ou cargo)
exit /b 1
