import type { PageLoad } from './$types';
import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { checkAuth } from '$lib/api';

export const load: PageLoad = async ({ fetch }) => {
    // Vérification d'authentification uniquement côté client
    if (browser) {
        try {
            const authenticated = await checkAuth(fetch);
            
            if (authenticated) {
                throw redirect(302, '/');
            }
        } catch (error) {
            // Si c'est une redirection, la propager
            if (error && typeof error === 'object' && 'status' in error && error.status === 302) {
                throw error;
            }
            // Sinon, on reste sur la page de login (utilisateur non authentifié)
        }
    }

    return;
};