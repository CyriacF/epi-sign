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

export interface User {
    id: string;
    username: string;
    jwtExpiresAt?: string | null;
    jwtIntraEpitech?: string | null;
    signatureManuscrite?: string | null;
}

export interface SaveSignaturePayload {
    signature: string;
}

export interface ValidateEdsquarePayload {
    code: string;
    planning_event_id: string;
}

export interface ValidateEdsquareResponse {
    success: boolean;
    message: string;
    code: string;
    planning_event_id?: string | null;
}

// Réponse pour la validation multi-utilisateurs
export interface EdsquareUserValidationResult {
    userId: string;
    username: string;
    success: boolean;
    message: string;
}

export interface ValidateEdsquareMultiResponse {
    globalSuccess: boolean;
    results: EdsquareUserValidationResult[];
}

export interface LoginEdsquarePayload {
    email: string;
    password: string;
}

export interface LoginEdsquareResponse {
    success: boolean;
    message: string;
}

export interface EdsquareStatusResponse {
    has_signature: boolean;
    has_cookies: boolean;
    has_saved_credentials: boolean;
    is_ready: boolean;
}

export interface EdsquareEligibleUser {
    id: string;
    username: string;
}

export interface EdsquareEligibleUsersResponse {
    users: EdsquareEligibleUser[];
}

export interface EdsquarePlanningEvent {
    id: number;
    title: string;
    target?: string;
    start: string;
    end: string;
    event_type?: string;
    registrable?: boolean;
}

export interface EdsquarePlanningEventsResponse {
    events: EdsquarePlanningEvent[];
}

export interface UserPlanningEvents {
    user_id: string;
    username: string;
    events: EdsquarePlanningEvent[];
    error?: string | null;
}

export interface PlanningEventsForUsersResponse {
    user_events: UserPlanningEvents[];
}

export interface DashboardPageData {
    users: PublicUserResponse[];
    error?: string;
}

export interface LoginPageData {}
export interface RegisterPageData {}
export interface IndexPageData {}
export interface SelfSignPageData {}
export interface EdsquarePageData {
    users: PublicUserResponse[];
    error?: string;
}
