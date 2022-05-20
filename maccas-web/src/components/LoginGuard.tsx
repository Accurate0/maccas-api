import { useMsal } from "@azure/msal-react";
import { useEffect } from "react";
import { LoginRequest } from "../config/msal";

const LoginGuard = () => {
  const { instance, accounts } = useMsal();

  useEffect(() => {
    if (accounts.length === 0) {
      instance.loginRedirect(LoginRequest);
    }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, []);

  return <></>;
};

export default LoginGuard;
