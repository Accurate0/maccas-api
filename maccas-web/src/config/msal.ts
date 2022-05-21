import { Configuration, PublicClientApplication } from "@azure/msal-browser";

export const MSALConfig: Configuration = {
  auth: {
    clientId: "f285a3b9-c589-4fae-971e-edd635df6b96",
    authority: "https://apib2clogin.b2clogin.com/apib2clogin.onmicrosoft.com/B2C_1_SignIn",
    knownAuthorities: ["https://apib2clogin.b2clogin.com/"],
  },
  cache: {
    cacheLocation: "sessionStorage",
    storeAuthStateInCookie: false,
  },
};

export const LoginRequest = {
  scopes: ["openid", "https://login.anurag.sh/f285a3b9-c589-4fae-971e-edd635df6b96/Maccas.ReadWrite"],
};

export const MSALInstance = new PublicClientApplication(MSALConfig);
