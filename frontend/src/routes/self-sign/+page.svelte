<script lang="ts">
  import { onMount } from "svelte";
  import { signUsers } from "$lib/api";
  import { currentUser } from "$lib/stores";
  import { isMobileDevice } from "$lib/utils/device";
  import type { ApiError, UserSignResponse } from "$lib/types";
  import QRScanner from "$lib/components/QRScanner.svelte";
  import SignResults from "$lib/components/SignResults.svelte";
  import AlertMessage from "$lib/components/AlertMessage.svelte";
  import { ScanQrCode, ArrowLeft } from "@lucide/svelte";
  import { fly, scale } from "svelte/transition";
  import { quintOut } from "svelte/easing";
  import type { PageData } from "./$types";
  import { goto } from "$app/navigation";

  let showScanner = false;
  let error = "";
  let success = "";
  let signing = false;
  let showResults = false;
  let signResults: UserSignResponse[] = [];
  let isMobile = false;

  onMount(() => {
    // Vérifier si on est sur mobile
    isMobile = isMobileDevice();
  });

  function handleScanRequest() {
    error = "";
    success = "";
    showScanner = true;
  }

  function handleCloseScanner() {
    showScanner = false;
  }

  async function handleScanResult(result: string) {
    // Vérifier si c'est une URL Epitech valide
    if (!result.includes("intra.epitech.eu")) {
      error = "QR Code invalide - URL Epitech attendue";
      return;
    }

    if (!$currentUser?.id) {
      error = "Utilisateur non trouvé";
      return;
    }

    // Fermer le scanner
    showScanner = false;

    // Commencer la signature
    signing = true;
    error = "";
    success = "";

    try {
      // Appeler l'API avec uniquement l'ULID de l'utilisateur actuel
      const response = await signUsers([$currentUser.id], result);

      signResults = response;
      showResults = true;

      // Message de succès si la signature a réussi
      const myResult = response.find((r) => r.ulid === $currentUser?.id);
      if (myResult?.response === "success") {
        success = "Signature réussie !";
      } else if (myResult?.response === "alreadySigned") {
        success = "Vous êtes déjà signé pour cet événement";
      }
    } catch (e) {
      const apiError = e as ApiError;
      if (apiError.status === 404) {
        error =
          "Aucun cookie trouvé pour aujourd'hui. Veuillez vous connecter à l'intranet Epitech.";
      } else if (apiError.status === 400) {
        error = "Données invalides";
      } else if (apiError.status === 401) {
        error = "Non autorisé - Vérifiez votre JWT";
      } else {
        error = "Erreur lors de la signature";
      }
    } finally {
      signing = false;
    }
  }

  function handleScanError(errorMsg: string) {
    error = errorMsg;
  }

  function handleCloseResults() {
    showResults = false;
    signResults = [];
  }

  function goBack() {
    goto("/");
  }

  // Vérifier le statut JWT de l'utilisateur actuel
  $: jwtStatus = (() => {
    if (!$currentUser?.jwtExpiresAt) return "missing";
    const expiresAt = new Date($currentUser.jwtExpiresAt);
    return expiresAt > new Date() ? "valid" : "expired";
  })();

  $: canSign = jwtStatus === "valid";
</script>

