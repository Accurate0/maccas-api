import { useEffect, useState } from "react";
import AxiosInstance from "../lib/AxiosInstance";
import { Offer, OfferDealStackResponse } from "../types";

const useCode = () => {
  const [deal, setDeal] = useState<Offer>();
  const [isDone, setIsDone] = useState(false);
  const [code, setResponse] = useState<OfferDealStackResponse>();

  useEffect(() => {
    const get = async () => {
      try {
        const response = await AxiosInstance.post(`/deals/${deal?.dealUuid}`);
        setResponse(response.data as OfferDealStackResponse);
      } finally {
        setIsDone(true);
      }
    };

    if (deal) {
      get();
    }
  }, [deal]);

  const remove = async () => {
    setIsDone(false);
    try {
      const response = await AxiosInstance.delete(`/deals/${deal?.dealUuid}`);
      return response.data as OfferDealStackResponse;
    } finally {
      setIsDone(true);
    }
  };

  return {
    code,
    setDeal,
    isDone,
    remove,
  };
};

export default useCode;
