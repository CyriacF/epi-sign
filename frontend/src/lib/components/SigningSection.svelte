<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import { ScanQrCode } from "@lucide/svelte";

  export let signUrl: string;
  export let selectedUsers: Set<string>;
  export let signing: boolean;
  export let isMobile: boolean = false;

  const dispatch = createEventDispatcher();

  function handleSign() {
    dispatch("sign");
  }

  function handleUrlChange(event: Event) {
    const target = event.target as HTMLInputElement;
    dispatch("urlChange", target.value);
  }

  function handleScanRequest() {
    dispatch("scanRequest");
  }
</script>

<!-- Mobile: sticky top, Desktop: static dans la sidebar -->
<div
  class="sm:static sm:-mx-0 sm:px-0 sm:py-0 sm:rounded-xl sm:bg-transparent sm:border-0 sm:mb-0
         sticky top-16 z-40 -mx-1 px-4 pt-4 pb-1 rounded-xl bg-gray-900/95 border-b border-gray-700/50 backdrop-blur-2xl mb-6
         sm:glass-effect-card"
>
  <div class="space-y-4">
    <!-- Titre desktop uniquement -->
    <div class="hidden sm:block">
      <h2 class="text-lg font-semibold gradient-text mb-4">Signature</h2>
    </div>

    <div>
      <label for="signUrl" class="block text-sm font-medium text-gray-300 mb-2">
        URL de signature Epitech
      </label>
      <div class="flex gap-2">
        <input
          type="text"
          id="signUrl"
          value={signUrl}
          on:input={handleUrlChange}
          placeholder="https://intra.epitech.eu/..."
          class="input-field flex-1 text-sm"
        />
        {#if isMobile}
          <button
            on:click={handleScanRequest}
            class="cursor-pointer px-4 bg-gray-700/80 hover:bg-gray-700/90 sm:bg-white/10 sm:hover:bg-white/20 rounded-xl justify-center align-center transition-all duration-200 ease-out transform hover:scale-105 active:scale-95"
            title="Scanner QR Code"
            aria-label="Scanner QR Code"
          >
            <ScanQrCode />
          </button>
        {/if}
      </div>
    </div>

    <!-- Informations sélection -->
    <div
      class="sm:bg-white/5 sm:rounded-lg sm:p-3 sm:border sm:border-white/10"
    >
      <div class="flex items-center justify-between text-sm">
        <span class="text-gray-400">Utilisateurs sélectionnés</span>
        <span class="font-semibold text-red-500">{selectedUsers.size}</span>
      </div>
    </div>

    <!-- Bouton de signature -->
    <div class="flex gap-3">
      <button
        on:click={handleSign}
        disabled={signing || selectedUsers.size === 0 || !signUrl}
        class="btn-primary flex-1 text-sm sm:text-base"
      >
        {#if signing}
          <span
            class="inline-block animate-spin rounded-full h-5 w-5 border-b-2 border-white"
          ></span>
        {:else}
          Signer
        {/if}
      </button>
    </div>

    <!-- Instructions desktop -->
    <div
      class="hidden sm:block text-xs text-gray-500 bg-white/5 rounded-lg p-3"
    >
      <p class="font-medium mb-2">💡 Instructions :</p>
      <ol class="space-y-1">
        <li>1. Collez l'URL de signature Epitech</li>
        <li>2. Sélectionnez les utilisateurs dans la liste</li>
        <li>3. Cliquez sur "Signer" pour lancer la signature</li>
      </ol>
    </div>
  </div>
</div>
