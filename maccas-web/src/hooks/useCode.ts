import { AxiosError } from "axios";
import { useEffect, useState } from "react";
import AxiosInstance from "../lib/AxiosInstance";
import { OfferDealStackResponse } from "../types";
import useNotification from "./useNotification";
import useSelectedDeal from "./useSelectedDeal";
import useSetBackdrop from "./useSetBackdrop";
import { useGetUserConfig } from "./useUserConfig";

const useCode = () => {
  const [deal, setDeal] = useSelectedDeal();
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
      await AxiosInstance.delete(`/deals/${deal?.dealUuid}`);
      setDeal(null);
    } finally {
      setBackdrop(false);
    }
  };

  const refreshCode = async () => {
    try {
      setBackdrop(true);
      const response = await AxiosInstance.get(
        `/code/${deal?.dealUuid}`,
        userConfig
          ? {
              params: {
                store: userConfig.storeId,
              },
            }
          : undefined
      );
      setResponse(response.data);
      return response.data as OfferDealStackResponse;
    } finally {
      setBackdrop(false);
    }
  };

  return {
    code,
    setDeal,
    remove,
    refreshCode,
  };
};

export default useCode;
