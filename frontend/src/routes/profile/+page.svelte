<script lang="ts">
  import { onMount } from "svelte";
  import { currentUser } from "$lib/stores";
  import { updateUserProfile, saveSignature, loginEdsquare, getCurrentUser, getEdsquareStatus } from "$lib/api";
  import type { ApiError, LoginEdsquareResponse, EdsquareStatusResponse } from "$lib/types";
  import { fly, fade, scale } from "svelte/transition";
  import { quintOut } from "svelte/easing";
  import { goto } from "$app/navigation";
  import { ArrowLeft } from "@lucide/svelte";
  import SignatureCanvas from "$lib/components/SignatureCanvas.svelte";
  import AlertMessage from "$lib/components/AlertMessage.svelte";

  let username: string = "";
  let oldPassword: string = "";
  let newPassword: string = "";
  let confirmPassword: string = "";
  let loading: boolean = false;
  let error: string = "";
  let success: string = "";
  let showPasswords: boolean = false;
  let showSignatureCanvas: boolean = false;
  let signatureImage: string | null = null;

  // EDSquare login
  let showEdsquareLogin = false;
  let loggingIn = false;
  let emailInput = "";
  let passwordInput = "";
  let edsquareError = "";
  let edsquareSuccess = "";
  let edsquareStatus: EdsquareStatusResponse | null = null;

  // Initialize form + statut EDSquare
  onMount(async () => {
    if ($currentUser) {
      username = $currentUser.username || "";
      signatureImage = $currentUser.signatureManuscrite || null;
    }
    try {
      edsquareStatus = await getEdsquareStatus();
    } catch {
      edsquareStatus = null;
    }
  });

  // Update form when currentUser changes
  $: if ($currentUser) {
    username = $currentUser.username || "";
    signatureImage = $currentUser.signatureManuscrite || null;
  }

  async function handleSubmit() {
    error = "";
    success = "";

    if (!username.trim()) {
      error = "Le nom d'utilisateur ne peut pas être vide";
      return;
    }

    if (newPassword && newPassword !== confirmPassword) {
      error = "Les mots de passe ne correspondent pas";
      return;
    }

    if (newPassword && newPassword.length < 8) {
      error = "Le nouveau mot de passe doit contenir au moins 8 caractères";
      return;
    }

    if (newPassword && !oldPassword) {
      error = "Veuillez entrer votre mot de passe actuel pour le modifier";
      return;
    }

    loading = true;

    try {
      const payload: any = {};

      if (username.trim() !== $currentUser?.username) {
        payload.username = username.trim();
      }

      if (newPassword) {
        payload.old_password = oldPassword;
        payload.new_password = newPassword;
      }

      if (Object.keys(payload).length === 0) {
        error = "Aucune modification détectée";
        loading = false;
        return;
      }

      await updateUserProfile(payload);
      success = "Profil mis à jour avec succès !";
      
      if (payload.username) {
        currentUser.update((user) =>
          user ? { ...user, username: payload.username } : user
        );
      }

      // Reset password fields
      oldPassword = "";
      newPassword = "";
      confirmPassword = "";
    } catch (e) {
      const apiError = e as ApiError;
      if (apiError.status === 400) {
        error = "Données invalides";
      } else if (apiError.status === 401) {
        error = "Mot de passe actuel incorrect";
      } else if (apiError.status === 404) {
        error = "Utilisateur non trouvé";
      } else {
        error = "Erreur lors de la mise à jour du profil";
      }
    } finally {
      loading = false;
    }
  }

  function togglePasswordVisibility() {
    showPasswords = !showPasswords;
  }

  async function handleSignatureSave(event: CustomEvent<string>) {
    const signatureDataUrl = event.detail;
    signatureImage = signatureDataUrl;
    showSignatureCanvas = false;
    
    try {
      await saveSignature(signatureDataUrl);
      success = "Signature sauvegardée avec succès !";
      // Recharger l'utilisateur pour mettre à jour le store
      const user = await getCurrentUser();
      currentUser.set(user);
      signatureImage = user.signatureManuscrite || null;
    } catch (e) {
      const apiError = e as ApiError;
      error = "Erreur lors de l'enregistrement de la signature";
    }
  }

  function openSignatureCanvas() {
    showSignatureCanvas = true;
  }

  async function loginToEdsquare() {
    if (!emailInput.trim() || !passwordInput.trim()) {
      edsquareError = "Veuillez entrer votre email et mot de passe EDSquare";
      return;
    }

    loggingIn = true;
    edsquareError = "";
    edsquareSuccess = "";

    try {
      const response = await loginEdsquare(emailInput.trim(), passwordInput.trim());
      if (response.success) {
        edsquareSuccess = response.message || "Connexion EDSquare réussie ! Les cookies ont été sauvegardés.";
        emailInput = "";
        passwordInput = "";
        showEdsquareLogin = false;
        try {
          edsquareStatus = await getEdsquareStatus();
        } catch {
          edsquareStatus = null;
        }
      } else {
        edsquareError = response.message || "Erreur lors de la connexion";
      }
    } catch (e) {
      const apiError = e as ApiError;
      // Le backend peut retourner une erreur dans le message même avec status 200
      if (apiError.message && (apiError.message.includes("invalide") || apiError.message.includes("échec") || apiError.message.includes("Échec"))) {
        edsquareError = apiError.message;
      } else if (apiError.status === 400) {
        edsquareError = "Identifiants invalides. Vérifiez votre email et mot de passe.";
      } else if (apiError.status === 401) {
        edsquareError = "Non autorisé - Vérifiez votre connexion";
      } else if (apiError.status === 500) {
        // Le backend peut retourner 500 avec un message d'erreur dans le body
        edsquareError = apiError.message || "Erreur lors de la connexion à EDSquare";
      } else {
        edsquareError = apiError.message || "Erreur lors de la connexion à EDSquare";
      }
    } finally {
      loggingIn = false;
    }
  }

  function goBack() {
    goto("/");
  }
