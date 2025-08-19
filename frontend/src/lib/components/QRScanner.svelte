<script lang="ts">
  import { onMount, onDestroy, tick } from "svelte";
  import { Html5Qrcode } from "html5-qrcode";
  import { TriangleAlert, X } from "@lucide/svelte";

  export let onScan: (result: string) => void;
  export let onClose: () => void;
  export let onError: (error: string) => void;

  let scanner: Html5Qrcode | null = null;
  let scannerElement: HTMLElement;
  let hasPermission: boolean = false;
  let isChecking: boolean = true;
  let permissionError: string = "";
  let isIOS: boolean = false;
  let isScanning: boolean = false;

  onMount(async () => {
    // Détecter iOS
    isIOS =
      /iPad|iPhone|iPod/.test(navigator.userAgent) && !(window as any).MSStream;

    // Vérifier d'abord les permissions
    await checkCameraPermission();
  });

  onDestroy(() => {
    if (scanner) {
      scanner.stop();
      scanner.clear();
      scanner = null;
    }
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

      // Nettoyer toute instance précédente
      await cleanupScanner();

      console.log("Initialisation du scanner QR Code");
      scanner = new Html5Qrcode("qr-reader");
      console.log("Scanner créé avec succès");

      // Obtenir les caméras disponibles
      const cameras = await Html5Qrcode.getCameras();
      console.log("Caméras trouvées:", cameras);

      if (cameras && cameras.length > 0) {
        // Préférer la caméra arrière
        const backCamera =
          cameras.find(
            (camera) =>
              camera.label.toLowerCase().includes("back") ||
              camera.label.toLowerCase().includes("rear") ||
              camera.label.toLowerCase().includes("environment")
          ) || cameras[0];

        console.log("Caméra sélectionnée:", backCamera);
        await startScanning(backCamera.id);
      } else {
        throw new Error("Aucune caméra trouvée");
      }
    } catch (err) {
      console.error("Erreur d'initialisation:", err);
      if (err instanceof Error) {
        handleCameraError(err);
      }
    }
  }

  async function startScanning(cameraId: string) {
    try {
      if (!scanner) {
        console.error("Scanner non initialisé");
        return;
      }

      const config = {
        fps: 10,
        qrbox: { width: 250, height: 250 },
        aspectRatio: isIOS ? 1.0 : 1.7777778,
        videoConstraints: {
          deviceId: cameraId,
          facingMode: "environment",
        },
      };

      console.log("Démarrage du scan avec config:", config);

      // Démarrer le scan
      await scanner.start(
        cameraId,
        config,
        async (decodedText: string) => {
          console.log("QR Code détecté:", decodedText);

          // Arrêter le scanner avant de fermer
          await stopScanner();

          // Appeler les callbacks
          onScan(decodedText);
          onClose();
          handleClose();
        },
        (errorMessage: string) => {
          // Ignorer les erreurs de scan normales
          if (!errorMessage.includes("NotFoundException")) {
            console.warn("Erreur de scan:", errorMessage);
          }
        }
      );

      isScanning = true;
      console.log("Scanner démarré avec succès");

      // Appliquer les styles après le démarrage
      setTimeout(() => {
        applyCustomStyles();
        fixIOSVideoElement();
      }, 500);
    } catch (err) {
      console.error("Erreur de démarrage du scan:", err);
      permissionError = `Erreur de démarrage: ${err}`;
      hasPermission = false;
    }
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
      if (!button.id?.includes("html5-qrcode")) {
        button.classList.add("btn-secondary", "text-sm", "m-2");
      }
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
    if (scanner) {
      try {
        // Arrêter le scan si en cours
        if (isScanning) {
          await scanner.stop();
          console.log("Scanner arrêté");
        }

        // Clear pour libérer les ressources
        scanner.clear();
        console.log("Scanner nettoyé");
      } catch (err) {
        console.error("Erreur lors du nettoyage du scanner:", err);
      } finally {
        scanner = null;
        isScanning = false;
      }
    }
  }

  async function stopScanner() {
    if (scanner && isScanning) {
      try {
        await scanner.stop();
        isScanning = false;
      } catch (err) {
        console.error("Erreur lors de l'arrêt du scanner:", err);
      }
    }
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
  <div class="flex-1 flex items-center justify-center p-4">
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
