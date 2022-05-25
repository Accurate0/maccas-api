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
        const err = error as AxiosError;
        if (err.response?.status !== 404) {
          notification({ msg: err.message, variant: "error" });
        }
      } finally {
        setBackdrop(false);
      }
    };

    get();
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return {
    config,
  };
};

export const useUpdateUserConfig = () => {
  const [, setConfig] = useRecoilState(UserConfig);

  const updateConfig = async (c: UserOptions) => {
    setConfig(c);
    await AxiosInstance.patch("/user/config", c);
  };

  return updateConfig;
};

export default useUserConfig;
