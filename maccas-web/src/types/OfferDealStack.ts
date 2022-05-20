import type { DealStack } from "./DealStack";

export interface OfferDealStack { randomCode: string, expirationTime: string, dealStack: Array<DealStack> | null, }