<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { currentUser } from "$lib/stores";
  import { updateUserProfile, saveSignature } from "$lib/api";
  import type { ApiError } from "$lib/types";
  import { fly, fade, scale } from "svelte/transition";
  import { quintOut } from "svelte/easing";
  import SignatureCanvas from "./SignatureCanvas.svelte";

  export let isOpen: boolean = false;

  const dispatch = createEventDispatcher();

  let username: string = "";
  let oldPassword: string = "";
  let newPassword: string = "";
  let confirmPassword: string = "";
  let loading: boolean = false;
  let error: string = "";
  let success: boolean = false;
  let showPasswords: boolean = false;
  let showSignatureCanvas: boolean = false;
  let signatureImage: string | null = null;

  // Reset form when opening
  $: if (isOpen) {
    username = $currentUser?.username || "";
    oldPassword = "";
    newPassword = "";
    confirmPassword = "";
    error = "";
    success = false;
    showPasswords = false;
    signatureImage = $currentUser?.signatureManuscrite || null;
  }

  async function handleSubmit() {
    error = "";
    success = false;

    // Validation
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

      // Only include username if it changed
      if (username.trim() !== $currentUser?.username) {
        payload.username = username.trim();
      }

      // Only include password fields if changing password
      if (newPassword) {
        payload.old_password = oldPassword;
        payload.new_password = newPassword;
      }

      // If no changes, show error
      if (Object.keys(payload).length === 0) {
        error = "Aucune modification détectée";
        loading = false;
        return;
      }

      await updateUserProfile(payload);
      success = true;

      // Update the current user store with new username if changed
      if (payload.username) {
        currentUser.update((user) =>
          user ? { ...user, username: payload.username } : user
        );
      }

      // Close after 2 seconds of success
      setTimeout(() => {
        handleClose();
      }, 2000);
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

  function handleClose() {
    username = "";
    oldPassword = "";
    newPassword = "";
    confirmPassword = "";
    error = "";
    success = false;
    showPasswords = false;
    dispatch("close");
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
      success = true;
      setTimeout(() => {
        handleClose();
      }, 2000);
    } catch (e) {
      const apiError = e as ApiError;
      error = "Erreur lors de l'enregistrement de la signature";
    }
  }

  function openSignatureCanvas() {
    showSignatureCanvas = true;
  }

  // Fermer avec Escape
  function handleKeydown(event: KeyboardEvent) {
    if (event.key === "Escape" && isOpen) {
      handleClose();
    }
  }
</script>

<svelte:window on:keydown={handleKeydown} />

