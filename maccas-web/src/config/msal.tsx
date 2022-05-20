import { Configuration, PublicClientApplication } from "@azure/msal-browser";

export const MSALConfig: Configuration = {
  auth: {
    clientId: "871d2afa-2389-401d-aaa9-94bf38d05c1d",
    authority: "https://login.microsoftonline.com/b1f3a0a4-f4e2-4300-b952-88f3dc55ee9b",
  },
  cache: {
    cacheLocation: "sessionStorage",
    storeAuthStateInCookie: false,
  },
};

export const LoginRequest = {
  scopes: ["https://apib2clogin.onmicrosoft.com/871d2afa-2389-401d-aaa9-94bf38d05c1d/.default"],
};

export const MSALInstance = new PublicClientApplication(MSALConfig);
