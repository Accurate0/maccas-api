import type { Address } from "./Address";
import type { Location } from "./Location";

export interface Restaurant { restaurantStatus: string, address: Address, location: Location, name: string, nationalStoreNumber: bigint, status: bigint, timeZone: string, phoneNumber: string | null, }