<script lang="ts">
  import "../app.css";
  import { page } from "$app/stores";
  import { isAuthenticated, currentUser } from "$lib/stores";
  import ProfileMenu from "$lib/components/ProfileMenu.svelte";
  import JWTUpdater from "$lib/components/JWTUpdater.svelte";
  import ProfileUpdater from "$lib/components/ProfileUpdater.svelte";
  import { goto } from "$app/navigation";

  let showJWTUpdater = false;
  let showProfileUpdater = false;
  let isMobile = false;
  let scrolled = false;

  $: isMobile = typeof window !== "undefined" && window.innerWidth < 640;
  $: hideNavbar = ["/login", "/register", "/home"].includes($page.url.pathname);

  function handleUpdateJWT() { showJWTUpdater = true; }
  function handleCloseJWTUpdater() { showJWTUpdater = false; }
  function handleUpdateProfile() { goto("/profile"); }
  function handleCloseProfileUpdater() { showProfileUpdater = false; }
  function handleGoingHome() { goto("/"); }

  if (typeof window !== "undefined") {
    const onScroll = () => { scrolled = window.scrollY > 8; };
    window.addEventListener("scroll", onScroll);
  }
</script>

<div class="min-h-screen relative">
  <div class="fixed inset-0 -z-10">
    <div class="absolute inset-0 bg-gradient-to-b from-black via-black to-zinc-900"></div>
  </div>

  {#if $isAuthenticated && !hideNavbar}
    <nav class="sticky top-0 z-50 safe-top transition-colors">
      <div class="px-4 py-3 sm:px-6 sm:py-4">
        <div class="flex justify-between items-center">
          <button
            class="text-xl sm:text-2xl font-bold gradient-text"
            onclick={handleGoingHome}
            aria-label="Aller à l'accueil"
          >
            EpiSign
          </button>

          <div class="flex items-center gap-6">
            <a href="/" class="hidden sm:inline text-sm tracking-widest uppercase hover:opacity-80">Accueil</a>
            <a href="/dashboard" class="hidden sm:inline text-sm tracking-widest uppercase hover:opacity-80">Dashboard</a>
            <a href="/edsquare" class="hidden sm:inline text-sm tracking-widest uppercase hover:opacity-80">EDSquare</a>
            <a href="/roadmap" class="hidden sm:inline text-sm tracking-widest uppercase hover:opacity-80">Roadmap</a>
            <ProfileMenu {isMobile} on:updateJWT={handleUpdateJWT} on:updateProfile={handleUpdateProfile} />
          </div>
        </div>
      </div>
      <div class={scrolled ? "absolute inset-0 -z-10 bg-black/80 backdrop-blur-md" : "absolute inset-0 -z-10"}></div>
    </nav>
  {/if}

  <main class="safe-bottom">
    <slot />
  </main>
</div>

<JWTUpdater isOpen={showJWTUpdater} on:close={handleCloseJWTUpdater} />
<ProfileUpdater isOpen={showProfileUpdater} on:close={handleCloseProfileUpdater} />
