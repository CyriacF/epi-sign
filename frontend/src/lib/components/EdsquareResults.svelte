<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { fade, fly, scale } from "svelte/transition";
  import { quintOut } from "svelte/easing";
  import { CheckCircle2, XCircle, X, Activity } from "@lucide/svelte";
  import type { EdsquareUserValidationResult } from "$lib/types";

  export let isOpen: boolean = false;
  export let results: EdsquareUserValidationResult[] = [];

  const dispatch = createEventDispatcher();

  $: stats = {
    total: results.length,
    success: results.filter((r) => r.success).length,
    failed: results.filter((r) => !r.success).length,
  };

  function handleClose() {
    dispatch("close");
  }

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
    in:fade={{ duration: 150 }}
    out:fade={{ duration: 150 }}
  >
    <!-- Backdrop -->
    <button
      class="absolute inset-0 bg-black/50 backdrop-blur-sm"
      on:click={handleClose}
      aria-label="Fermer les résultats EDSquare"
    />

    <!-- Modal -->
    <div
      class="relative w-full h-[80vh] sm:h-auto sm:max-w-2xl sm:max-h-[85vh] bg-gray-900 sm:glass-effect sm:rounded-2xl shadow-2xl overflow-hidden rounded-t-2xl flex flex-col"
      in:fly={{ y: 300, duration: 300, easing: quintOut }}
      out:fly={{ y: 300, duration: 250, easing: quintOut }}
      style="transform-origin: center bottom;"
    >
      <!-- Header -->
      <div
        class="flex justify-between items-start p-4 sm:p-6 border-b border-white/10 flex-shrink-0"
        in:fly={{ y: -20, duration: 250, easing: quintOut }}
      >
        <div class="flex-1 min-w-0">
          <h2 class="text-xl sm:text-2xl font-bold gradient-text">
            Résultats EDSquare
          </h2>
          <div class="flex items-center gap-4 mt-2 text-sm text-gray-400">
            <div class="flex items-center gap-1">
              <Activity class="w-4 h-4" />
              <span>{stats.total} tentative{stats.total > 1 ? "s" : ""}</span>
            </div>
            <div class="flex items-center gap-1 text-green-400">
              <CheckCircle2 class="w-4 h-4" />
              <span>{stats.success} réussie{stats.success > 1 ? "s" : ""}</span>
            </div>
            <div class="flex items-center gap-1 text-red-400">
              <XCircle class="w-4 h-4" />
              <span>{stats.failed} échouée{stats.failed > 1 ? "s" : ""}</span>
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

      <!-- Content -->
      <div
        class="flex-1 overflow-y-auto p-4 sm:p-6 space-y-3 sm:space-y-4"
        in:fly={{ y: 20, duration: 300, delay: 100, easing: quintOut }}
      >
        {#if results.length === 0}
          <div class="flex flex-col items-center justify-center h-full text-gray-400 py-8">
            <Activity class="w-10 h-10 mb-4 opacity-50" />
            <p>Aucun résultat à afficher</p>
          </div>
        {:else}
          {#each results as r, index}
            <div
              class="flex items-start gap-3 border rounded-xl p-3 sm:p-4 bg-white/5 {r.success
                ? 'border-green-500/40'
                : 'border-red-500/40'}"
              in:scale={{ duration: 200, delay: 80 + index * 40, easing: quintOut }}
            >
              {#if r.success}
                <CheckCircle2 class="w-5 h-5 mt-0.5 text-green-400" />
              {:else}
                <XCircle class="w-5 h-5 mt-0.5 text-red-400" />
              {/if}

              <div class="flex-1 min-w-0">
                <div class="flex items-center justify-between gap-2">
                  <div class="flex items-center gap-2 min-w-0">
                    <span class="font-semibold text-sm sm:text-base truncate">
                      {r.username}
                    </span>
                    <span
                      class="text-[11px] px-2 py-0.5 rounded-full border {r.success
                        ? 'border-green-500/40 text-green-400 bg-green-500/10'
                        : 'border-red-500/40 text-red-400 bg-red-500/10'}"
                    >
                      {r.success ? "Succès" : "Échec"}
                    </span>
                  </div>
                </div>
                {#if r.message}
                  <p class="text-xs sm:text-sm text-gray-300 mt-1">
                    {r.message}
                  </p>
                {/if}
              </div>
            </div>
          {/each}
        {/if}
      </div>
    </div>
  </div>
{/if}