{#if isOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4"
    in:fade={{ duration: 300, easing: quintOut }}
    out:fade={{ duration: 200, easing: quintOut }}
  >
    <!-- Backdrop -->
    <button
      class="absolute inset-0 bg-black/50 backdrop-blur-sm"
      on:click={handleClose}
      in:fade={{ duration: 300 }}
      out:fade={{ duration: 200 }}
      aria-label="Fermer le modal"
    ></button>

    <!-- Modal -->
    <div
      class="relative w-full max-w-lg glass-effect-modal rounded-2xl p-6 sm:p-8 shadow-2xl"
      in:scale={{
        duration: 400,
        easing: quintOut,
        start: 0.9,
        opacity: 0,
      }}
      out:scale={{
        duration: 200,
        easing: quintOut,
        start: 0.95,
        opacity: 0,
      }}
    >
      <!-- Header -->
      <div
        class="flex justify-between items-start mb-6"
        in:fly={{ y: -20, duration: 400, delay: 100, easing: quintOut }}
      >
        <div>
          <h2 class="text-2xl font-bold gradient-text">Modifier le profil</h2>
          <p class="text-sm text-gray-400 mt-2">
            Modifiez votre nom d'utilisateur et/ou mot de passe
          </p>
        </div>
        <button
          on:click={handleClose}
          class="p-2 rounded-lg hover:bg-white/10 transition-all duration-200 ease-out transform hover:scale-110 active:scale-95 hover:rotate-90"
          aria-label="Fermer"
        >
          <svg
            class="w-6 h-6"
            fill="none"
            stroke="currentColor"
            viewBox="0 0 24 24"
          >
            <path
              stroke-linecap="round"
              stroke-linejoin="round"
              stroke-width="2"
              d="M6 18L18 6M6 6l12 12"
            />
          </svg>
        </button>
      </div>

      <!-- Alerts -->
      {#if error}
        <div
          class="mb-6 p-4 bg-red-500/10 border border-red-500/50 rounded-xl text-red-400 text-sm"
          in:fly={{ x: -50, duration: 300, easing: quintOut }}
          out:fly={{ x: 50, duration: 200, easing: quintOut }}
        >
          ❌ {error}
        </div>
      {/if}

      {#if success}
        <div
          class="mb-6 p-4 bg-green-500/10 border border-green-500/50 rounded-xl text-green-400 text-sm"
          in:scale={{ duration: 400, easing: quintOut }}
          out:scale={{ duration: 200, easing: quintOut }}
        >
          ✅ Profil mis à jour avec succès !
        </div>
      {/if}

      <!-- Form -->
      <form
        on:submit|preventDefault={handleSubmit}
        class="space-y-6"
        in:fly={{ y: 20, duration: 400, delay: 200, easing: quintOut }}
      >
        <!-- Username -->
        <div
          class="transform transition-all duration-200 ease-out hover:scale-[1.02]"
        >
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
            disabled={loading || success}
            placeholder="Votre nom d'utilisateur"
            class="input-field transition-all duration-200 ease-out focus:scale-[1.02]"
          />
        </div>

        <!-- Password Section -->
        <div class="space-y-4">
          <div
            class="flex items-center justify-between"
            in:fade={{ delay: 300, duration: 200 }}
          >
            <h3 class="text-sm font-medium text-gray-300">
              Modifier le mot de passe
            </h3>
            <span class="text-xs text-gray-500">(optionnel)</span>
          </div>

          <!-- Current Password -->
          <div
            class="transform transition-all duration-200 ease-out hover:scale-[1.02]"
            in:fly={{ x: -20, duration: 300, delay: 350, easing: quintOut }}
          >
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
                disabled={loading || success}
                placeholder="Votre mot de passe actuel"
                class="input-field pr-12 transition-all duration-200 ease-out focus:scale-[1.02]"
              />
              <button
                type="button"
                on:click={togglePasswordVisibility}
                class="absolute right-3 top-1/2 transform -translate-y-1/2 p-2 rounded-lg hover:bg-white/10 transition-all duration-200 ease-out hover:scale-110 active:scale-95"
                aria-label={showPasswords ? "Masquer" : "Afficher"}
              >
                {#if showPasswords}
                  <svg
                    class="w-5 h-5 text-gray-400 transition-transform duration-300 ease-out"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                    in:scale={{ duration: 200 }}
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M13.875 18.825A10.05 10.05 0 0112 19c-4.478 0-8.268-2.943-9.543-7a9.97 9.97 0 011.563-3.029m5.858.908a3 3 0 114.243 4.243M9.878 9.878l4.242 4.242M9.88 9.88l-3.29-3.29m7.532 7.532l3.29 3.29M3 3l3.59 3.59m0 0A9.953 9.953 0 0112 5c4.478 0 8.268 2.943 9.543 7a10.025 10.025 0 01-4.132 5.411m0 0L21 21"
                    />
                  </svg>
                {:else}
                  <svg
                    class="w-5 h-5 text-gray-400 transition-transform duration-300 ease-out"
                    fill="none"
                    stroke="currentColor"
                    viewBox="0 0 24 24"
                    in:scale={{ duration: 200 }}
                  >
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M15 12a3 3 0 11-6 0 3 3 0 016 0z"
                    />
                    <path
                      stroke-linecap="round"
                      stroke-linejoin="round"
                      stroke-width="2"
                      d="M2.458 12C3.732 7.943 7.523 5 12 5c4.478 0 8.268 2.943 9.542 7-1.274 4.057-5.064 7-9.542 7-4.477 0-8.268-2.943-9.542-7z"
                    />
                  </svg>
                {/if}
              </button>
            </div>
          </div>

          <!-- New Password -->
          <div
            class="transform transition-all duration-200 ease-out hover:scale-[1.02]"
            in:fly={{ x: 20, duration: 300, delay: 400, easing: quintOut }}
          >
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
              disabled={loading || success}
              placeholder="Nouveau mot de passe (min. 8 caractères)"
              class="input-field transition-all duration-200 ease-out focus:scale-[1.02]"
            />
          </div>

          <!-- Confirm Password -->
          <div
            class="transform transition-all duration-200 ease-out hover:scale-[1.02]"
            in:fly={{ x: -20, duration: 300, delay: 450, easing: quintOut }}
          >
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
              disabled={loading || success}
              placeholder="Confirmez le nouveau mot de passe"
              class="input-field transition-all duration-200 ease-out focus:scale-[1.02]"
            />
          </div>
        </div>

        <!-- Signature manuscrite -->
        <div class="space-y-4">
          <div
            class="flex items-center justify-between"
            in:fade={{ delay: 500, duration: 200 }}
          >
            <h3 class="text-sm font-medium text-gray-300">
              Signature manuscrite (pour EDSquare)
            </h3>
            <span class="text-xs text-gray-500">(optionnel)</span>
          </div>

          {#if signatureImage}
            <div
              class="bg-white/5 rounded-lg p-4 border border-white/10 overflow-hidden"
              in:fly={{ x: -20, duration: 300, delay: 550, easing: quintOut }}
            >
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

          <div
            class="transform transition-all duration-200 ease-out hover:scale-[1.02]"
            in:fly={{ x: 20, duration: 300, delay: 600, easing: quintOut }}
          >
            <button
              type="button"
              on:click={openSignatureCanvas}
              disabled={loading || success}
              class="btn-secondary w-full transform transition-all duration-200 ease-out hover:scale-105 active:scale-95"
            >
              {signatureImage ? "Modifier la signature" : "Créer une signature"}
            </button>
          </div>
        </div>

        <!-- Actions -->
        <div
          class="flex gap-3"
          in:fly={{ y: 20, duration: 300, delay: 500, easing: quintOut }}
        >
          <button
            type="button"
            on:click={handleClose}
            disabled={loading}
            class="btn-secondary flex-1 transform transition-all duration-200 ease-out hover:scale-105 active:scale-95"
          >
            Annuler
          </button>
          <button
            type="submit"
            disabled={loading || success}
            class="btn-primary flex-1 transform transition-all duration-200 ease-out hover:scale-105 active:scale-95"
          >
            {#if loading}
              <span
                class="inline-block animate-spin rounded-full h-5 w-5 border-b-2 border-white"
                in:scale={{ duration: 300 }}
              ></span>
            {:else if success}
              <span in:scale={{ duration: 300, easing: quintOut }}
                >Mis à jour !</span
              >
            {:else}
              Mettre à jour
            {/if}
          </button>
        </div>
      </form>
    </div>
  </div>
{/if}

<!-- Signature Canvas Modal -->
<SignatureCanvas
  isOpen={showSignatureCanvas}
  currentSignature={signatureImage}
  on:save={handleSignatureSave}
  on:close={() => (showSignatureCanvas = false)}
/>
