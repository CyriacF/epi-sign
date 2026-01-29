import type { PageLoad } from './$types';
import { browser } from '$app/environment';
import { redirect } from '@sveltejs/kit';
import { checkAuth, getCurrentUser } from '$lib/api';
import { currentUser } from '$lib/stores';

export const prerender = false;

export const load: PageLoad = async ({ fetch }) => {
    if (browser) {
        const authenticated = await checkAuth(fetch);
        
        if (!authenticated) {
            throw redirect(302, '/login');
        }

        try {
            const user = await getCurrentUser(fetch);
            currentUser.set(user);
        } catch (e) {
            console.error("Erreur lors du chargement de l'utilisateur:", e);
        }
    }
    
    return {};
};
