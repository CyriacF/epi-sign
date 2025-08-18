import { browser } from '$app/environment';
import { isAuthenticated, currentUser } from './stores';
import type { 
    LoginPayload, 
    RegisterPayload, 
    SignPayload, 
    PublicUserResponse,
    ApiError,
    User, 
    JwtPayload,
    UserSignResponse,
    UpdateUserPayload
} from './types';

const API_BASE = '/api';

// Configuration pour les cookies
const fetchConfig: RequestInit = {
    credentials: 'include', // IMPORTANT: inclure les cookies
    headers: {
        'Content-Type': 'application/json',
    }
};

async function apiCall<T>(
    endpoint: string, 
    options: RequestInit = {},
    customFetch?: typeof fetch
): Promise<T> {
    const fetchFn = customFetch || fetch;
    
    const response = await fetchFn(`${API_BASE}${endpoint}`, {
        ...fetchConfig,
        ...options,
        headers: {
            ...fetchConfig.headers,
            ...options.headers
        }
    });

    if (!response.ok) {
        // Si 401, déconnecter l'utilisateur (seulement côté client)
        if (response.status === 401 && endpoint !== '/auth/login' && browser) {
            isAuthenticated.set(false);
            currentUser.set(null);
        }
        
        const error: ApiError = {
            status: response.status,
            message: `HTTP error! status: ${response.status}`
        };
        throw error;
    }

    const contentType = response.headers.get('content-type');
    if (contentType && contentType.includes('application/json')) {
        return await response.json() as T;
    }
    return await response.text() as T;
}

export async function login(username: string, password: string, customFetch?: typeof fetch): Promise<void> {
    const payload: LoginPayload = { username, password };
    
    await apiCall<void>('/auth/login', {
        method: 'POST',
        body: JSON.stringify(payload)
    }, customFetch);
    
    // Récupérer les infos utilisateur après connexion (seulement côté client)
    if (browser) {
        try {
            const user = await getCurrentUser(customFetch);
            isAuthenticated.set(true);
            currentUser.set(user);
        } catch {
            // Si on ne peut pas récupérer l'utilisateur, on considère qu'on est connecté avec les infos minimales
            isAuthenticated.set(true);
            currentUser.set({ id: '', username });
        }
    }
}

export async function register(username: string, password: string, key: string, customFetch?: typeof fetch): Promise<void> {
    const payload: RegisterPayload = { username, password, key };
    
    await apiCall<void>('/auth/register', {
        method: 'POST',
        body: JSON.stringify(payload)
    }, customFetch);
}

export async function logout(customFetch?: typeof fetch): Promise<void> {
    try {
        await apiCall<void>('/auth/logout', { method: 'POST' }, customFetch);
    } finally {
        if (browser) {
            isAuthenticated.set(false);
            currentUser.set(null);
        }
    }
}

export async function getCurrentUser(customFetch?: typeof fetch): Promise<User> {
    return await apiCall<User>('/users/me', {}, customFetch);
}

export async function loadUsers(customFetch?: typeof fetch): Promise<PublicUserResponse[]> {
    return await apiCall<PublicUserResponse[]>('/users', {}, customFetch);
}

export async function signUsers(ulids: string[], url: string, customFetch?: typeof fetch): Promise<UserSignResponse[]> {
    const payload: SignPayload = { ulids, url };
    
    return await apiCall<UserSignResponse[]>('/sign', {
        method: 'POST',
        body: JSON.stringify(payload)
    }, customFetch);
}

export async function updateUserProfile(payload: UpdateUserPayload, customFetch?: typeof fetch): Promise<void> {
    const user: User = await apiCall<User>('/users/me', {
        method: 'PATCH',
        body: JSON.stringify(payload)
    }, customFetch);

    if (browser) {
        currentUser.set(user);
    }
}

// Fonction pour vérifier l'état de l'authentification
export async function checkAuth(customFetch?: typeof fetch): Promise<boolean> {
    try {
        const user = await getCurrentUser(customFetch);
        if (browser) {
            isAuthenticated.set(true);
            currentUser.set(user);
        }
        return true;
    } catch {
        if (browser) {
            isAuthenticated.set(false);
            currentUser.set(null);
        }
        return false;
    }
}

export async function updateUserJWT(jwt: string, customFetch?: typeof fetch): Promise<void> {
    const payload: JwtPayload = { jwt };
    
    await apiCall<void>('/users/me/update-jwt', {
        method: 'POST',
        body: JSON.stringify(payload)
    }, customFetch);
}
