<script lang="ts">
  import { createEventDispatcher } from "svelte";
  import type { PublicUserResponse } from "$lib/types";
  import { scale, fly } from "svelte/transition";
  import { quintOut } from "svelte/easing";
  import { currentUser } from "$lib/stores";
  import { Braces, Check, X } from "@lucide/svelte";

  export let user: PublicUserResponse;
  export let isSelected: boolean;
  // Mode d’affichage :
  // - "jwt" (par défaut) : badge basé sur l’expiration du JWT
  // - "edsquare" : badge basé sur l’éligibilité EDSquare (propagée par le parent)
  export let mode: "jwt" | "edsquare" = "jwt";
  // Pour le mode "edsquare" : indique si l'utilisateur est prêt (signature + cookies)
  // Si null, on considère l'utilisateur comme cliquable par défaut
  export let canValidate: boolean | null = null;

  const dispatch = createEventDispatcher();

  // Calcul local de l’état JWT (sans muter l’objet user)
  $: jwtIsExpiredComputed =
    user.jwtExpiresAt === undefined ||
    user.jwtExpiresAt === null ||
    new Date(user.jwtExpiresAt) < new Date();

  // Déterminer si la carte est cliquable
  $: isEnabled =
    mode === "jwt"
      ? !jwtIsExpiredComputed
      : canValidate ?? true;

  // Texte et style du badge en fonction du mode
  $: badgeText =
    mode === "jwt"
      ? jwtIsExpiredComputed
        ? "JWT expiré"
        : "JWT valide"
      : canValidate === false
        ? "EDSquare non prêt"
        : "EDSquare prêt";

  $: badgeClasses =
    mode === "jwt"
      ? jwtIsExpiredComputed
        ? "bg-red-500/20 text-red-400 border-red-500/30"
        : "bg-green-500/20 text-green-400 border-green-500/30"
      : canValidate === false
        ? "bg-red-500/20 text-red-400 border-red-500/30"
        : "bg-green-500/20 text-green-400 border-green-500/30";

  function handleToggle() {
    if (isEnabled) {
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
  class="block glass-effect-card rounded-xl p-4 user-card-subtle {isEnabled
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
        disabled={!isEnabled}
        class="checkbox-custom"
      />
      {#if isSelected && isEnabled}
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
        {badgeClasses}"
        in:scale={{ duration: 200, delay: 100, easing: quintOut }}
      >
        <span class="hidden sm:inline"
          >{badgeText}</span
        >
        <span class="sm:hidden flex flex-row">
          <Braces size="18" class=" mr-1" />
          {#if mode === "jwt" && jwtIsExpiredComputed}
            <X size="18" />
          {:else}
            <Check size="18" />
          {/if}
        </span>
      </span>
    </div>
  </div>
</label>
