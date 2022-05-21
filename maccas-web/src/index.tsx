import React, { Suspense } from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { MSALInstance } from "./config/msal";
import { MsalProvider } from "@azure/msal-react";
import "./index.css";
import { BrowserRouter } from "react-router-dom";
import { ThemeProvider } from "@mui/material/styles";
import { theme } from "./styles";
import { RecoilRoot } from "recoil";

const root = ReactDOM.createRoot(document.getElementById("root") as HTMLElement);

root.render(
  <React.StrictMode>
    <MsalProvider instance={MSALInstance}>
      <RecoilRoot>
        <BrowserRouter>
          <ThemeProvider theme={theme}>
            <App />
          </ThemeProvider>
        </BrowserRouter>
      </RecoilRoot>
    </MsalProvider>
  </React.StrictMode>
);