<div class="min-h-screen pb-safe">
  <div class="px-4 max-w-lg mx-auto">
    <!-- En-tête avec navigation -->
    <div
      class="flex items-center justify-start -ml-4 my-3 sm:mb-8"
      in:fly={{ y: -20, duration: 400, easing: quintOut }}
    >
      <button
        on:click={goBack}
        class="p-2 rounded-xl hover:bg-white/10 transition-all duration-200 ease-out transform hover:scale-110 active:scale-95"
        aria-label="Retour"
      >
        <ArrowLeft class="w-5 h-5 sm:w-6 sm:h-6" />
      </button>
    </div>

    <!-- Messages d'état -->
    {#if error}
      <AlertMessage type="error" message={error} />
    {/if}

    {#if success}
      <AlertMessage type="success" message={success} />
    {/if}

    <!-- Statut JWT -->
    <div
      class="mb-6 sm:mb-8"
      in:scale={{ duration: 400, delay: 100, easing: quintOut }}
    >
      <div class="glass-effect-card rounded-xl p-4 sm:p-6">
        <div class="flex items-center gap-3 mb-4">
          <div
            class="w-10 h-10 rounded-xl bg-white text-gray-900 flex items-center justify-center text-lg font-bold"
          >
            {$currentUser?.username?.charAt(0).toUpperCase() || "?"}
          </div>
          <div>
            <p class="font-semibold text-white">
              {$currentUser?.username || "Utilisateur"}
            </p>
            <p class="text-sm text-gray-400">Utilisateur actuel</p>
          </div>
        </div>

        <div class="flex items-center gap-2">
          <div
            class="w-3 h-3 rounded-full {canSign
              ? 'bg-green-400'
              : 'bg-red-400'} animate-pulse"
          ></div>
          <span class="text-sm {canSign ? 'text-green-400' : 'text-red-400'}">
            {#if jwtStatus === "valid"}
              JWT valide - Prêt à signer
            {:else if jwtStatus === "expired"}
              JWT expiré - Veuillez le mettre à jour
            {:else}
              JWT manquant - Veuillez le configurer
            {/if}
          </span>
        </div>
      </div>
    </div>

    <!-- Bouton principal de scan -->
    <div
      class="text-center"
      in:scale={{ duration: 500, delay: 200, easing: quintOut }}
    >
      <button
        on:click={handleScanRequest}
        disabled={!canSign || signing}
        class="w-full glass-effect-card rounded-2xl p-8 sm:p-12 hover:scale-[1.02] active:scale-[0.98] transition-all duration-200 ease-out group disabled:opacity-50 disabled:cursor-not-allowed disabled:hover:scale-100"
      >
        <div class="flex flex-col items-center gap-4">
          <div class="relative">
            <div
              class="w-20 h-20 sm:w-24 sm:h-24 rounded-3xl bg-white flex items-center justify-center group-hover:scale-110 group-disabled:hover:scale-100 transition-transform duration-200 ease-out"
            >
              {#if signing}
                <span
                  class="inline-block animate-spin rounded-full h-8 w-8 sm:h-10 sm:w-10 border-b-2 border-black"
                ></span>
              {:else}
                <ScanQrCode class="w-10 h-10 sm:w-12 sm:h-12 text-gray-900" />
              {/if}
            </div>
            {#if canSign && !signing}
              <div
                class="absolute -inset-2 rounded-3xl border-2 border-red-600/30 animate-pulse"
              ></div>
            {/if}
          </div>

          <div>
            <h3 class="text-xl sm:text-2xl font-bold text-white mb-2">
              {#if signing}
                Signature en cours...
              {:else if !canSign}
                JWT requis
              {:else}
                Scanner QR Code
              {/if}
            </h3>
            <p class="text-sm sm:text-base text-gray-400 leading-relaxed">
              {#if signing}
                Traitement de votre signature
              {:else if !canSign}
                Configurez votre JWT dans le profil
              {:else}
                Scannez le QR code de l'événement Epitech pour vous signer
                automatiquement
              {/if}
            </p>
          </div>
        </div>
      </button>
    </div>

    <!-- Instructions -->
    {#if canSign}
      <div
        class="mt-6 sm:mt-8"
        in:fly={{ y: 20, duration: 400, delay: 300, easing: quintOut }}
      >
        <div class="glass-effect-card rounded-xl p-4 sm:p-6">
          <h4 class="font-semibold text-white mb-3 flex items-center gap-2">
            <span class="text-lg">💡</span>
            Comment ça marche ?
          </h4>
          <ol class="space-y-2 text-sm text-gray-300">
            <li class="flex gap-3">
              <span
                class="flex-shrink-0 w-6 h-6 bg-red-600/10 text-red-500 rounded-full flex items-center justify-center text-xs font-bold"
                >1</span
              >
              <span>Appuyez sur "Scanner QR Code"</span>
            </li>
            <li class="flex gap-3">
              <span
                class="flex-shrink-0 w-6 h-6 bg-red-600/10 text-red-500 rounded-full flex items-center justify-center text-xs font-bold"
                >2</span
              >
              <span>Scannez le QR code sur l'intranet Epitech</span>
            </li>
            <li class="flex gap-3">
              <span
                class="flex-shrink-0 w-6 h-6 bg-red-600/10 text-red-500 rounded-full flex items-center justify-center text-xs font-bold"
                >3</span
              >
              <span>Votre signature sera effectuée automatiquement</span>
            </li>
          </ol>
        </div>
      </div>
    {/if}
  </div>
</div>

<!-- Scanner QR Code -->
{#if showScanner}
  <QRScanner
    onScan={handleScanResult}
    onError={handleScanError}
    onClose={handleCloseScanner}
  />
{/if}

<!-- Résultats de signature -->
<SignResults
  isOpen={showResults}
  results={signResults}
  users={$currentUser
    ? [
        {
          id: $currentUser.id,
          username: $currentUser.username,
          jwtExpiresAt: $currentUser.jwtExpiresAt,
        },
      ]
    : []}
  on:close={handleCloseResults}
/>
