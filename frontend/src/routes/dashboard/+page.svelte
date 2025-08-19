<script lang="ts">
  import { onMount } from "svelte";
  import { signUsers } from "$lib/api";
  import { isMobileDevice } from "$lib/utils/device";
  import type {
    PublicUserResponse,
    ApiError,
    UserSignResponse,
    DashboardPageData,
  } from "$lib/types";
  import AlertMessage from "$lib/components/AlertMessage.svelte";
  import SigningSection from "$lib/components/SigningSection.svelte";
  import UsersList from "$lib/components/UsersList.svelte";
  import QRScanner from "$lib/components/QRScanner.svelte";
  import SignResults from "$lib/components/SignResults.svelte";
  import { ArrowLeft, HelpCircle } from "@lucide/svelte";
  import { fly, scale, fade } from "svelte/transition";
  import { quintOut } from "svelte/easing";
  import { goto } from "$app/navigation";

  export let data: DashboardPageData;

  let users: PublicUserResponse[] = data.users || [];
  let selectedUsers = new Set<string>();
  let signUrl: string = "";
  let error: string = data.error || "";
  let success: string = "";
  let signing: boolean = false;
  let selectAll: boolean = false;
  let showScanner: boolean = false;
  let isMobile: boolean = false;
  let showResults: boolean = false;
  let signResults: UserSignResponse[] = [];
  let showGuideModal = false;

  onMount(() => {
    // Vérifier si on est sur mobile
    isMobile = isMobileDevice();
  });

  async function handleSign(): Promise<void> {
    error = "";
    success = "";
    showResults = false;
    signResults = [];

    if (selectedUsers.size === 0) {
      error = "Sélectionnez au moins un utilisateur";
      return;
    }

    if (!signUrl) {
      error = "Entrez une URL de signature";
      return;
    }

    signing = true;
    try {
      const selectedUserIds = Array.from(selectedUsers);

      // L'API retourne maintenant directement un array de UserSignResponse
      const response = await signUsers(selectedUserIds, signUrl);

      // Response est déjà au bon format : UserSignResponse[]
      signResults = response;

      showResults = true;

      // Réinitialiser la sélection
      selectedUsers = new Set();
      selectAll = false;
      signUrl = "";
    } catch (e) {
      const apiError = e as ApiError;
      if (apiError.status === 404) {
        error = "Aucun cookie trouvé pour aujourd'hui";
      } else if (apiError.status === 400) {
        error = "Utilisateurs non trouvés";
      } else {
        error = "Erreur lors de la signature";
      }
    } finally {
      signing = false;
    }
  }

  function handleUserToggle(event: CustomEvent<string>): void {
    const userId = event.detail;
    if (selectedUsers.has(userId)) {
      selectedUsers.delete(userId);
    } else {
      selectedUsers.add(userId);
    }
    selectedUsers = selectedUsers;

    // Mettre à jour selectAll en fonction de la sélection
    const availableUsers = users.filter((u) => !u.jwtIsExpired);
    selectAll =
      selectedUsers.size === availableUsers.length && availableUsers.length > 0;
  }

  function handleToggleSelectAll(): void {
    const availableUsers = users.filter((u) => !u.jwtIsExpired);

    if (selectAll) {
      // Désélectionner tous
      selectedUsers = new Set();
      selectAll = false;
    } else {
      // Sélectionner tous les utilisateurs disponibles
      selectedUsers = new Set(availableUsers.map((u) => u.id));
      selectAll = true;
    }
  }

  function handleUrlChange(event: CustomEvent<string>): void {
    signUrl = event.detail;
  }

  function handleScanRequest(): void {
    showScanner = true;
  }

  function handleScanResult(result: string): void {
    // Vérifier si c'est une URL Epitech valide
    if (result.includes("intra.epitech.eu")) {
      signUrl = result;
      success = "QR Code scanné avec succès";
    } else {
      error = "QR Code invalide - URL Epitech attendue";
    }
  }

  function handleScanError(errorMsg: string): void {
    error = errorMsg;
  }

  function handleCloseResults() {
    showResults = false;
  }

  function goBack() {
    goto("/");
  }

  function openGuideModal() {
    showGuideModal = true;
  }

  function closeGuideModal() {
    showGuideModal = false;
  }
