<script lang="ts">
  import { onMount, onDestroy, tick } from "svelte";
  
  import { TriangleAlert, X } from "@lucide/svelte";

  export let onScan: (result: string) => void;
  export let onClose: () => void;
  export let onError: (error: string) => void;

  let scannerElement: HTMLElement;
  let hasPermission: boolean = false;
  let isChecking: boolean = true;
  let permissionError: string = "";
  let isIOS: boolean = false;
  let isScanning: boolean = false;
  let videoEl: HTMLVideoElement | null = null;
  let mediaStream: MediaStream | null = null;
  let videoTrack: MediaStreamTrack | null = null;
  let googleScanTimer: number | null = null;
  let barcodeDetector: any = null;
  let zoomSupported: boolean = false;
  let minZoom = 1;
  let maxZoom = 1;
  let zoomStep = 0.1;
  let zoom = 1;
  let cssZoom = 1;

  onMount(async () => {
    // Détecter iOS
    isIOS =
      /iPad|iPhone|iPod/.test(navigator.userAgent) && !(window as any).MSStream;

    // Vérifier d'abord les permissions
    await checkCameraPermission();
  });

  onDestroy(() => {
    stopGoogleScanning();
  });

  async function checkCameraPermission() {
    try {
      // Sur iOS, l'API Permissions n'est pas fiable pour la caméra
      // On essaie directement d'accéder à la caméra
      await requestCameraAccess();
    } catch (error) {
      console.error("Erreur lors de la vérification des permissions:", error);
      onError(
        "Erreur lors de la vérification des permissions. Veuillez réessayer."
      );
    } finally {
      isChecking = false;
    }
  }

  async function requestCameraAccess() {
    try {
      // Configuration spécifique pour iOS
      const constraints = {
        video: {
          facingMode: "environment",
          width: { ideal: 1280 },
          height: { ideal: 720 },
        },
      };

      // Essayer d'accéder à la caméra
      const stream = await navigator.mediaDevices.getUserMedia(constraints);

      // Si on arrive ici, on a la permission
      stream.getTracks().forEach((track) => track.stop());
      hasPermission = true;

      await tick();
      setTimeout(() => {
        initializeScanner();
      }, 500);
    } catch (error) {
      console.error("Erreur d'accès caméra:", error);
      onError(
        "Erreur lors de la vérification des permissions. Veuillez réessayer."
      );
      if (error instanceof Error) {
        if (
          error.name === "NotAllowedError" ||
          error.name === "PermissionDeniedError"
        ) {
          if (isIOS) {
            permissionError =
              "L'accès à la caméra a été refusé. Sur iOS, vous devez autoriser l'accès dans :";
          } else {
            permissionError =
              "L'accès à la caméra a été refusé. Veuillez autoriser l'accès et réessayer.";
          }
        } else if (
          error.name === "NotFoundError" ||
          error.name === "DevicesNotFoundError"
        ) {
          permissionError = "Aucune caméra trouvée sur cet appareil.";
        } else if (
          error.name === "NotReadableError" ||
          error.name === "TrackStartError"
        ) {
          permissionError =
            "La caméra est déjà utilisée par une autre application.";
        } else if (
          error.name === "OverconstrainedError" ||
          error.name === "ConstraintNotSatisfiedError"
        ) {
          permissionError =
            "La caméra ne supporte pas la configuration demandée.";
        } else if (error.name === "TypeError" && isIOS) {
          permissionError =
            "Sur iOS, l'accès à la caméra nécessite HTTPS. Assurez-vous d'utiliser une connexion sécurisée.";
        } else {
          permissionError = `Erreur d'accès à la caméra: ${error.message}`;
        }
      }
      hasPermission = false;
    }
  }

  async function initializeScanner() {
    try {
      // Vérifier que l'élément existe
      const element = document.getElementById("qr-reader");
      if (!element) {
        console.error("Element #qr-reader non trouvé dans le DOM");
        setTimeout(() => initializeScanner(), 500);
        return;
      }
      await stopGoogleScanning();
      await startGoogleScanning();
    } catch (err) {
      console.error("Erreur d'initialisation:", err);
      if (err instanceof Error) {
        handleCameraError(err);
      }
    }
  }

  function initZoomFromVideoElement() {
    const video = document.querySelector(
      "#qr-reader video"
    ) as HTMLVideoElement | null;
    if (video && (video as any).srcObject) {
      videoEl = video;
      mediaStream = (video as any).srcObject as MediaStream;
      const tracks = mediaStream.getVideoTracks();
      videoTrack = tracks && tracks.length > 0 ? tracks[0] : null;
      setupZoomCapabilities();
    }
  }

  function setupZoomCapabilities() {
    zoomSupported = false;
    minZoom = 1;
    maxZoom = 1;
    zoomStep = 0.1;
    zoom = 1;
    cssZoom = 1;
    if (!videoTrack) {
      applyCssZoom();
      return;
    }
    const caps: any = (videoTrack as any).getCapabilities
      ? (videoTrack as any).getCapabilities()
      : null;
    if (caps && typeof caps.zoom === "object") {
      zoomSupported = true;
      minZoom = caps.zoom.min ?? 1;
      maxZoom = caps.zoom.max ?? Math.max(2, minZoom);
      zoomStep = caps.zoom.step ?? 0.1;
      zoom = Math.min(Math.max(zoom, minZoom), maxZoom);
      applyTrackZoom(zoom);
    } else {
      zoomSupported = false;
      minZoom = 1;
      maxZoom = 3;
      zoomStep = 0.1;
      zoom = 1;
      cssZoom = 1;
      applyCssZoom();
    }
  }

  async function applyTrackZoom(val: number) {
    if (!videoTrack) return;
    const clamped = Math.min(Math.max(val, minZoom), maxZoom);
    zoom = clamped;
    try {
      await (videoTrack as any).applyConstraints({ advanced: [{ zoom: clamped }] });
    } catch (e) {
      applyCssZoomFromTrackZoom();
    }
  }

  function applyCssZoomFromTrackZoom() {
    const range = maxZoom - minZoom || 1;
    const normalized = (zoom - minZoom) / range;
    cssZoom = 1 + normalized * 1.5;
    applyCssZoom();
  }

  function applyCssZoom() {
    const video = videoEl || (document.querySelector("#qr-reader video") as HTMLVideoElement | null);
    if (video) {
      const scale = cssZoom || 1;
      video.style.transformOrigin = "center center";
      video.style.transform = `scale(${scale})`;
    }
  }

  function decreaseZoom() {
    if (zoomSupported) {
      applyTrackZoom(zoom - zoomStep);
    } else {
      cssZoom = Math.max(1, (cssZoom || 1) - 0.1);
      applyCssZoom();
    }
  }

  function increaseZoom() {
    if (zoomSupported) {
      applyTrackZoom(zoom + zoomStep);
    } else {
      const targetMax = 3;
      cssZoom = Math.min(targetMax, (cssZoom || 1) + 0.1);
      applyCssZoom();
    }
  }

  function setZoomFromSlider(v: number) {
    if (zoomSupported) {
      applyTrackZoom(v);
    } else {
      cssZoom = v;
      applyCssZoom();
    }
  }

  async function startGoogleScanning() {
    const container = document.getElementById("qr-reader");
    if (!container) return;
    container.innerHTML = "";
    const vid = document.createElement("video");
    vid.setAttribute("playsinline", "true");
    vid.setAttribute("autoplay", "true");
    vid.setAttribute("muted", "true");
    vid.style.width = "100%";
    vid.style.height = "100%";
    vid.style.objectFit = "cover";
    vid.style.borderRadius = "12px";
    container.appendChild(vid);
    videoEl = vid;
    const constraints: MediaStreamConstraints = {
      video: {
        facingMode: "environment",
        width: { ideal: 1280 },
        height: { ideal: 720 },
      },
      audio: false,
    };
    mediaStream = await navigator.mediaDevices.getUserMedia(constraints);
    vid.srcObject = mediaStream;
    await vid.play().catch(() => {});
    const tracks = mediaStream.getVideoTracks();
    videoTrack = tracks && tracks.length > 0 ? tracks[0] : null;
    setupZoomCapabilities();
    applyCustomStyles();
    fixIOSVideoElement();
    let SupportedDetector = (window as any).BarcodeDetector;
    let formats: string[] | undefined = undefined;
    if (SupportedDetector && SupportedDetector.getSupportedFormats) {
      try {
        formats = await SupportedDetector.getSupportedFormats();
      } catch (_) {}
    }
    if (!SupportedDetector || (formats && !formats.includes("qr_code"))) {
      const { BarcodeDetector: Polyfill } = await import("barcode-detector");
      SupportedDetector = Polyfill as any;
    }
    barcodeDetector = new SupportedDetector({ formats: ["qr_code"] });
    if (googleScanTimer) {
      window.clearInterval(googleScanTimer);
      googleScanTimer = null;
    }
    googleScanTimer = window.setInterval(async () => {
      if (!videoEl || videoEl.readyState < 2) return;
      try {
        const barcodes = await barcodeDetector.detect(videoEl);
        if (barcodes && barcodes.length > 0) {
          const qr = barcodes.find((b: any) => (b.format || b.rawValue) && (b.format === "qr_code" || true));
          const value = qr?.rawValue || barcodes[0]?.rawValue;
          if (value) {
            await stopGoogleScanning();
            onScan(value);
            onClose();
            handleClose();
          }
        }
      } catch (e) {}
    }, 200);
    isScanning = true;
  }

  async function stopGoogleScanning() {
    if (googleScanTimer) {
      window.clearInterval(googleScanTimer);
      googleScanTimer = null;
    }
    if (videoEl) {
      try {
        await videoEl.pause();
      } catch {}
      videoEl.srcObject = null as any;
      videoEl = null;
    }
    if (mediaStream) {
      mediaStream.getTracks().forEach((t) => t.stop());
      mediaStream = null;
    }
    videoTrack = null;
    isScanning = false;
  }

  function applyCustomStyles() {
    const scannerContainer = document.querySelector("#qr-reader");
    if (!scannerContainer) return;

    scannerContainer.classList.add(
      "relative",
      "w-full",
      "h-full",
      "min-h-[400px]"
    );

    const video = scannerContainer.querySelector("video");
    if (video) {
      video.classList.add("rounded-xl", "w-full", "h-full", "object-cover");
    }

    const scanRegion = scannerContainer.querySelector("#qr-shaded-region");
    if (scanRegion) {
      scanRegion.classList.add("absolute", "inset-0");
    }

    const buttons = scannerContainer.querySelectorAll("button");
    buttons.forEach((button) => {
      button.classList.add("btn-secondary", "text-sm", "m-2");
    });
  }

  function fixIOSVideoElement() {
    if (!isIOS) return;

    const video = document.querySelector(
      "#qr-reader video"
    ) as HTMLVideoElement;
    if (video) {
      video.setAttribute("playsinline", "true");
      video.setAttribute("webkit-playsinline", "true");
      video.setAttribute("muted", "true");
      video.setAttribute("autoplay", "true");

      video.play().catch((e) => console.error("Erreur play video:", e));

      video.style.width = "100%";
      video.style.height = "100%";
      video.style.objectFit = "cover";
      video.style.borderRadius = "12px";
    }
  }

  function handleCameraError(error: Error) {
    if (
      error.name === "NotAllowedError" ||
      error.name === "PermissionDeniedError"
    ) {
      if (isIOS) {
        permissionError =
          "L'accès à la caméra a été refusé. Sur iOS, vous devez autoriser l'accès dans :";
      } else {
        permissionError =
          "L'accès à la caméra a été refusé. Veuillez autoriser l'accès et réessayer.";
      }
    } else if (
      error.name === "NotFoundError" ||
      error.name === "DevicesNotFoundError"
    ) {
      permissionError = "Aucune caméra trouvée sur cet appareil.";
    } else if (
      error.name === "NotReadableError" ||
      error.name === "TrackStartError"
    ) {
      permissionError =
        "La caméra est déjà utilisée par une autre application.";
    } else {
      permissionError = `Erreur d'accès à la caméra: ${error.message}`;
    }
    hasPermission = false;
  }

  async function retryPermission() {
    isChecking = true;
    permissionError = "";
    hasPermission = false;
    await checkCameraPermission();
  }

  async function cleanupScanner() {
    await stopGoogleScanning();
  }

  async function handleClose() {
    // Nettoyer avant de fermer
    await cleanupScanner();
    onClose();
  }

  function openIOSSettings() {
    // On ne peut pas ouvrir directement les réglages, mais on peut donner des instructions claires
    alert("Allez dans Réglages > Safari > Caméra > Autoriser");
  }
