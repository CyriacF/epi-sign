<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { fly, fade, scale } from "svelte/transition";
  import { quintOut } from "svelte/easing";
  import {
    CheckCircle,
    XCircle,
    AlertCircle,
    Clock,
    Wifi,
    X,
    TrendingUp,
    TrendingDown,
    Activity,
  } from "@lucide/svelte";
  import type {
    PublicUserResponse,
    SignResponse,
    UserSignResponse,
  } from "$lib/types";

  export let isOpen: boolean = false;
  export let results: UserSignResponse[] = [];
  export let users: PublicUserResponse[] = [];

  const dispatch = createEventDispatcher();

  // Mapping des résultats vers des configurations d'affichage
  function getResultConfig(result: SignResponse) {
    switch (result) {
      case "success":
        return {
          icon: CheckCircle,
          color: "text-green-400 bg-green-500/10 border-green-500/30",
          title: "Signature réussie",
          description: "L'utilisateur a été signé avec succès",
        };
      case "tokenExpired":
        return {
          icon: Clock,
          color: "text-orange-400 bg-orange-500/10 border-orange-500/30",
          title: "JWT expiré",
          description: "Le JWT de l'utilisateur a expiré",
        };
      case "tokenNotFound":
        return {
          icon: XCircle,
          color: "text-red-400 bg-red-500/10 border-red-500/30",
          title: "JWT non trouvé",
          description: "Aucun JWT configuré pour cet utilisateur",
        };
      case "alreadySigned":
        return {
          icon: AlertCircle,
          color: "text-blue-400 bg-blue-500/10 border-blue-500/30",
          title: "Déjà signé",
          description: "L'utilisateur a déjà signé pour cet événement",
        };
      case "serviceUnavailable":
        return {
          icon: Wifi,
          color: "text-gray-400 bg-gray-500/10 border-gray-500/30",
          title: "Service indisponible",
          description: "Le service de signature n'est pas accessible",
        };
      case "unknownError":
      default:
        return {
          icon: XCircle,
          color: "text-gray-400 bg-gray-500/10 border-gray-500/30",
          title: "Erreur inconnue",
          description: "Une erreur inattendue s'est produite",
        };
    }
  }

  // Trouver un utilisateur par son ULID
  function findUser(ulid: string): PublicUserResponse | undefined {
    return users.find((user) => user.id === ulid);
  }

  // Grouper les résultats par statut
  $: groupedResults = results.reduce(
    (acc, item) => {
      if (!acc[item.response]) {
        acc[item.response] = [];
      }
      acc[item.response].push(item);
      return acc;
    },
    {} as Record<SignResponse, typeof results>
  );

  // Statistiques
  $: stats = {
    total: results.length,
    success: (groupedResults.success || []).length,
    failed: results.length - (groupedResults.success || []).length,
    successRate:
      results.length > 0
        ? Math.round(
            ((groupedResults.success || []).length / results.length) * 100
          )
        : 0,
  };

  function handleClose() {
    dispatch("close");
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
    class="fixed inset-0 z-50 flex items-end sm:items-center justify-center"
    in:fade={{ duration: 200 }}
    out:fade={{ duration: 150 }}
  >
    <!-- Backdrop -->
    <button
      class="absolute inset-0 bg-black/50 backdrop-blur-sm"
      on:click={handleClose}
      aria-label="Fermer les résultats"
    ></button>

    <!-- Modal -->
    <div
      class="relative w-full h-[90vh] sm:h-auto sm:max-w-2xl sm:max-h-[90vh] bg-gray-900 sm:glass-effect sm:rounded-2xl shadow-2xl overflow-hidden rounded-t-2xl flex flex-col"
      in:fly={{
        y: 300,
        duration: 400,
        easing: quintOut,
      }}
      out:fly={{
        y: 300,
        duration: 300,
        easing: quintOut,
      }}
      style="transform-origin: center bottom;"
    >
      <!-- Header -->
      <div
        class="flex justify-between items-start p-4 sm:p-6 border-b border-white/10 safe-top flex-shrink-0"
        in:fly={{
          y: -20,
          duration: 400,
          delay: 100,
          easing: quintOut,
        }}
      >
        <div class="flex-1 min-w-0">
          <h2 class="text-xl sm:text-2xl font-bold gradient-text">Résultats</h2>
          <!-- Mobile: Stats condensées -->
          <div class="sm:hidden mt-2">
            <div class="flex items-center gap-3 text-sm">
              <div class="flex items-center gap-1">
                <Activity class="w-3 h-3 text-gray-400" />
                <span class="text-gray-400">{stats.total}</span>
              </div>
              <div class="flex items-center gap-1">
                <CheckCircle class="w-3 h-3 text-green-400" />
                <span class="text-green-400">{stats.success}</span>
              </div>
              <div class="flex items-center gap-1">
                <XCircle class="w-3 h-3 text-red-400" />
                <span class="text-red-400">{stats.failed}</span>
              </div>
              <div class="flex items-center gap-1 font-semibold">
                {#if stats.successRate >= 80}
                  <TrendingUp class="w-3 h-3 text-green-400" />
                  <span class="text-green-400">{stats.successRate}%</span>
                {:else if stats.successRate >= 50}
                  <Activity class="w-3 h-3 text-orange-400" />
                  <span class="text-orange-400">{stats.successRate}%</span>
                {:else}
                  <TrendingDown class="w-3 h-3 text-red-400" />
                  <span class="text-red-400">{stats.successRate}%</span>
                {/if}
              </div>
            </div>
          </div>
          <!-- Desktop: Stats détaillées -->
          <div class="hidden sm:flex items-center gap-4 mt-2 text-sm">
            <div class="flex items-center gap-1 text-gray-400">
              <Activity class="w-4 h-4" />
              <span>{stats.total} tentatives</span>
            </div>
            <div class="flex items-center gap-1 text-green-400">
              <CheckCircle class="w-4 h-4" />
              <span>{stats.success} réussies</span>
            </div>
            <div class="flex items-center gap-1 text-red-400">
              <XCircle class="w-4 h-4" />
              <span>{stats.failed} échouées</span>
            </div>
          </div>
        </div>
        <button
          on:click={handleClose}
          class="p-2 rounded-lg hover:bg-white/10 transition-colors flex-shrink-0 ml-3"
          aria-label="Fermer"
        >
          <X class="w-5 h-5 sm:w-6 sm:h-6" />
        </button>
      </div>

      <!-- Results Content - Scrollable -->
      <div
        class="flex-1 overflow-y-auto p-4 sm:p-6 min-h-0"
        in:fly={{
          y: 20,
          duration: 400,
          delay: 200,
          easing: quintOut,
        }}
      >
        {#if results.length === 0}
          <div
            class="flex flex-col items-center justify-center h-full text-gray-400 py-8"
          >
            <Activity class="w-12 h-12 mb-4 opacity-50" />
            <p>Aucun résultat à afficher</p>
          </div>
        {:else}
          <div class="space-y-4 sm:space-y-6">
            {#each Object.entries(groupedResults) as [status, items], index}
              {@const config = getResultConfig(status as SignResponse)}
              {@const IconComponent = config.icon}

              <div
                class="space-y-3"
                in:fly={{
                  y: 20,
                  duration: 300,
                  delay: 250 + index * 50,
                  easing: quintOut,
                }}
              >
                <!-- Status Header -->
                <div class="flex items-center gap-2 sm:gap-3">
                  <svelte:component
                    this={IconComponent}
                    class="w-4 h-4 sm:w-5 sm:h-5 {config.color.split(
                      ' '
                    )[0]} flex-shrink-0"
                  />
                  <h3 class="font-semibold text-base sm:text-lg">
                    {config.title}
                  </h3>
                  <span
                    class="px-2 py-1 rounded-lg bg-white/10 text-xs font-medium"
                  >
                    {items.length}
                  </span>
                </div>

                <!-- Users List -->
                <div class="space-y-2">
                  {#each items as item, itemIndex}
                    {@const user = findUser(item.ulid)}
                    <div
                      class="flex items-center gap-3 p-3 rounded-xl {config.color} border"
                      in:scale={{
                        duration: 200,
                        delay: 300 + index * 50 + itemIndex * 30,
                        start: 0.95,
                        easing: quintOut,
                      }}
                    >
                      <svelte:component
                        this={IconComponent}
                        class="w-4 h-4 flex-shrink-0"
                      />
                      <div class="flex-1 min-w-0">
                        <div class="font-medium text-sm sm:text-base">
                          {user?.username || "Utilisateur inconnu"}
                        </div>
                        <div class="text-xs opacity-75 truncate font-mono">
                          {item.ulid}
                        </div>
                      </div>
                      <!-- Mobile: masquer la description, Desktop: afficher -->
                      <div
                        class="hidden sm:block text-xs opacity-75 text-right max-w-[120px]"
                      >
                        {config.description}
                      </div>
                    </div>
                  {/each}
                </div>
              </div>
            {/each}
          </div>
        {/if}
      </div>
      <!-- Actions - Fixed bottom -->
      <div
        class="bg-gray-900 sm:bg-transparent sm:backdrop-blur-xl border-t border-white/10 safe-bottom flex-shrink-0"
        in:fly={{
          y: 20,
          duration: 400,
          delay: 300,
          easing: quintOut,
        }}
      >
        <div class="flex gap-3 p-4 sm:p-6">
          <button on:click={handleClose} class="btn-secondary flex-1">
            Fermer
          </button>
        </div>
      </div>
    </div>
  </div>
{/if}
