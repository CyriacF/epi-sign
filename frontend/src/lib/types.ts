export interface User {
    id: string;
    username: string;
    jwtExpiresAt?: string | null;
    jwtIntraEpitech?: string | null;
}

export interface PublicUserResponse {
    id: string;
    username: string;
    jwtExpiresAt?: string | null;
    jwtIsExpired?: boolean;
}

export interface LoginPayload {
    username: string;
    password: string;
}

export interface RegisterPayload {
    username: string;
    password: string;
    key: string
}

export interface SignPayload {
    ulids: string[];
    url: string;
}

export type SignResponse = 
    | 'success'
    | 'tokenExpired'
    | 'tokenNotFound'
    | 'alreadySigned'
    | 'unknownError'
    | 'serviceUnavailable';

export interface UserSignResponse {
    response: SignResponse,
    ulid: string
}

export interface JwtPayload {
    jwt: string;
}

export interface ApiError {
    status: number;
    message: string;
}

export interface UpdateUserPayload {
    username?: string | null;
    old_password?: string | null;
    new_password?: string | null;
}

export interface DashboardPageData {
    users: PublicUserResponse[];
    error?: string;
}

export interface LoginPageData {}
export interface RegisterPageData {}
export interface IndexPageData {}
export interface SelfSignPageData {}
