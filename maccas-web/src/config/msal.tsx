import { Configuration, PublicClientApplication } from "@azure/msal-browser";

export const MSALConfig: Configuration = {
  auth: {
    clientId: "871d2afa-2389-401d-aaa9-94bf38d05c1d",
    authority: "https://apib2clogin.b2clogin.com/apib2clogin.onmicrosoft.com/B2C_1_signin",
    knownAuthorities: ["https://apib2clogin.b2clogin.com/"],
  },
  cache: {
    cacheLocation: "sessionStorage",
    storeAuthStateInCookie: false,
  },
};

export const LoginRequest = {
  scopes: ["openid", "https://apib2clogin.onmicrosoft.com/871d2afa-2389-401d-aaa9-94bf38d05c1d/MaccasApi.ReadWrite"],
};

export const MSALInstance = new PublicClientApplication(MSALConfig);
