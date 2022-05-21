import { useEffect } from "react";
import { atom, useRecoilState, useRecoilValue } from "recoil";
import AxiosInstance from "../lib/AxiosInstance";
import { UserOptions } from "../types";

const UserConfig = atom<UserOptions>({
  key: "userConfig",
  default: undefined,
});

export const useGetUserConfig = () => {
  return useRecoilValue(UserConfig);
};

const useUserConfig = () => {
  const [config, setConfig] = useRecoilState(UserConfig);

  useEffect(() => {
    const get = async () => {
      try {
        const response = await AxiosInstance.get("/user/config");
        setConfig(response.data as UserOptions);
      } finally {
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
