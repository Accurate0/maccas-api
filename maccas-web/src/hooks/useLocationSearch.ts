import { useState } from "react";
import AxiosInstance from "../lib/AxiosInstance";
import { RestaurantInformation } from "../types";

const useLocationSearch = () => {
  const [results, setResults] = useState<RestaurantInformation | undefined>();
  const search = async (text: string) => {
    const response = await AxiosInstance.get("/locations/search", {
      params: {
        text: encodeURIComponent(text),
      },
    });

    setResults(response.data as RestaurantInformation);

    return response.data as RestaurantInformation;
  };

  return {
    search,
    results,
  };
};

export default useLocationSearch;
