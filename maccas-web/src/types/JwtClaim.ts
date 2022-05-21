
export interface JwtClaim { exp: bigint, nbf: bigint, ver: string, iss: string, sub: string, aud: string, nonce: string, iat: bigint, auth_time: bigint, oid: string, name: string, tfp: string, }