import { AxiosError } from "axios";
import { useEffect, useState } from "react";
import AxiosInstance from "../lib/AxiosInstance";
import { Offer, OfferDealStackResponse } from "../types";
import useNotification from "./useNotification";
import useSetBackdrop from "./useSetBackdrop";
import { useGetUserConfig } from "./useUserConfig";

const useCode = () => {
  const [deal, setDeal] = useState<Offer>();
  const [code, setResponse] = useState<OfferDealStackResponse>();
  const userConfig = useGetUserConfig();
  const setBackdrop = useSetBackdrop();
  const notification = useNotification();

  useEffect(() => {
    const get = async () => {
      try {
        setBackdrop(true);
        const response = await AxiosInstance.post(
          `/deals/${deal?.dealUuid}`,
          null,
          userConfig
            ? {
                params: {
                  store: userConfig.storeId,
                },
              }
            : undefined
        );
        setResponse(response.data as OfferDealStackResponse);
      } catch (error) {
        notification({ msg: (error as AxiosError).message, variant: "error" });
      } finally {
        setBackdrop(false);
      }
    };

    if (deal) {
      get();
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [deal]);

  const remove = async () => {
    try {
      setBackdrop(true);
      const response = await AxiosInstance.delete(`/deals/${deal?.dealUuid}`);
      return response.data as OfferDealStackResponse;
    } finally {
      setBackdrop(false);
    }
  };

  return {
    code,
    setDeal,
    remove,
  };
};

export default useCode;
