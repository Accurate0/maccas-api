import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { MSALInstance } from "./config/msal";
import { MsalProvider } from "@azure/msal-react";
import "./index.css";
import { BrowserRouter } from "react-router-dom";
import { ThemeProvider } from "@mui/material/styles";
import { theme } from "./styles";
import { RecoilRoot } from "recoil";
import Backdrop from "./components/Backdrop";
import { SnackbarProvider } from "notistack";

const root = ReactDOM.createRoot(document.getElementById("root") as HTMLElement);

root.render(
  <React.StrictMode>
    <MsalProvider instance={MSALInstance}>
      <RecoilRoot>
        <BrowserRouter>
          <ThemeProvider theme={theme}>
            <SnackbarProvider>
              <Backdrop />
              <App />
            </SnackbarProvider>
          </ThemeProvider>
        </BrowserRouter>
      </RecoilRoot>
    </MsalProvider>
  </React.StrictMode>
);
