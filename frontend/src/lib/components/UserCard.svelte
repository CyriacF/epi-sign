<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { PublicUserResponse } from "$lib/types";
  import { scale, fly } from "svelte/transition";
  import { quintOut } from "svelte/easing";
  import { currentUser } from "$lib/stores";
  import { Braces, Check, X } from "@lucide/svelte";

  export let user: PublicUserResponse;
  export let isSelected: boolean;

  const dispatch = createEventDispatcher();

  user.jwtIsExpired =
    user.jwtExpiresAt === undefined ||
    user.jwtExpiresAt === null ||
    new Date(user.jwtExpiresAt) < new Date();

  function handleToggle() {
    if (!user.jwtIsExpired) {
      dispatch("toggle", user.id);
    }
  }

  // Vérifier si c'est l'utilisateur actuel
  $: isCurrentUser = $currentUser?.id === user.id;

  // Animation pour le changement de sélection
  let previouslySelected = isSelected;
  $: if (isSelected !== previouslySelected) {
    previouslySelected = isSelected;
  }
</script>

<label
  for="user-{user.id}"
  class="block glass-effect-card rounded-xl p-4 user-card-subtle {!user.jwtIsExpired
    ? 'cursor-pointer'
    : 'cursor-not-allowed opacity-50'} 
    {isSelected ? 'ring-2 ring-red-600/40 bg-red-600/10' : ''}"
  in:fly={{ x: -20, duration: 300, easing: quintOut }}
>
  <div class="flex items-center gap-3">
    <div class="relative flex-shrink-0 flex items-center justify-center">
      <input
        type="checkbox"
        id="user-{user.id}"
        checked={isSelected}
        on:change={handleToggle}
        disabled={user.jwtIsExpired}
        class="checkbox-custom"
      />
      {#if isSelected && !user.jwtIsExpired}
        <div
          class="absolute inset-0 rounded border-2 border-red-600 animate-pulse pointer-events-none"
          in:scale={{ duration: 200, easing: quintOut }}
        ></div>
      {/if}
    </div>

    <div class="flex-1 min-w-0">
      <div
        class="font-semibold text-base transition-colors duration-200 ease-out {isSelected
          ? 'text-white'
          : ''} flex items-center gap-2"
      >
        <span class="truncate">{user.username}</span>
        {#if isCurrentUser}
          <span
            class="inline-block px-2 py-0.5 bg-white text-gray-900 text-xs font-bold rounded-full"
            in:scale={{ duration: 300, easing: quintOut }}
          >
            Toi
          </span>
        {/if}
      </div>
      <div class="text-xs text-gray-400 truncate font-mono">
        {user.id}
      </div>
    </div>

    <div class="flex-shrink-0">
      <span
        class="inline-block px-3 py-1.5 rounded-lg text-xs font-medium border transition-all duration-200 ease-out
        {user.jwtIsExpired
          ? 'bg-red-500/20 text-red-400 border-red-500/30'
          : 'bg-green-500/20 text-green-400 border-green-500/30'}"
        in:scale={{ duration: 200, delay: 100, easing: quintOut }}
      >
        <span class="hidden sm:inline"
          >{user.jwtIsExpired ? "JWT expiré" : "JWT valide"}</span
        >
        <span class="sm:hidden flex flex-row">
          <Braces size="18" class=" mr-1" />
          {#if user.jwtIsExpired}<X size="18" />{:else}<Check
              size="18"
            />{/if}</span
        >
      </span>
    </div>
  </div>
</label>
