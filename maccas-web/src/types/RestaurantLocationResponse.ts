import type { RestaurantLocationList } from "./RestaurantLocationList";
import type { Status } from "./Status";

export interface RestaurantLocationResponse { status: Status, response: RestaurantLocationList | null, }