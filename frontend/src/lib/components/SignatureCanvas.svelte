<script lang="ts">
  import { onMount, onDestroy } from "svelte";
  import { createEventDispatcher } from "svelte";
  import { fly, fade } from "svelte/transition";
  import { quintOut } from "svelte/easing";

  export let isOpen: boolean = false;
  export let currentSignature: string | null = null;

  const dispatch = createEventDispatcher();

  let canvas: HTMLCanvasElement;
  let ctx: CanvasRenderingContext2D | null = null;
  let isDrawing = false;
  let hasSignature = false;

  $: if (isOpen && canvas) {
    ctx = canvas.getContext("2d");
    if (ctx) {
      ctx.strokeStyle = "#000000";
      ctx.lineWidth = 2;
      ctx.lineCap = "round";
      ctx.lineJoin = "round";
      
      // Charger la signature existante si elle existe
      if (currentSignature) {
        const img = new Image();
        img.onload = () => {
          if (ctx && canvas) {
            ctx.clearRect(0, 0, canvas.width, canvas.height);
            ctx.drawImage(img, 0, 0);
            hasSignature = true;
          }
        };
        img.src = currentSignature;
      } else {
        // Réinitialiser le canvas si pas de signature existante
        ctx.clearRect(0, 0, canvas.width, canvas.height);
        hasSignature = false;
      }
    }
  }

  function startDrawing(e: MouseEvent | TouchEvent) {
    isDrawing = true;
    const point = getPoint(e);
    if (ctx && point) {
      ctx.beginPath();
      ctx.moveTo(point.x, point.y);
      hasSignature = true;
    }
  }

  function draw(e: MouseEvent | TouchEvent) {
    if (!isDrawing || !ctx) return;
    e.preventDefault();
    
    const point = getPoint(e);
    if (point) {
      ctx.lineTo(point.x, point.y);
      ctx.stroke();
    }
  }

  function stopDrawing() {
    isDrawing = false;
  }

  function getPoint(e: MouseEvent | TouchEvent): { x: number; y: number } | null {
    if (!canvas) return null;
    
    const rect = canvas.getBoundingClientRect();
    
    if (e instanceof MouseEvent) {
      return {
        x: e.clientX - rect.left,
        y: e.clientY - rect.top,
      };
    } else if (e instanceof TouchEvent) {
      const touch = e.touches[0] || e.changedTouches[0];
      return {
        x: touch.clientX - rect.left,
        y: touch.clientY - rect.top,
      };
    }
    
    return null;
  }

  function clear() {
    if (ctx && canvas) {
      ctx.clearRect(0, 0, canvas.width, canvas.height);
      hasSignature = false;
    }
  }

  function save() {
    if (!canvas || !hasSignature) return;
    
    const dataUrl = canvas.toDataURL("image/png");
    dispatch("save", dataUrl);
  }

  function close() {
    dispatch("close");
  }

  // Empêcher le scroll sur mobile pendant le dessin
  function preventScroll(e: TouchEvent) {
    if (isDrawing) {
      e.preventDefault();
    }
  }
</script>

{#if isOpen}
  <div
    class="fixed inset-0 z-50 flex items-center justify-center p-4"
    in:fade={{ duration: 300, easing: quintOut }}
    out:fade={{ duration: 200, easing: quintOut }}
  >
    <!-- Backdrop -->
    <button
      class="absolute inset-0 bg-black/50 backdrop-blur-sm"
      on:click={close}
      in:fade={{ duration: 300 }}
      out:fade={{ duration: 200 }}
      aria-label="Fermer"
    ></button>

    <!-- Modal -->
    <div
      class="relative w-full max-w-2xl glass-effect-modal rounded-2xl p-6 sm:p-8 shadow-2xl"
      in:fly={{ y: -20, duration: 400, easing: quintOut }}
      out:fly={{ y: 20, duration: 200, easing: quintOut }}
    >
      <!-- Header -->
      <div class="flex justify-between items-start mb-6">
        <div>
          <h2 class="text-2xl font-bold gradient-text">Signature manuscrite</h2>
          <p class="text-sm text-gray-400 mt-2">
            Dessinez votre signature avec la souris ou votre doigt
          </p>
        </div>
        <button
          on:click={close}
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

      <!-- Canvas -->
      <div class="mb-6">
        <div
          class="bg-white rounded-lg border-2 border-gray-700 overflow-hidden"
          style="touch-action: none;"
        >
          <canvas
            bind:this={canvas}
            width={800}
            height={300}
            class="w-full h-auto cursor-crosshair"
            on:mousedown={startDrawing}
            on:mousemove={draw}
            on:mouseup={stopDrawing}
            on:mouseleave={stopDrawing}
            on:touchstart|preventDefault={startDrawing}
            on:touchmove|preventDefault={draw}
            on:touchend|preventDefault={stopDrawing}
            style="touch-action: none;"
          ></canvas>
        </div>
      </div>

      <!-- Actions -->
      <div class="flex gap-3">
        <button
          type="button"
          on:click={clear}
          disabled={!hasSignature}
          class="btn-secondary flex-1 transform transition-all duration-200 ease-out hover:scale-105 active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Effacer
        </button>
        <button
          type="button"
          on:click={close}
          class="btn-secondary flex-1 transform transition-all duration-200 ease-out hover:scale-105 active:scale-95"
        >
          Annuler
        </button>
        <button
          type="button"
          on:click={save}
          disabled={!hasSignature}
          class="btn-primary flex-1 transform transition-all duration-200 ease-out hover:scale-105 active:scale-95 disabled:opacity-50 disabled:cursor-not-allowed"
        >
          Enregistrer
        </button>
      </div>
    </div>
  </div>
{/if}