</script>

<div class="fixed inset-0 z-50 bg-black flex flex-col">
  <!-- Header -->
  <div class="glass-effect border-b border-white/10 px-4 py-3 safe-top">
    <div class="flex justify-between items-center">
      <h3 class="text-lg font-semibold">Scanner QR Code</h3>
      <button
        on:click={handleClose}
        class="p-2 rounded-lg hover:bg-white/10 transition-colors"
        aria-label="Fermer"
      >
        <X />
      </button>
    </div>
  </div>

  <!-- Scanner ou messages d'erreur -->
  <div class="flex-1 flex flex-col p-4 gap-3">
    <div class="flex items-center justify-end">
      <div class="flex items-center gap-2">
        <button class="btn-secondary px-3" on:click={decreaseZoom}>-</button>
        {#if zoomSupported}
          <input type="range" min={minZoom} max={maxZoom} step={zoomStep} bind:value={zoom} on:input={(e: any) => setZoomFromSlider(parseFloat(e.currentTarget.value))} class="w-40" />
        {:else}
          <input type="range" min={1} max={3} step={0.1} bind:value={cssZoom} on:input={(e: any) => setZoomFromSlider(parseFloat(e.currentTarget.value))} class="w-40" />
        {/if}
        <button class="btn-secondary px-3" on:click={increaseZoom}>+</button>
      </div>
    </div>
    {#if isChecking}
      <div class="text-center">
        <div
          class="inline-block animate-spin rounded-full h-12 w-12 border-b-2 border-red-600 mb-4"
        ></div>
        <p class="text-gray-400">Vérification des permissions...</p>
      </div>
    {:else if !hasPermission && permissionError}
      <div class="max-w-sm text-center">
        <div class="mb-6">
          <TriangleAlert class="w-12 h-12 mx-auto text-yellow-400" />
        </div>
        <h4 class="text-lg font-semibold mb-2">Permission caméra requise</h4>
        <p class="text-gray-400 mb-6">{permissionError}</p>

        {#if isIOS && permissionError.includes("refusé")}
          <div class="space-y-4">
            <div class="text-left bg-white/5 rounded-xl p-4 text-sm">
              <p class="font-semibold mb-2">
                📱 Instructions pour iPhone/iPad :
              </p>
              <ol class="space-y-2 text-gray-300">
                <li class="flex gap-2">
                  <span class="text-red-500">1.</span>
                  <span>Ouvrez l'app <strong>Réglages</strong> (icône ⚙️)</span>
                </li>
                <li class="flex gap-2">
                  <span class="text-red-500">2.</span>
                  <span>Descendez et tapez sur <strong>Safari</strong></span>
                </li>
                <li class="flex gap-2">
                  <span class="text-red-500">3.</span>
                  <span
                    >Dans "Réglages pour les sites web", tapez <strong
                      >Caméra</strong
                    ></span
                  >
                </li>
                <li class="flex gap-2">
                  <span class="text-red-500">4.</span>
                  <span>Sélectionnez <strong>Autoriser</strong></span>
                </li>
                <li class="flex gap-2">
                  <span class="text-red-500">5.</span>
                  <span>Revenez ici et tapez <strong>Réessayer</strong></span>
                </li>
              </ol>
            </div>

            <div class="flex gap-3">
              <button on:click={retryPermission} class="btn-primary flex-1">
                Réessayer
              </button>
              <button on:click={openIOSSettings} class="btn-secondary flex-1">
                Voir instructions
              </button>
            </div>
          </div>
        {:else}
          <button on:click={retryPermission} class="btn-primary">
            Réessayer
          </button>
          <div class="mt-6 text-sm text-gray-500">
            <p class="mb-2">Pour autoriser l'accès :</p>
            <ol class="text-left space-y-1">
              <li>1. Vérifiez que vous êtes en HTTPS</li>
              <li>2. Autorisez l'accès à la caméra quand demandé</li>
              <li>3. Rechargez la page si nécessaire</li>
            </ol>
          </div>
        {/if}
      </div>
    {:else if hasPermission}
      <!-- <div>qrcode reader actif</div> -->
      <div
        id="qr-reader"
        bind:this={scannerElement}
        class="w-full max-w-sm h-50"
      >
        <!-- Le scanner sera rendu ici -->
      </div>
    {/if}
  </div>

  <!-- Instructions -->
  {#if hasPermission && !isChecking}
    <div class="glass-effect border-t border-white/10 px-4 py-4 safe-bottom">
      <p class="text-center text-sm text-gray-400">
        Placez le QR code dans le cadre pour le scanner
      </p>
    </div>
  {/if}
</div>
