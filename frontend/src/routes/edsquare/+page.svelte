<script lang="ts">
  import { validateEdsquareCodeForUsers, getCurrentUser, loadUsers, getEdsquareStatus, getEdsquareEligibleUsers } from "$lib/api";
  import { currentUser } from "$lib/stores";
  import EdsquareResults from "$lib/components/EdsquareResults.svelte";
  import type {
    ApiError,
    ValidateEdsquareResponse,
    PublicUserResponse,
    EdsquarePageData,
    EdsquareStatusResponse,
    EdsquareEligibleUsersResponse,
    EdsquareUserValidationResult
  } from "$lib/types";
  import AlertMessage from "$lib/components/AlertMessage.svelte";
  import UsersList from "$lib/components/UsersList.svelte";
  import { ArrowLeft, CheckCircle2, XCircle } from "@lucide/svelte";
  import { fly, scale } from "svelte/transition";
  import { quintOut } from "svelte/easing";
  import { goto } from "$app/navigation";
  import { onMount } from "svelte";
  import { browser } from "$app/environment";

  export let data: EdsquarePageData;

  let users: PublicUserResponse[] = data.users || [];
  let selectedUsers = new Set<string>();
  let error = "";
  let success = "";
  let validating = false;
  let validationResult: ValidateEdsquareResponse | null = null;
  let codeInput = "";
  let planningEventIdInput = "";
  let eligibleUserIds: string[] = [];
  let showResultsModal = false;
  let edsquareResults: EdsquareUserValidationResult[] = [];

  // Charger les utilisateurs au montage
  onMount(async () => {
    try {
      const loadedUsers = await loadUsers();
      users = loadedUsers;

      // Récupérer les utilisateurs éligibles EDSquare (signature + cookies valides)
      try {
        const eligibleResponse: EdsquareEligibleUsersResponse = await getEdsquareEligibleUsers();
        eligibleUserIds = eligibleResponse.users.map((u) => u.id);
        
        // Pré-sélectionner l'utilisateur actuel s'il est éligible
        if ($currentUser?.id && eligibleUserIds.includes($currentUser.id)) {
          selectedUsers.add($currentUser.id);
          selectedUsers = selectedUsers;
        }
      } catch (e) {
        console.error("Erreur lors du chargement des utilisateurs éligibles EDSquare:", e);
        eligibleUserIds = [];
      }
    } catch (e) {
      console.error("Erreur lors du chargement des utilisateurs:", e);
    }
  });

  function handleUserToggle(event: CustomEvent<string>) {
    const userId = event.detail;
    if (selectedUsers.has(userId)) {
      selectedUsers.delete(userId);
    } else {
      selectedUsers.add(userId);
    }
    selectedUsers = selectedUsers;
  }

  // Vérifier si un utilisateur peut valider (signature + cookie EDSquare valide)
  function canUserValidate(userId: string): boolean {
    // Le backend renvoie déjà uniquement les utilisateurs avec signature + cookies valides
    return eligibleUserIds.includes(userId);
  }

  // Tous les utilisateurs sont affichés comme sur la page de signatures multiples,
  // mais seuls ceux présents dans eligibleUserIds sont réellement cliquables (canValidate=true)
  $: hasSelectedUsers = selectedUsers.size > 0;

  async function validateCode() {
    if (!codeInput.trim()) {
      error = "Veuillez entrer un code EDSquare";
      return;
    }

    if (!planningEventIdInput.trim()) {
      error = "Veuillez entrer un planning_event_id";
      return;
    }

    // Si aucun utilisateur sélectionné, utiliser l'utilisateur actuel
    const usersToValidate = selectedUsers.size > 0 
      ? Array.from(selectedUsers)
      : [$currentUser?.id].filter(Boolean) as string[];

    if (usersToValidate.length === 0) {
      error = "Aucun utilisateur sélectionné";
      return;
    }

    // Vérifier les prérequis pour l'utilisateur actuel
    if (usersToValidate.includes($currentUser?.id || "") && !$currentUser?.signatureManuscrite) {
      error = "Vous devez d'abord créer une signature manuscrite dans votre profil";
      return;
    }

    validating = true;
    error = "";
    success = "";
    validationResult = null;

    try {
      const response = await validateEdsquareCodeForUsers(
        codeInput.trim(),
        planningEventIdInput.trim(),
        usersToValidate
      );

      // Stocker les résultats détaillés pour la modale
      edsquareResults = response.results;
      showResultsModal = true;

      const successCount = response.results.filter((r) => r.success).length;
      const total = response.results.length;

      // Si un seul utilisateur et que ça échoue, on affiche directement le message détaillé
      if (total === 1 && successCount === 0) {
        const first = response.results[0];
        validationResult = {
          success: false,
          message: first.message || "Erreur lors de la validation du code",
          code: codeInput.trim(),
          planning_event_id: planningEventIdInput.trim(),
        };
        error = validationResult.message;
        return;
      }

      // Cas multi-utilisateurs : message agrégé
      validationResult = {
        success: response.globalSuccess,
        message: response.globalSuccess
          ? `Code validé avec succès pour ${successCount}/${total} utilisateur(s)`
          : `Certaines validations ont échoué (${successCount}/${total} utilisateur(s) réussies)`,
        code: codeInput.trim(),
        planning_event_id: planningEventIdInput.trim(),
      };

      if (response.globalSuccess || successCount > 0) {
        success = `Code traité pour ${total} utilisateur(s) (succès: ${successCount})`;
        codeInput = "";
        planningEventIdInput = "";
        selectedUsers = new Set();
      } else {
        error = validationResult.message || "Erreur lors de la validation";
      }
    } catch (e) {
      const apiError = e as ApiError;
      if (apiError.status === 404) {
        if (apiError.message.includes("cookie") || apiError.message.includes("session")) {
          error = "Aucun cookie EDSquare trouvé. Veuillez vous connecter à EDSquare dans votre profil.";
        } else {
          error = "Signature non trouvée. Veuillez créer une signature dans votre profil.";
        }
      } else if (apiError.status === 400) {
        error = "Code invalide ou planning_event_id manquant";
      } else if (apiError.status === 401) {
        error = "Non autorisé - Vérifiez votre connexion";
      } else {
        error = "Erreur lors de la validation du code";
      }
    } finally {
      validating = false;
    }
  }

  function goBack() {
    goto("/");
  }


  let edsquareStatus: EdsquareStatusResponse | null = null;
  let loadingStatus = false;

  $: hasSignature = !!$currentUser?.signatureManuscrite;
  $: hasCookies = edsquareStatus?.has_cookies ?? false;
  $: isReady = edsquareStatus?.is_ready ?? false;

  // Recharger l'utilisateur et le statut EDSquare au montage
  onMount(async () => {
    try {
      const [user, status] = await Promise.all([
        getCurrentUser(),
        getEdsquareStatus().catch(() => null)
      ]);
      currentUser.set(user);
      edsquareStatus = status;
    } catch (e) {
      console.error("Erreur lors du rechargement:", e);
    }
  });

  // Recharger le statut après connexion EDSquare
  async function refreshStatus() {
    loadingStatus = true;
    try {
      edsquareStatus = await getEdsquareStatus();
    } catch (e) {
      console.error("Erreur lors du rechargement du statut:", e);
    } finally {
      loadingStatus = false;
    }
  }
