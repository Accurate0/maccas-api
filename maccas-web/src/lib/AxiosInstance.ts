import axios, { AxiosRequestConfig } from "axios";
import { PUBLIC_API_KEY } from "../config/api";
import { LoginRequest, MSALInstance } from "../config/msal";

const AxiosInstance = axios.create({
  baseURL: "https://api.anurag.sh/maccas/v2",
});

const fetchAccessToken = (): Promise<string> => {
  const accounts = MSALInstance.getAllAccounts();

  return new Promise((resolve) => {
    MSALInstance.acquireTokenSilent({
      ...LoginRequest,
      account: accounts[0] ?? undefined,
    }).then((response) => {
      resolve(response.accessToken);
    });
  });
};

AxiosInstance.interceptors.request.use(
  async (config: AxiosRequestConfig) => {
    const accessToken = await fetchAccessToken();
    config.headers = config.headers ?? {};

    if (accessToken) {
      config.headers["Authorization"] = "Bearer " + accessToken;
    }
    config.headers["X-Api-Key"] = PUBLIC_API_KEY;
    config.headers["Content-Type"] = "application/json";
    return config;
  },
  (error) => {
    Promise.reject(error);
  }
);

export default AxiosInstance;
