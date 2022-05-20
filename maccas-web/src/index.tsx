import React from "react";
import ReactDOM from "react-dom/client";
import App from "./App";
import { MSALInstance } from "./config/msal";
import { MsalProvider } from "@azure/msal-react";
import "./index.css";
import { BrowserRouter } from "react-router-dom";
import { ThemeProvider } from "@mui/material/styles";
import { theme } from "./styles";

const root = ReactDOM.createRoot(document.getElementById("root") as HTMLElement);

root.render(
  <React.StrictMode>
    <MsalProvider instance={MSALInstance}>
      <BrowserRouter>
        <ThemeProvider theme={theme}>
          <App />
        </ThemeProvider>
      </BrowserRouter>
    </MsalProvider>
  </React.StrictMode>
);