</script>

<div class="min-h-screen bg-gradient-to-br from-gray-900 via-gray-800 to-gray-900 p-4 sm:p-6">
  <div class="max-w-2xl mx-auto">
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
      <h1 class="text-3xl font-bold gradient-text">Validation EDSquare</h1>
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

    <!-- Statut EDSquare -->
    <div
      class="glass-effect-card rounded-xl p-6 sm:p-8 mb-6"
      in:fly={{ y: 20, duration: 400, delay: 50, easing: quintOut }}
    >
      <div class="flex items-center justify-between mb-4">
        <h2 class="text-xl font-semibold gradient-text">Statut EDSquare</h2>
        <button
          on:click={refreshStatus}
          disabled={loadingStatus}
          class="p-2 rounded-lg bg-white/10 hover:bg-white/20 transition-all duration-200"
          aria-label="Actualiser"
        >
          {#if loadingStatus}
            <span class="inline-block animate-spin rounded-full h-4 w-4 border-b-2 border-white"></span>
          {:else}
            <svg class="w-5 h-5" fill="none" stroke="currentColor" viewBox="0 0 24 24">
              <path stroke-linecap="round" stroke-linejoin="round" stroke-width="2" d="M4 4v5h.582m15.356 2A8.001 8.001 0 004.582 9m0 0H9m11 11v-5h-.581m0 0a8.003 8.003 0 01-15.357-2m15.357 2H15" />
            </svg>
          {/if}
        </button>
      </div>

      {#if edsquareStatus}
        <div class="space-y-3">
          <div class="flex items-center justify-between p-3 rounded-lg {edsquareStatus.has_signature ? 'bg-green-500/10 border border-green-500/30' : 'bg-red-500/10 border border-red-500/30'}">
            <div class="flex items-center gap-3">
              {#if edsquareStatus.has_signature}
                <CheckCircle2 class="w-5 h-5 text-green-400" />
              {:else}
                <XCircle class="w-5 h-5 text-red-400" />
              {/if}
              <span class="text-sm font-medium">Signature manuscrite</span>
            </div>
            {#if !edsquareStatus.has_signature}
              <button
                on:click={() => goto("/profile")}
                class="text-xs btn-secondary px-3 py-1"
              >
                Créer
              </button>
            {/if}
          </div>

          <div class="flex items-center justify-between p-3 rounded-lg {edsquareStatus.has_cookies ? 'bg-green-500/10 border border-green-500/30' : 'bg-red-500/10 border border-red-500/30'}">
            <div class="flex items-center gap-3">
              {#if edsquareStatus.has_cookies}
                <CheckCircle2 class="w-5 h-5 text-green-400" />
              {:else}
                <XCircle class="w-5 h-5 text-red-400" />
              {/if}
              <span class="text-sm font-medium">Connexion EDSquare</span>
            </div>
            {#if !edsquareStatus.has_cookies}
              <button
                on:click={() => goto("/profile")}
                class="text-xs btn-secondary px-3 py-1"
              >
                Se connecter
              </button>
            {/if}
          </div>

          {#if edsquareStatus.is_ready}
            <div class="p-3 rounded-lg bg-green-500/10 border border-green-500/30">
              <p class="text-sm text-green-400 font-medium">
                ✓ Prêt à valider des codes EDSquare
              </p>
            </div>
          {:else}
            <div class="p-3 rounded-lg bg-yellow-500/10 border border-yellow-500/30">
              <p class="text-sm text-yellow-400">
                ⚠️ Configurez votre signature et votre connexion EDSquare dans votre profil
              </p>
              <button
                on:click={() => goto("/profile")}
                class="mt-2 btn-primary text-sm w-full"
              >
                Aller au profil
              </button>
            </div>
          {/if}
        </div>
      {:else}
        <p class="text-sm text-gray-400">Chargement du statut...</p>
      {/if}
    </div>


    <!-- Main Card -->
    <div
      class="glass-effect-card rounded-xl p-6 sm:p-8 mb-6"
      in:fly={{ y: 20, duration: 400, delay: 100, easing: quintOut }}
    >
      {#if !isReady}
        <div
          class="bg-yellow-500/10 border border-yellow-500/50 rounded-xl p-4 mb-6"
          in:scale={{ duration: 400, easing: quintOut }}
        >
          <p class="text-yellow-400 text-sm">
            ⚠️ Configurez votre signature et votre connexion EDSquare dans votre profil pour valider des codes.
          </p>
          <button
            on:click={() => goto("/profile")}
            class="mt-3 btn-primary text-sm"
          >
            Aller au profil
          </button>
        </div>
      {/if}

      <h2 class="text-xl font-semibold gradient-text mb-4">
        Valider un code EDSquare
      </h2>

      <!-- Sélection des utilisateurs -->
      {#if isReady}
        <div class="mb-6">
          <div class="flex items-center justify-between mb-3">
            <div class="block text-sm font-medium text-gray-300">
              Sélectionner les utilisateurs à valider (vous pouvez en cocher plusieurs)
            </div>
            <span class="text-xs text-gray-400">
              {eligibleUserIds.length} utilisateur{eligibleUserIds.length > 1 ? 's' : ''} éligible{eligibleUserIds.length > 1 ? 's' : ''}
            </span>
          </div>
          <div class="bg-white/5 rounded-lg p-4 border border-white/10 max-h-64 overflow-y-auto">
            {#if users.length === 0}
              <p class="text-xs text-gray-400 text-center py-4">
                Aucun utilisateur éligible trouvé
              </p>
            {:else}
              <UsersList
                {users}
                {selectedUsers}
                loading={false}
                mode="edsquare"
                edsquareEligibleIds={eligibleUserIds}
                on:userToggle={handleUserToggle}
              />
            {/if}
          </div>
          {#if selectedUsers.size === 0}
            <p class="text-xs text-yellow-400 mt-2">
              ⚠️ Aucun utilisateur sélectionné. Le code sera validé pour vous uniquement si vous cliquez sur "Valider le code".
            </p>
          {:else if selectedUsers.size === 1}
            <p class="text-xs text-green-400 mt-2">
              ✓ {selectedUsers.size} utilisateur sélectionné
            </p>
          {:else}
            <p class="text-xs text-green-400 mt-2">
              ✓ {selectedUsers.size} utilisateurs sélectionnés - Le code sera validé pour tous les comptes cochés
            </p>
          {/if}
        </div>
      {:else if isReady && eligibleUsers.length === 0}
        <div class="mb-6 p-4 bg-yellow-500/10 border border-yellow-500/30 rounded-lg">
          <p class="text-xs text-yellow-400">
            ⚠️ Aucun utilisateur éligible trouvé (signature + connexion EDSquare valide). Le code sera validé uniquement pour votre compte.
          </p>
        </div>
      {/if}

      <div class="space-y-4">
        <!-- Planning Event ID Input -->
        <div>
          <label for="planningEventId" class="block text-sm font-medium text-gray-300 mb-2">
            Planning Event ID
          </label>
          <input
            type="text"
            id="planningEventId"
            bind:value={planningEventIdInput}
            disabled={validating || !isReady}
            placeholder="199289"
            class="input-field w-full"
            on:keydown={(e) => {
              if (e.key === "Enter" && !validating && isReady) {
                validateCode();
              }
            }}
          />
        </div>

        <!-- Code Input -->
        <div>
          <label for="code" class="block text-sm font-medium text-gray-300 mb-2">
            Code EDSquare
          </label>
            <input
              type="text"
              id="code"
              bind:value={codeInput}
              disabled={validating || !isReady}
              placeholder="000000"
              class="input-field w-full"
              on:keydown={(e) => {
                if (e.key === "Enter" && !validating && isReady) {
                  validateCode();
                }
              }}
            />
        </div>

        <!-- Validate Button -->
        <button
          on:click={validateCode}
            disabled={validating || !codeInput.trim() || !planningEventIdInput.trim() || !isReady}
          class="btn-primary w-full"
        >
          {#if validating}
            <span
              class="inline-block animate-spin rounded-full h-5 w-5 border-b-2 border-white"
            ></span>
          {:else}
            Valider le code
          {/if}
        </button>
      </div>

      <!-- Result -->
      {#if validationResult}
        <div
          class="mt-6 p-4 rounded-xl border {validationResult.success
            ? 'bg-green-500/10 border-green-500/50'
            : 'bg-red-500/10 border-red-500/50'}"
          in:scale={{ duration: 400, easing: quintOut }}
        >
          <div class="flex items-center gap-3">
            {#if validationResult.success}
              <CheckCircle2 class="w-6 h-6 text-green-400" />
            {:else}
              <XCircle class="w-6 h-6 text-red-400" />
            {/if}
            <div>
              <p
                class="font-semibold {validationResult.success
                  ? 'text-green-400'
                  : 'text-red-400'}"
              >
                {validationResult.message}
              </p>
              {#if validationResult.success}
                <p class="text-sm text-gray-400 mt-1">
                  Code validé : {validationResult.code}
                </p>
              {/if}
    </div>
  </div>

  <EdsquareResults
    isOpen={showResultsModal}
    results={edsquareResults}
    on:close={() => (showResultsModal = false)}
  />
</div>
      {/if}
    </div>

    <!-- Instructions -->
    {#if isReady}
      <div
        class="glass-effect-card rounded-xl p-4 sm:p-6"
        in:fly={{ y: 20, duration: 400, delay: 300, easing: quintOut }}
      >
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
              <span>Connectez-vous à EDSquare avec vos identifiants (en haut de la page)</span>
            </li>
            <li class="flex gap-3">
              <span
                class="flex-shrink-0 w-6 h-6 bg-red-600/10 text-red-500 rounded-full flex items-center justify-center text-xs font-bold"
                >2</span
              >
              <span>Entrez le Planning Event ID et le code EDSquare</span>
            </li>
            <li class="flex gap-3">
              <span
                class="flex-shrink-0 w-6 h-6 bg-red-600/10 text-red-500 rounded-full flex items-center justify-center text-xs font-bold"
                >3</span
              >
              <span>Votre signature manuscrite sera utilisée pour valider le code</span>
            </li>
            <li class="flex gap-3">
              <span
                class="flex-shrink-0 w-6 h-6 bg-red-600/10 text-red-500 rounded-full flex items-center justify-center text-xs font-bold"
                >4</span
              >
              <span>Le code sera validé automatiquement</span>
            </li>
        </ol>
      </div>
    {/if}
  </div>
</div>

