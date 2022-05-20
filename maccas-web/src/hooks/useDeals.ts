import { useEffect, useState } from "react";
import AxiosInstance from "../lib/AxiosInstance";
import { Offer } from "../types";

const useDeals = () => {
  const [state, setState] = useState<Offer[]>();
  useEffect(() => {
    const get = async () => {
      const response = await AxiosInstance.get("/deals");
      setState(response.data as Offer[]);
    };

    get();
  }, []);

  return state;
};

export default useDeals;
