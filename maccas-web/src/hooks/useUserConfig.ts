import { AxiosError } from "axios";
import { useEffect } from "react";
import { atom, useRecoilState, useRecoilValue } from "recoil";
import AxiosInstance from "../lib/AxiosInstance";
import { UserOptions } from "../types";
import useNotification from "./useNotification";
import useSetBackdrop from "./useSetBackdrop";

const UserConfig = atom<UserOptions>({
  key: "userConfig",
  default: undefined,
});

export const useGetUserConfig = () => {
  return useRecoilValue(UserConfig);
};

const useUserConfig = () => {
  const [config, setConfig] = useRecoilState(UserConfig);
  const setBackdrop = useSetBackdrop();
  const notification = useNotification();

  useEffect(() => {
    const get = async () => {
      try {
        setBackdrop(true);
        const response = await AxiosInstance.get("/user/config");
        setConfig(response.data as UserOptions);
      } catch (error) {
        notification({ msg: (error as AxiosError).message, variant: "error" });
      } finally {
        setBackdrop(false);
      }
    };

    get();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  const updateConfig = async (c: UserOptions) => {
    setConfig(c);
    await AxiosInstance.patch("/user/config", c);
  };

  return {
    config,
    updateConfig,
  };
};

export default useUserConfig;