</script>

<div class="min-h-screen bg-gradient-to-br from-gray-900 via-gray-800 to-gray-900 p-4 sm:p-6">
  <div class="max-w-4xl mx-auto">
    <!-- Header -->
    <div
      class="flex items-center gap-4 mb-6"
      in:fly={{ y: -20, duration: 400, easing: quintOut }}
    >
      <button
        on:click={goBack}
        class="p-2 rounded-lg bg-white/10 hover:bg-white/20 transition-all duration-200 ease-out transform hover:scale-110 active:scale-95"
        aria-label="Retour"
      >
        <ArrowLeft class="w-6 h-6" />
      </button>
      <h1 class="text-3xl font-bold gradient-text">Mon Profil</h1>
    </div>

    <!-- Alerts -->
    {#if error}
      <div
        in:fly={{ x: -50, duration: 300, easing: quintOut }}
        out:fly={{ x: 50, duration: 200, easing: quintOut }}
      >
        <AlertMessage
          message={error}
          type="error"
          on:close={() => (error = "")}
        />
      </div>
    {/if}

    {#if success}
      <div
        in:fly={{ x: -50, duration: 300, easing: quintOut }}
        out:fly={{ x: 50, duration: 200, easing: quintOut }}
      >
        <AlertMessage
          message={success}
          type="success"
          on:close={() => (success = "")}
        />
      </div>
    {/if}

    <div class="grid grid-cols-1 lg:grid-cols-2 gap-6">
      <!-- Informations du profil -->
      <div
        class="glass-effect-card rounded-xl p-6 sm:p-8"
        in:fly={{ y: 20, duration: 400, delay: 100, easing: quintOut }}
      >
        <h2 class="text-xl font-semibold gradient-text mb-6">Informations personnelles</h2>

        <form on:submit|preventDefault={handleSubmit} class="space-y-6">
          <!-- Username -->
          <div>
            <label
              for="username"
              class="block text-sm font-medium text-gray-300 mb-2"
            >
              Nom d'utilisateur
            </label>
            <input
              type="text"
              id="username"
              bind:value={username}
              disabled={loading}
              placeholder="Votre nom d'utilisateur"
              class="input-field w-full"
            />
          </div>

          <!-- Password Section -->
          <div class="space-y-4">
            <div class="flex items-center justify-between">
              <h3 class="text-sm font-medium text-gray-300">
                Modifier le mot de passe
              </h3>
              <span class="text-xs text-gray-500">(optionnel)</span>
            </div>

            <!-- Current Password -->
            <div>
              <label
                for="oldPassword"
                class="block text-sm font-medium text-gray-300 mb-2"
              >
                Mot de passe actuel
              </label>
              <div class="relative">
                <input
                  type={showPasswords ? "text" : "password"}
                  id="oldPassword"
                  bind:value={oldPassword}
                  disabled={loading}
                  placeholder="Votre mot de passe actuel"
                  class="input-field w-full pr-12"
                />
                <button
                  type="button"
                  on:click={togglePasswordVisibility}
                  class="absolute right-3 top-1/2 transform -translate-y-1/2 p-2 rounded-lg hover:bg-white/10"
                  aria-label={showPasswords ? "Masquer" : "Afficher"}
                >
                  {#if showPasswords}
                    <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21" />
                    </svg>
                  {:else}
                    <svg class="w-5 h-5 text-gray-400" fill="none" stroke="currentColor" viewBox="0 0 24 24">
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M15 12a3 3 0 11-6 0 3 3 0 016 0z" />
                      <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z" />
                    </svg>
                  {/if}
                </button>
              </div>
            </div>

            <!-- New Password -->
            <div>
              <label
                for="newPassword"
                class="block text-sm font-medium text-gray-300 mb-2"
              >
                Nouveau mot de passe
              </label>
              <input
                type={showPasswords ? "text" : "password"}
                id="newPassword"
                bind:value={newPassword}
                disabled={loading}
                placeholder="Nouveau mot de passe (min. 8 caractères)"
                class="input-field w-full"
              />
            </div>

            <!-- Confirm Password -->
            <div>
              <label
                for="confirmPassword"
                class="block text-sm font-medium text-gray-300 mb-2"
              >
                Confirmer le nouveau mot de passe
              </label>
              <input
                type={showPasswords ? "text" : "password"}
                id="confirmPassword"
                bind:value={confirmPassword}
                disabled={loading}
                placeholder="Confirmez le nouveau mot de passe"
                class="input-field w-full"
              />
            </div>
          </div>

          <button
            type="submit"
            disabled={loading}
            class="btn-primary w-full"
          >
            {#if loading}
              <span class="inline-block animate-spin rounded-full h-5 w-5 border-b-2 border-white"></span>
            {:else}
              Mettre à jour le profil
            {/if}
          </button>
        </form>
      </div>

      <!-- Signature et EDSquare -->
      <div class="space-y-6">
        <!-- Signature manuscrite -->
        <div
          class="glass-effect-card rounded-xl p-6 sm:p-8"
          in:fly={{ y: 20, duration: 400, delay: 200, easing: quintOut }}
        >
          <h2 class="text-xl font-semibold gradient-text mb-6">Signature manuscrite</h2>

          {#if signatureImage}
            <div class="bg-white/5 rounded-lg p-4 border border-white/10 mb-4">
              <p class="text-xs text-gray-400 mb-2">Signature actuelle :</p>
              <div class="flex justify-center items-center max-h-[120px] overflow-hidden">
                <img
                  src={signatureImage}
                  alt="Signature"
                  class="max-w-[250px] max-h-[100px] w-auto h-auto rounded border border-white/20 object-contain"
                />
              </div>
            </div>
          {/if}

          <button
            type="button"
            on:click={openSignatureCanvas}
            class="btn-secondary w-full"
          >
            {signatureImage ? "Modifier la signature" : "Créer une signature"}
          </button>
        </div>

        <!-- Connexion EDSquare -->
        <div
          class="glass-effect-card rounded-xl p-6 sm:p-8"
          in:fly={{ y: 20, duration: 400, delay: 300, easing: quintOut }}
        >
          <h2 class="text-xl font-semibold gradient-text mb-6">Connexion EDSquare</h2>

          {#if edsquareStatus !== null}
            <div
              class="mb-4 p-3 rounded-lg flex items-center gap-3 {edsquareStatus.has_saved_credentials
                ? 'bg-green-500/10 border border-green-500/30 text-green-300'
                : 'bg-gray-500/10 border border-gray-500/30 text-gray-400'}"
            >
              {#if edsquareStatus.has_saved_credentials}
                <svg class="w-5 h-5 text-green-400 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M9 12l2 2 4-4m6 2a9 9 0 11-18 0 9 9 0 0118 0z" />
                </svg>
                <span class="text-sm font-medium">Identifiants EDSquare enregistrés</span>
                <span class="text-xs opacity-80">— reconnexion automatique en cas d’expiration</span>
              {:else}
                <svg class="w-5 h-5 text-gray-500 shrink-0" fill="none" stroke="currentColor" viewBox="0 0 24 24" aria-hidden="true">
                  <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M12 15v2m-6 4h12a2 2 0 002-2v-6a2 2 0 00-2-2H6a2 2 0 00-2 2v6a2 2 0 002 2zm10-10V7a4 4 0 00-8 0v4h8z" />
                </svg>
                <span class="text-sm">Aucun identifiant enregistré — connectez-vous une fois pour les sauvegarder.</span>
              {/if}
            </div>
          {/if}

          {#if edsquareError}
            <div class="mb-4">
              <AlertMessage
                message={edsquareError}
                type="error"
                on:close={() => (edsquareError = "")}
              />
            </div>
          {/if}

          {#if edsquareSuccess}
            <div class="mb-4">
              <AlertMessage
                message={edsquareSuccess}
                type="success"
                on:close={() => (edsquareSuccess = "")}
              />
            </div>
          {/if}

          {#if !showEdsquareLogin}
            <p class="text-sm text-gray-400 mb-4">
              Connectez-vous une fois à EDSquare : vos identifiants sont enregistrés. Ensuite, si la session expire, la reconnexion se fera automatiquement lors de la validation d’un code.
            </p>
            <button
              type="button"
              on:click={() => (showEdsquareLogin = true)}
              class="btn-primary w-full"
            >
              Se connecter à EDSquare
            </button>
          {:else}
            <div class="space-y-4">
              <div>
                <label for="edsquareEmail" class="block text-sm font-medium text-gray-300 mb-2">
                  Email EDSquare
                </label>
                <input
                  type="email"
                  id="edsquareEmail"
                  bind:value={emailInput}
                  disabled={loggingIn}
                  placeholder="votre@email.com"
                  class="input-field w-full"
                />
              </div>

              <div>
                <label for="edsquarePassword" class="block text-sm font-medium text-gray-300 mb-2">
                  Mot de passe EDSquare
                </label>
                <input
                  type="password"
                  id="edsquarePassword"
                  bind:value={passwordInput}
                  disabled={loggingIn}
                  placeholder="••••••••"
                  class="input-field w-full"
                />
              </div>

              <div class="flex gap-3">
                <button
                  type="button"
                  on:click={() => {
                    showEdsquareLogin = false;
                    emailInput = "";
                    passwordInput = "";
                    edsquareError = "";
                    edsquareSuccess = "";
                  }}
                  disabled={loggingIn}
                  class="btn-secondary flex-1"
                >
                  Annuler
                </button>
                <button
                  type="button"
                  on:click={loginToEdsquare}
                  disabled={loggingIn || !emailInput.trim() || !passwordInput.trim()}
                  class="btn-primary flex-1"
                >
                  {#if loggingIn}
                    <span class="inline-block animate-spin rounded-full h-5 w-5 border-b-2 border-white"></span>
                  {:else}
                    Se connecter
                  {/if}
                </button>
              </div>
            </div>
          {/if}
        </div>
      </div>
    </div>
  </div>
</div>

<!-- Signature Canvas Modal -->
<SignatureCanvas
  isOpen={showSignatureCanvas}
  currentSignature={signatureImage}
  on:save={handleSignatureSave}
  on:close={() => (showSignatureCanvas = false)}
/>
