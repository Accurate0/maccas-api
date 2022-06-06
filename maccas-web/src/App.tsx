import { AuthenticatedTemplate, UnauthenticatedTemplate, useMsal } from "@azure/msal-react";
import { Button, Container, Grid } from "@mui/material";
import { Routes, Route } from "react-router-dom";
import DealSelection from "./pages/DealSelection";
import LocationSelection from "./pages/LocationSelection";
import { LoginRequest } from "./config/msal";
import DealSelector from "./pages/DealSelector";
import NavBar from "./components/NavBar";

const App = () => {
  const { instance } = useMsal();
  return (
    <>
      <UnauthenticatedTemplate>
        <Grid
          container
          spacing={0}
          direction="column"
          alignItems="center"
          justifyContent="center"
          style={{ minHeight: "100vh" }}
        >
          <Grid item xs={3}>
            <Button variant="contained" onClick={() => instance.loginRedirect(LoginRequest)}>
              Login
            </Button>
          </Grid>
        </Grid>
      </UnauthenticatedTemplate>
      <AuthenticatedTemplate>
        <NavBar />
        <Container>
          <Routes>
            <Route path="/" element={<DealSelector />} />
            <Route path="/code" element={<DealSelection />} />
            <Route path="/location" element={<LocationSelection />} />
          </Routes>
        </Container>
      </AuthenticatedTemplate>
    </>
  );
};

export default App;
