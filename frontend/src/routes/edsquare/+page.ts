import type { PageLoad } from './$types';
import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { getCurrentUser, loadUsers, checkAuth } from '$lib/api';
import { get } from 'svelte/store';
import { currentUser } from '$lib/stores';
import type { PublicUserResponse } from '$lib/types';

// Désactiver le prerender pour cette page car elle nécessite l'authentification
export const prerender = false;

export const load: PageLoad = async ({ fetch }) => {
  if (browser) {
    const authenticated = await checkAuth(fetch);
    
    if (!authenticated) {
      throw redirect(302, '/login');
    }

    try {
      const [user, loadedUsers] = await Promise.all([
        getCurrentUser(fetch),
        loadUsers(fetch)
      ]);
      
      currentUser.set(user);

      // Marquer les utilisateurs avec JWT expiré et vérifier les prérequis EDSquare
      const currentUserId = get(currentUser)?.id;
      const usersWithPrereqs = loadedUsers.map((u) => {
        u.jwtIsExpired =
          u.jwtExpiresAt === undefined ||
          u.jwtExpiresAt === null ||
          new Date(u.jwtExpiresAt) < new Date();
        
        // Note: On ne peut pas vérifier la signature et les cookies EDSquare ici
        // car ces informations ne sont pas dans PublicUserResponse
        // On le fera côté client
        
        return u;
      });

      // Trier les utilisateurs
      const sortedUsers = usersWithPrereqs.sort((a, b) => {
        const aIsCurrent = currentUserId === a.id;
        const bIsCurrent = currentUserId === b.id;

        if (aIsCurrent && !bIsCurrent) return -1;
        if (!aIsCurrent && bIsCurrent) return 1;

        if (a.jwtIsExpired !== b.jwtIsExpired) {
          return a.jwtIsExpired ? 1 : -1;
        }

        return a.id.localeCompare(b.id);
      });

      return {
        users: sortedUsers as PublicUserResponse[]
      };
    } catch (e) {
      console.error("Erreur lors du chargement:", e);
      return {
        users: [] as PublicUserResponse[]
      };
    }
  }
  
  return {
    users: [] as PublicUserResponse[]
  };
};