</script>

<div class="min-h-screen pb-safe">
  <!-- En-tête avec navigation -->
  <div
    class="flex items-center justify-between p-4 sm:p-6"
    in:fly={{ y: -20, duration: 400, easing: quintOut }}
  >
    <div class="flex items-center">
      <button
        on:click={goBack}
        class="p-2 rounded-xl hover:bg-white/10 transition-all duration-200 ease-out transform hover:scale-110 active:scale-95"
        aria-label="Retour"
      >
        <ArrowLeft class="w-5 h-5 sm:w-6 sm:h-6" />
      </button>
      <h1 class="text-xl sm:text-2xl font-bold gradient-text ml-3">
        Signatures multiples
      </h1>
    </div>

    <!-- Bouton d'aide sur mobile -->
    <button
      on:click={openGuideModal}
      class="p-2 rounded-xl hover:bg-white/10 transition-all duration-200 ease-out transform hover:scale-110 active:scale-95 sm:hidden"
      aria-label="Aide"
    >
      <HelpCircle class="w-5 h-5" />
    </button>
  </div>

  <div class="px-4 sm:px-6 max-w-7xl mx-auto">
    <!-- Messages d'alerte avec animations -->
    {#if error}
      <div
        in:fly={{ x: -50, duration: 300, easing: quintOut }}
        out:fly={{ x: 50, duration: 200, easing: quintOut }}
      >
        <AlertMessage type="error" message={error} />
      </div>
    {/if}

    {#if success}
      <div
        in:scale={{ duration: 400, easing: quintOut }}
        out:scale={{ duration: 200, easing: quintOut }}
      >
        <AlertMessage type="success" message={success} />
      </div>
    {/if}

    <!-- Layout Mobile : vertical (comme avant) -->
    <!-- Layout Desktop : horizontal avec SigningSection à gauche (1/4) et UsersList à droite (3/4) -->
    <div class="flex flex-col sm:flex-row sm:gap-6 sm:h-[calc(100vh-180px)]">
      <!-- Section de signature - Mobile: pleine largeur, Desktop: 1/4 à gauche -->
      <div
        class="sm:w-1/4 sm:flex-shrink-0"
        in:fly={{ x: -30, duration: 500, delay: 100, easing: quintOut }}
      >
        <SigningSection
          {signUrl}
          {selectedUsers}
          {signing}
          {isMobile}
          on:sign={handleSign}
          on:toggleSelectAll={handleToggleSelectAll}
          on:urlChange={handleUrlChange}
          on:scanRequest={handleScanRequest}
        />
      </div>

      <!-- Liste des utilisateurs - Mobile: pleine largeur, Desktop: 3/4 à droite -->
      <div
        class="flex-1 sm:overflow-hidden sm:pl-2"
        in:fly={{ x: 30, duration: 500, delay: 200, easing: quintOut }}
      >
        <div class="sm:h-full sm:overflow-y-auto sm:overflow-x-hidden">
          <UsersList
            {users}
            {selectedUsers}
            loading={false}
            on:userToggle={handleUserToggle}
          />
        </div>
      </div>
    </div>

    <!-- Instructions animées - Desktop uniquement -->
    <div
      class="hidden mt-6 sm:mt-0 sm:absolute sm:bottom-6 sm:left-6 sm:right-6"
      in:fly={{ y: 30, duration: 400, delay: 400, easing: quintOut }}
    >
      <div class="glass-effect-card rounded-xl p-4 sm:p-6">
        <h4 class="font-semibold text-white mb-3 flex items-center gap-2">
          <span class="text-lg">💡</span>
          Guide rapide
        </h4>
        <div
          class="grid grid-cols-1 sm:grid-cols-3 gap-3 text-sm text-gray-300"
        >
          <div class="flex items-center gap-2">
            <span
              class="flex-shrink-0 w-6 h-6 bg-red-600/10 text-red-500 rounded-full flex items-center justify-center text-xs font-bold"
              >1</span
            >
            <span>Entrez l'URL Epitech</span>
          </div>
          <div class="flex items-center gap-2">
            <span
              class="flex-shrink-0 w-6 h-6 bg-red-600/10 text-red-500 rounded-full flex items-center justify-center text-xs font-bold"
              >2</span
            >
            <span>Sélectionnez les utilisateurs</span>
          </div>
          <div class="flex items-center gap-2">
            <span
              class="flex-shrink-0 w-6 h-6 bg-red-600/10 text-red-500 rounded-full flex items-center justify-center text-xs font-bold"
              >3</span
            >
            <span>Lancez la signature</span>
          </div>
        </div>
      </div>
    </div>
  </div>
</div>

<!-- Modal Guide Mobile -->
{#if showGuideModal}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4 sm:hidden"
    in:fade={{ duration: 300, easing: quintOut }}
    out:fade={{ duration: 200, easing: quintOut }}
  >
    <!-- Backdrop -->
    <button
      class="absolute inset-0 bg-black/50 backdrop-blur-sm"
      on:click={closeGuideModal}
      in:fade={{ duration: 300 }}
      out:fade={{ duration: 200 }}
      aria-label="Fermer le guide"
    ></button>

    <!-- Modal -->
    <div
      class="relative w-full max-w-sm glass-effect-modal rounded-2xl p-6 shadow-2xl"
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
          <h2 class="text-xl font-bold gradient-text flex items-center gap-2">
            <span class="text-lg">💡</span>
            Guide rapide
          </h2>
          <p class="text-sm text-gray-400 mt-2">
            Comment utiliser les signatures multiples
          </p>
        </div>
        <button
          on:click={closeGuideModal}
          class="p-2 rounded-lg hover:bg-white/10 transition-all duration-200 ease-out transform hover:scale-110 active:scale-95 hover:rotate-90"
          aria-label="Fermer"
        >
          <svg
            class="w-5 h-5"
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

      <!-- Étapes -->
      <div
        class="space-y-4"
        in:fly={{ y: 20, duration: 400, delay: 200, easing: quintOut }}
      >
        <div
          class="flex items-start gap-3 p-3 rounded-xl bg-white/10 border border-white/20"
        >
          <span
            class="flex-shrink-0 w-8 h-8 bg-white text-gray-900 rounded-full flex items-center justify-center text-sm font-bold mt-0.5"
            >1</span
          >
          <div>
            <h4 class="font-semibold text-white mb-1">URL de signature</h4>
            <p class="text-sm text-gray-300">
              Collez l'URL de l'événement Epitech ou scannez le QR code
            </p>
          </div>
        </div>

        <div
          class="flex items-start gap-3 p-3 rounded-xl bg-white/10 border border-white/20"
        >
          <span
            class="flex-shrink-0 w-8 h-8 bg-white text-gray-900 rounded-full flex items-center justify-center text-sm font-bold mt-0.5"
            >2</span
          >
          <div>
            <h4 class="font-semibold text-white mb-1">
              Sélection utilisateurs
            </h4>
            <p class="text-sm text-gray-300">
              Choisissez les utilisateurs à signer (JWT valide requis)
            </p>
          </div>
        </div>

        <div
          class="flex items-start gap-3 p-3 rounded-xl bg-white/10 border border-white/20"
        >
          <span
            class="flex-shrink-0 w-8 h-8 bg-white text-gray-900 rounded-full flex items-center justify-center text-sm font-bold mt-0.5"
            >3</span
          >
          <div>
            <h4 class="font-semibold text-white mb-1">Signature automatique</h4>
            <p class="text-sm text-gray-300">
              Lancez le processus et consultez les résultats
            </p>
          </div>
        </div>
      </div>

      <!-- Action -->
      <div
        class="mt-6"
        in:fly={{ y: 20, duration: 300, delay: 300, easing: quintOut }}
      >
        <button
          on:click={closeGuideModal}
          class="btn-primary w-full transform transition-all duration-200 ease-out hover:scale-105 active:scale-95"
        >
          Compris !
        </button>
      </div>
    </div>
  </div>
{/if}

<!-- Scanner QR Code -->
{#if showScanner}
  <QRScanner
    onScan={handleScanResult}
    onError={handleScanError}
    onClose={() => (showScanner = false)}
  />
{/if}

<SignResults
  isOpen={showResults}
  results={signResults}
  {users}
  on:close={handleCloseResults}
/>
